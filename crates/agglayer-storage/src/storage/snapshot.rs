use super::{DBError, DB};
use crate::schema::{Codec, ColumnSchema};

/// Typed reads from one RocksDB snapshot. Keep this borrowed view inside the
/// blocking store operation and return only owned results to async callers.
pub(crate) struct Snapshot<'a> {
    db: &'a DB,
    snapshot: rocksdb::Snapshot<'a>,
}

impl<'a> Snapshot<'a> {
    pub(super) fn new(db: &'a DB) -> Self {
        Self {
            db,
            snapshot: db.rocksdb.snapshot(),
        }
    }

    pub(crate) fn get<C: ColumnSchema>(&self, key: &C::Key) -> Result<Option<C::Value>, DBError> {
        let key = key.encode()?;
        let cf = self.db.cf::<C>()?;

        self.snapshot
            .get_cf(cf, key)?
            .map(|bytes| C::Value::decode(&bytes).map_err(Into::into))
            .transpose()
    }

    pub(crate) fn multi_get<C: ColumnSchema>(
        &self,
        keys: impl IntoIterator<Item = C::Key>,
    ) -> Result<Vec<Option<C::Value>>, DBError> {
        let cf = self.db.cf::<C>()?;
        let keys: Result<Vec<_>, _> = keys
            .into_iter()
            .map(|key| key.encode().map(|key| (cf, key)))
            .collect();

        self.snapshot
            .multi_get_cf(keys?)
            .into_iter()
            .map(|result| {
                result?
                    .map(|bytes| C::Value::decode(&bytes).map_err(Into::into))
                    .transpose()
            })
            .collect()
    }
}
