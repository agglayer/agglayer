use tracing::debug;

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
