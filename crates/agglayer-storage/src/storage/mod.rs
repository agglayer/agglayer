use std::path::Path;

use bincode::Options as _;
use iterators::{ColumnIterator, KeysIterator};
use rocksdb::{
    ColumnFamilyDescriptor, DBPinnableSlice, DBRawIteratorWithThreadMode, Direction, Options,
    ReadOptions, WriteBatch,
};
use serde::Serialize;

use crate::{
    columns::{default_bincode_options, Codec, ColumnSchema},
    error::Error,
};

pub(crate) mod cf_definitions;
pub(crate) mod iterators;

pub use cf_definitions::epochs::epochs_db_cf_definitions;
pub use cf_definitions::local_network_state::local_network_state_db_cf_definitions;
pub use cf_definitions::pending::pending_db_cf_definitions;
pub use cf_definitions::state::state_db_cf_definitions;

/// A physical storage storage component with an active RocksDB.
#[derive(Debug)]
pub struct DB {
    rocksdb: rocksdb::DB,
}

impl DB {
    /// Open a new RocksDB instance at the given path with some column families.
    pub fn open_cf(path: &Path, cfs: Vec<ColumnFamilyDescriptor>) -> Result<DB, Error> {
        let mut options = Options::default();
        options.create_if_missing(true);
        options.create_missing_column_families(true);

        Ok(DB {
            rocksdb: rocksdb::DB::open_cf_descriptors(&options, path, cfs)?,
        })
    }

    /// Try to get the value for the given key.
    pub fn get<C: ColumnSchema>(&self, key: &C::Key) -> Result<Option<C::Value>, Error> {
        let key = key.encode()?;
        let cf = self
            .rocksdb
            .cf_handle(C::COLUMN_FAMILY_NAME)
            .ok_or(Error::ColumnFamilyNotFound)?;

        self.rocksdb
            .get_cf(&cf, &key)?
            .map(|v| C::Value::decode(&v[..]))
            // If the value is not found, return None.
            // If the value is found, decode it and wrap it in Some to propagate decode error.
            .map_or(Ok(None), |v| v.map(Some))
    }

    pub fn multi_get<C: ColumnSchema>(
        &self,
        keys: impl IntoIterator<Item = C::Key>,
    ) -> Result<Vec<Option<C::Value>>, Error> {
        let cf = self
            .rocksdb
            .cf_handle(C::COLUMN_FAMILY_NAME)
            .ok_or(Error::ColumnFamilyNotFound)?;

        let keys: Result<Vec<_>, _> = keys.into_iter().map(|k| k.encode()).collect();

        let results: Result<Vec<Option<DBPinnableSlice>>, _> = self
            .rocksdb
            .batched_multi_get_cf(cf, &keys?, false)
            .into_iter()
            .map(|r| r.map_err(Error::from))
            .collect();

        results?
            .into_iter()
            .map(|bytes| match bytes {
                Some(bytes) => C::Value::decode(&bytes[..]).map(Some),
                None => Ok(None),
            })
            .collect()
    }

    /// Try to put the value for the given key.
    pub fn put<C: ColumnSchema>(&self, key: &C::Key, value: &C::Value) -> Result<(), Error> {
        let key = key.encode()?;
        let value = value.encode()?;
        let cf = self
            .rocksdb
            .cf_handle(C::COLUMN_FAMILY_NAME)
            .ok_or(Error::ColumnFamilyNotFound)?;

        self.rocksdb.put_cf(&cf, key, value)?;

        Ok(())
    }

    pub fn multi_insert<'a, C: ColumnSchema + 'a>(
        &self,
        key_val_pairs: impl IntoIterator<Item = (&'a C::Key, &'a C::Value)>,
    ) -> Result<(), Error> {
        let cf = self
            .rocksdb
            .cf_handle(C::COLUMN_FAMILY_NAME)
            .ok_or(Error::ColumnFamilyNotFound)?;

        let mut batch = WriteBatch::default();

        key_val_pairs
            .into_iter()
            .try_for_each::<_, Result<_, Error>>(|(k, v)| {
                let k_buf = k.encode()?;
                let v_buf = v.encode()?;

                batch.put_cf(&cf, k_buf, v_buf);
                Ok(())
            })?;

        self.rocksdb.write(batch)?;

        Ok(())
    }

    /// Try to get every key in the column family.
    pub fn keys<C: ColumnSchema>(&self) -> Result<KeysIterator<C>, Error> {
        let cf = self
            .rocksdb
            .cf_handle(C::COLUMN_FAMILY_NAME)
            .ok_or(Error::ColumnFamilyNotFound)?;

        let mut iterator = self.rocksdb.raw_iterator_cf(&cf);
        iterator.seek_to_first();

        Ok(KeysIterator::new(iterator, Direction::Forward))
    }

    pub(crate) fn iter_with_direction<C: ColumnSchema>(
        &self,
        opts: ReadOptions,
        direction: Direction,
    ) -> Result<ColumnIterator<C>, Error> {
        let cf = self
            .rocksdb
            .cf_handle(C::COLUMN_FAMILY_NAME)
            .ok_or(Error::ColumnFamilyNotFound)?;

        let mut iterator = self.rocksdb.raw_iterator_cf_opt(&cf, opts);

        match direction {
            Direction::Forward => iterator.seek_to_first(),
            Direction::Reverse => iterator.seek_to_last(),
        }

        Ok(ColumnIterator::new(iterator, direction))
    }

    pub(crate) fn delete<C: ColumnSchema>(&self, key: &C::Key) -> Result<(), Error> {
        let cf = self
            .rocksdb
            .cf_handle(C::COLUMN_FAMILY_NAME)
            .ok_or(Error::ColumnFamilyNotFound)?;
        let key = key.encode()?;

        Ok(self.rocksdb.delete_cf(&cf, key)?)
    }

    pub(crate) fn prefix_itererator_with_direction<C: ColumnSchema, P: Serialize>(
        &self,
        prefix: &P,
        direction: Direction,
    ) -> Result<ColumnIterator<C>, Error> {
        let cf = self
            .rocksdb
            .cf_handle(C::COLUMN_FAMILY_NAME)
            .ok_or(Error::ColumnFamilyNotFound)?;

        let mut iterator: DBRawIteratorWithThreadMode<_> = self
            .rocksdb
            .prefix_iterator_cf(&cf, default_bincode_options().serialize(prefix)?)
            .into();

        match direction {
            Direction::Forward => iterator.seek_to_first(),
            Direction::Reverse => iterator.seek_to_last(),
        }

        Ok(ColumnIterator::new(iterator, direction))
    }
}
