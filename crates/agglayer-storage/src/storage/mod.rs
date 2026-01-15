use std::path::Path;

use iterators::{ColumnIterator, KeysIterator};
use rocksdb::{
    ColumnFamily, ColumnFamilyDescriptor, DBCompressionType, DBPinnableSlice, Direction, Options,
    ReadOptions, SliceTransform, WriteBatch, WriteOptions,
};

use crate::schema::{
    options::{ColumnCompressionType, ColumnOptions, PrefixExtractor},
    Codec, ColumnDescriptor, ColumnSchema,
};

pub(crate) mod iterators;
mod migration;

pub use migration::{Builder, DBMigrationError, DBMigrationErrorDetails, DBOpenError, MigrateFn};

#[derive(Debug, thiserror::Error)]
pub enum DBError {
    #[error("RocksDB error: {0}")]
    RocksDB(#[from] rocksdb::Error),

    #[error("Codec error: {0}")]
    CodecError(#[from] crate::schema::CodecError),

    #[error("Trying to access an unknown ColumnFamily")]
    ColumnFamilyNotFound,

    #[error("Database was opened in read-only mode")]
    ReadOnlyMode,
}

/// A physical storage component with an active RocksDB.
pub struct DB {
    rocksdb: rocksdb::DB,
    default_write_options: Option<WriteOptions>,
}

impl DB {
    /// Create a migration builder with the initial database schema.
    pub fn builder(initial_schema: &[ColumnDescriptor]) -> Builder<'_> {
        Builder::new(initial_schema)
    }

    /// Open a new RocksDB instance at the given path with some column families.
    pub fn open_cf(path: &Path, cfs: &[ColumnDescriptor]) -> Result<DB, DBOpenError> {
        Self::builder(cfs).finalize(cfs)?.open(path)?.migrate()
    }

    /// Open a RocksDB instance in read-only mode at the given path with some
    /// column families. This prevents concurrency issues when multiple
    /// processes need to read from the database.
    pub fn open_cf_readonly(path: &Path, cfs: &[ColumnDescriptor]) -> Result<DB, DBError> {
        let mut options = Options::default();
        options.create_if_missing(false); // Don't create if missing in readonly mode
        options.create_missing_column_families(false); // Don't create missing column families

        let descriptors: Vec<_> = cfs.iter().map(Self::descriptor).collect();
        Ok(DB {
            rocksdb: rocksdb::DB::open_cf_descriptors_read_only(
                &options,
                path,
                descriptors,
                false,
            )?,
            default_write_options: None,
        })
    }

    fn write_options(&self) -> Result<&WriteOptions, DBError> {
        self.default_write_options
            .as_ref()
            .ok_or(DBError::ReadOnlyMode)
    }

    fn cf<C: ColumnSchema>(&self) -> Result<&ColumnFamily, DBError> {
        self.rocksdb
            .cf_handle(C::COLUMN_FAMILY_NAME)
            .ok_or(DBError::ColumnFamilyNotFound)
    }

    /// Check if a column family exists in the database.
    pub(crate) fn cf_exists(&self, cf: &str) -> bool {
        self.rocksdb.cf_handle(cf).is_some()
    }

    /// Try to get the value for the given key.
    pub fn get<C: ColumnSchema>(&self, key: &C::Key) -> Result<Option<C::Value>, DBError> {
        let key = key.encode()?;
        let cf = self.cf::<C>()?;

        self.rocksdb
            .get_cf(cf, &key)?
            .map(|v| C::Value::decode(&v[..]).map_err(Into::into))
            // If the value is not found, return None.
            // If the value is found, decode it and wrap it in Some to propagate decode error.
            .map_or(Ok(None), |v| v.map(Some))
    }

    pub fn atomic_multi_get<C: ColumnSchema>(
        &self,
        keys: impl IntoIterator<Item = C::Key>,
    ) -> Result<Vec<Option<C::Value>>, DBError> {
        let snapshot = self.rocksdb.snapshot();
        let cf = self
            .rocksdb
            .cf_handle(C::COLUMN_FAMILY_NAME)
            .ok_or(DBError::ColumnFamilyNotFound)?;

        let keys: Result<Vec<_>, _> = keys
            .into_iter()
            .map(|k| k.encode().map(|key| (cf, key)))
            .collect();

        let results = snapshot
            .multi_get_cf(keys?)
            .into_iter()
            .map(|r| r.map_err(DBError::from))
            .collect::<Result<Vec<Option<_>>, _>>()?;

        results
            .into_iter()
            .map(|bytes| match bytes {
                Some(bytes) => C::Value::decode(&bytes[..]).map_err(Into::into).map(Some),
                None => Ok(None),
            })
            .collect()
    }

    pub fn multi_get<C: ColumnSchema>(
        &self,
        keys: impl IntoIterator<Item = C::Key>,
    ) -> Result<Vec<Option<C::Value>>, DBError> {
        let cf = self.cf::<C>()?;
        let keys: Result<Vec<_>, _> = keys.into_iter().map(|k| k.encode()).collect();

        let results: Result<Vec<Option<DBPinnableSlice>>, _> = self
            .rocksdb
            .batched_multi_get_cf(cf, &keys?, false)
            .into_iter()
            .map(|r| r.map_err(DBError::from))
            .collect();

        results?
            .into_iter()
            .map(|bytes| match bytes {
                Some(bytes) => C::Value::decode(&bytes[..]).map_err(Into::into).map(Some),
                None => Ok(None),
            })
            .collect()
    }

    /// Try to put the value for the given key.
    pub fn put<C: ColumnSchema>(&self, key: &C::Key, value: &C::Value) -> Result<(), DBError> {
        let key = key.encode()?;
        let value = value.encode()?;
        let cf = self.cf::<C>()?;

        let write_options = self.write_options()?;
        self.rocksdb.put_cf_opt(cf, key, value, write_options)?;

        Ok(())
    }

    pub fn write_batch(&self, batch: WriteBatch) -> Result<(), DBError> {
        let write_options = self.write_options()?;
        self.rocksdb.write_opt(batch, write_options)?;

        Ok(())
    }

    pub fn multi_insert_batch<'a, C: ColumnSchema + 'a>(
        &self,
        key_val_pairs: impl IntoIterator<Item = (&'a C::Key, &'a C::Value)>,
        batch: &mut WriteBatch,
    ) -> Result<(), DBError> {
        let cf = self.cf::<C>()?;

        key_val_pairs
            .into_iter()
            .try_for_each::<_, Result<_, DBError>>(|(k, v)| {
                let k_buf = k.encode()?;
                let v_buf = v.encode()?;

                batch.put_cf(&cf, k_buf, v_buf);
                Ok(())
            })?;

        Ok(())
    }

    pub fn multi_insert<'a, C: ColumnSchema + 'a>(
        &self,
        key_val_pairs: impl IntoIterator<Item = (&'a C::Key, &'a C::Value)>,
    ) -> Result<(), DBError> {
        let mut batch = WriteBatch::default();
        self.multi_insert_batch::<C>(key_val_pairs, &mut batch)?;
        self.write_batch(batch)?;

        Ok(())
    }

    /// Try to get every key in the column family.
    pub fn keys<C: ColumnSchema>(&self) -> Result<KeysIterator<'_, C>, DBError> {
        let cf = self.cf::<C>()?;

        let mut iterator = self.rocksdb.raw_iterator_cf(&cf);
        iterator.seek_to_first();

        Ok(KeysIterator::new(iterator, Direction::Forward))
    }

    pub(crate) fn iter_with_direction<C: ColumnSchema>(
        &self,
        opts: ReadOptions,
        direction: Direction,
    ) -> Result<ColumnIterator<'_, C>, DBError> {
        let cf = self.cf::<C>()?;

        let mut iterator = self.rocksdb.raw_iterator_cf_opt(&cf, opts);

        match direction {
            Direction::Forward => iterator.seek_to_first(),
            Direction::Reverse => iterator.seek_to_last(),
        }

        Ok(ColumnIterator::new(iterator, direction))
    }

    pub(crate) fn delete<C: ColumnSchema>(&self, key: &C::Key) -> Result<(), DBError> {
        let cf = self.cf::<C>()?;
        let key = key.encode()?;

        let write_options = self.write_options()?;
        Ok(self.rocksdb.delete_cf_opt(&cf, key, write_options)?)
    }

    pub(crate) fn raw_rocksdb(&self) -> &rocksdb::DB {
        &self.rocksdb
    }

    // Convert a ColumnDescriptor to a RocksDB ColumnFamilyDescriptor.
    fn descriptor(descriptor: &ColumnDescriptor) -> ColumnFamilyDescriptor {
        ColumnFamilyDescriptor::new(descriptor.name(), Self::options(descriptor.options()))
    }

    // Convert ColumnOptions to RocksDB Options.
    fn options(options: &ColumnOptions) -> Options {
        let mut opts = Options::default();

        // Set compression type
        let compression = match options.compression {
            ColumnCompressionType::None => DBCompressionType::None,
            ColumnCompressionType::Lz4 => DBCompressionType::Lz4,
        };
        opts.set_compression_type(compression);

        // Set prefix extractor
        match options.prefix_extractor {
            PrefixExtractor::Default => {}
            PrefixExtractor::Fixed { size } => {
                opts.set_prefix_extractor(SliceTransform::create_fixed_prefix(size));
            }
        }

        opts
    }
}
