use std::path::Path;

use iterators::{ColumnIterator, KeysIterator};
use rocksdb::{
    ColumnFamily, ColumnFamilyDescriptor, DBPinnableSlice, Direction, Options, ReadOptions,
    WriteBatch, WriteOptions,
};

use crate::columns::{Codec, ColumnSchema};

pub(crate) mod iterators;
mod migration;

pub mod backup;

pub use migration::{Builder, DBMigrationError, DBMigrationErrorDetails, DBOpenError};

pub(crate) fn default_db_cf_definitions(cfs: &[&'static str]) -> Vec<ColumnFamilyDescriptor> {
    cfs.iter()
        .map(|cf| {
            let mut cfg = rocksdb::Options::default();

            cfg.set_compression_type(rocksdb::DBCompressionType::Lz4);
            cfg.create_if_missing(true);

            ColumnFamilyDescriptor::new(*cf, cfg)
        })
        .collect()
}

#[derive(Debug, thiserror::Error)]
pub enum DBError {
    #[error("RocksDB error: {0}")]
    RocksDB(#[from] rocksdb::Error),

    #[error("Codec error: {0}")]
    CodecError(#[from] crate::columns::CodecError),

    #[error("Trying to access an unknown ColumnFamily")]
    ColumnFamilyNotFound,

    #[error("Database was opened in read-only mode")]
    ReadOnlyMode,
}

#[derive(Debug, thiserror::Error)]
pub enum BackupError {
    #[error("Unable to send backup request")]
    UnableToSendBackupRequest,

    #[error("RocksDB error: {0}")]
    RocksDB(#[from] rocksdb::Error),

    #[error("IO Error: {0}")]
    IO(#[from] std::io::Error),
}

/// A physical storage component with an active RocksDB.
pub struct DB {
    rocksdb: rocksdb::DB,
    default_write_options: Option<WriteOptions>,
}

impl DB {
    /// Open a new RocksDB instance at the given path with initial column families and a
    /// possibility to migrate the database.
    pub fn builder(
        path: &Path,
        cfs: impl IntoIterator<Item = ColumnFamilyDescriptor>,
    ) -> Result<Builder, DBOpenError> {
        Builder::open(path, cfs)
    }

    /// Open a new RocksDB instance at the given path with some column families.
    pub fn open_cf(path: &Path, cfs: Vec<ColumnFamilyDescriptor>) -> Result<DB, DBOpenError> {
        let cf_names: Vec<_> = cfs.iter().map(|cf| cf.name().to_string()).collect();
        Builder::open(path, cfs)?.finalize(cf_names.iter().map(|n| n.as_str()))
    }

    /// Open a RocksDB instance in read-only mode at the given path with some
    /// column families. This prevents concurrency issues when multiple
    /// processes need to read from the database.
    pub fn open_cf_readonly(path: &Path, cfs: Vec<ColumnFamilyDescriptor>) -> Result<DB, DBError> {
        let mut options = Options::default();
        options.create_if_missing(false); // Don't create if missing in readonly mode
        options.create_missing_column_families(false); // Don't create missing column families

        Ok(DB {
            rocksdb: rocksdb::DB::open_cf_descriptors_read_only(&options, path, cfs, false)?,
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
}
