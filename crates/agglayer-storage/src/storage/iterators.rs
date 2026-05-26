use super::DBError;
use crate::schema::{Codec as _, ColumnSchema};

/// The status of the iterator.
enum IteratorStatus {
    /// The iterator is initialized and ready to be used.
    Initialized,
    /// The iterator is currently progressing.
    Progressing,
    /// The iterator has reached the end.
    Done,
}

/// An iterator over the keys of a column.
pub struct KeysIterator<'a, C: ColumnSchema> {
    iter: rocksdb::DBRawIterator<'a>,
    status: IteratorStatus,
    direction: rocksdb::Direction,
    _phantom: std::marker::PhantomData<C>,
}

fn invalid_iterator_error(status: Result<(), rocksdb::Error>) -> Option<DBError> {
    status.err().map(Into::into)
}

// Related issue: https://github.com/rust-lang/rust-clippy/issues/12908
#[allow(clippy::needless_lifetimes)]
impl<'a, C: ColumnSchema> KeysIterator<'a, C> {
    /// Creates a new iterator over the keys of a column using the given raw
    /// iterator and a direction.
    pub(crate) fn new(iter: rocksdb::DBRawIterator<'a>, direction: rocksdb::Direction) -> Self {
        Self {
            iter,
            direction,
            status: IteratorStatus::Initialized,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<C: ColumnSchema> Iterator for KeysIterator<'_, C> {
    type Item = Result<C::Key, DBError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.status {
            IteratorStatus::Done => {
                return None;
            }
            IteratorStatus::Initialized => {
                self.status = IteratorStatus::Progressing;
            }

            IteratorStatus::Progressing => match self.direction {
                rocksdb::Direction::Forward => self.iter.next(),
                rocksdb::Direction::Reverse => self.iter.prev(),
            },
        }

        // If the iterator is invalid, return None
        if !self.iter.valid() {
            self.status = IteratorStatus::Done;

            return invalid_iterator_error(self.iter.status()).map(Err);
        }

        self.iter
            .key()
            .map(|v| C::Key::decode(v).map_err(Into::into))
    }
}

pub struct ColumnIterator<'a, C: ColumnSchema> {
    iter: rocksdb::DBRawIterator<'a>,
    status: IteratorStatus,
    direction: rocksdb::Direction,
    _phantom: std::marker::PhantomData<C>,
}

type KeyValueResult<K, V> = Result<Option<(K, V)>, DBError>;

impl<'a, C: ColumnSchema> ColumnIterator<'a, C> {
    pub(crate) fn new(iter: rocksdb::DBRawIterator<'a>, direction: rocksdb::Direction) -> Self {
        Self {
            iter,
            direction,
            status: IteratorStatus::Initialized,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Seeks to the first entry whose key is greater than or equal to `key`.
    pub fn seek(&mut self, key: &C::Key) -> Result<(), DBError> {
        let encoded = C::Key::encode(key)?;
        self.iter.seek(&encoded);
        self.status = IteratorStatus::Initialized;

        Ok(())
    }

    fn parse_key_value(&self) -> KeyValueResult<C::Key, C::Value> {
        let key = self.iter.key().map(C::Key::decode).transpose()?;
        let value = self.iter.value().map(C::Value::decode).transpose()?;

        Ok(key.zip(value))
    }
}

impl<C: ColumnSchema> Iterator for ColumnIterator<'_, C> {
    type Item = Result<(C::Key, C::Value), DBError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.status {
            IteratorStatus::Done => {
                return None;
            }
            IteratorStatus::Initialized => {
                self.status = IteratorStatus::Progressing;
            }

            IteratorStatus::Progressing => match self.direction {
                rocksdb::Direction::Forward => self.iter.next(),
                rocksdb::Direction::Reverse => self.iter.prev(),
            },
        }

        // If the iterator is invalid, return None
        if !self.iter.valid() {
            self.status = IteratorStatus::Done;

            return invalid_iterator_error(self.iter.status()).map(Err);
        }

        self.parse_key_value().transpose()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_iterator_error_converts_rocksdb_status() {
        let missing_path = std::env::temp_dir().join(format!(
            "agglayer-storage-missing-rocksdb-{}",
            ulid::Ulid::new()
        ));
        let rocksdb_error =
            rocksdb::DB::open_for_read_only(&rocksdb::Options::default(), missing_path, false)
                .expect_err("opening a missing DB read-only should fail");

        let error = invalid_iterator_error(Err(rocksdb_error))
            .expect("rocksdb status error should be propagated");

        assert!(matches!(error, DBError::RocksDB(_)));
        assert!(error.to_string().contains("RocksDB error"));
    }
}
