use std::path::Path;

use iterators::{ColumnIterator, KeysIterator};
use rocksdb::{
    ColumnFamilyDescriptor, DBCompressionType, DBPinnableSlice, Direction, Options, ReadOptions,
};

use crate::{
    columns::{Codec, ColumnSchema},
    error::Error,
};

pub(crate) mod iterators;

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

        let x = 1;
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
            .get_cf(&cf, key)?
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

        Ok(ColumnIterator::new(
            self.rocksdb.raw_iterator_cf_opt(&cf, opts),
            direction,
        ))
    }

    pub(crate) fn delete<C: ColumnSchema>(&self, key: &C::Key) -> Result<(), Error> {
        let cf = self
            .rocksdb
            .cf_handle(C::COLUMN_FAMILY_NAME)
            .ok_or(Error::ColumnFamilyNotFound)?;
        let key = key.encode()?;

        Ok(self.rocksdb.delete_cf(&cf, key)?)
    }
}

/// Definitions for the column families in the pending queue storage.
pub fn pending_db_cf_definitions() -> Vec<ColumnFamilyDescriptor> {
    [
        crate::columns::PENDING_QUEUE_CF,
        crate::columns::PROOF_PER_CERTIFICATE_CF,
    ]
    .iter_mut()
    .map(|cf| {
        let mut cfg = rocksdb::Options::default();

        cfg.set_compression_type(DBCompressionType::Lz4);
        cfg.create_if_missing(true);

        ColumnFamilyDescriptor::new(*cf, cfg)
    })
    .collect()
}

/// Definitions for the column families in the epochs storage.
pub fn epochs_db_cf_definitions() -> Vec<ColumnFamilyDescriptor> {
    [
        crate::columns::PER_EPOCH_METADATA_CF,
        crate::columns::PER_EPOCH_PROOFS_CF,
        crate::columns::PER_EPOCH_CERTIFICATES_CF,
    ]
    .iter_mut()
    .map(|cf| {
        let mut cfg = rocksdb::Options::default();

        cfg.set_compression_type(DBCompressionType::Lz4);
        cfg.create_if_missing(true);

        ColumnFamilyDescriptor::new(*cf, cfg)
    })
    .collect()
}

/// Definitions for the column families in the state storage.
pub fn state_db_cf_definitions() -> Vec<ColumnFamilyDescriptor> {
    [
        crate::columns::LATEST_SETTLED_CERTIFICATE_PER_NETWORK_CF,
        crate::columns::CERTIFICATE_PER_NETWORK_CF,
    ]
    .iter_mut()
    .map(|cf| {
        let mut cfg = rocksdb::Options::default();

        cfg.set_compression_type(DBCompressionType::Lz4);
        cfg.create_if_missing(true);

        ColumnFamilyDescriptor::new(*cf, cfg)
    })
    .collect()
}

/// Definitions for the column families in the metadata storage.
pub fn metadata_db_cf_definitions() -> Vec<ColumnFamilyDescriptor> {
    [crate::columns::METADATA_CF]
        .iter_mut()
        .map(|cf| {
            let mut cfg = rocksdb::Options::default();

            cfg.set_compression_type(DBCompressionType::Lz4);
            cfg.create_if_missing(true);

            ColumnFamilyDescriptor::new(*cf, cfg)
        })
        .collect()
}
