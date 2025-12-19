use tracing::debug;

use super::DBError;
use crate::columns::{Codec as _, ColumnSchema};

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

            return None;
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

    fn parse_key_value(&self) -> KeyValueResult<C::Key, C::Value> {
        let key = self.iter.key().map(C::Key::decode).transpose()?;
        let value = self.iter.value().map(C::Value::decode).transpose()?;

        Ok(key.zip(value))
    }

    /// Seeks to the first key.
    #[allow(unused)]
    pub fn seek_to_first(&mut self) {
        self.iter.seek_to_first();
    }

    /// Seeks to the last key.
    #[allow(unused)]
    pub fn seek_to_last(&mut self) {
        self.iter.seek_to_last();
    }

    /// Seeks for the first key (binary equal to or greater)
    #[allow(unused)]
    pub fn seek(&mut self, seek_key: &C::Key) -> Result<(), DBError> {
        let key = seek_key.encode()?;
        self.iter.seek(&key);

        Ok(())
    }

    /// Seeks for the last key (binary equal to or less)
    #[allow(unused)]
    pub fn seek_for_prev(&mut self, seek_key: &C::Key) -> Result<(), DBError> {
        let key = seek_key.encode()?;
        self.iter.seek_for_prev(&key);

        Ok(())
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
            if let Err(error) = self.iter.status() {
                debug!("Invalid iterator {}", error);
            }

            return None;
        }

        self.parse_key_value().transpose()
    }
}
