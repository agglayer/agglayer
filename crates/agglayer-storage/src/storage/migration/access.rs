use tracing::error;

use crate::{
    schema::{ColumnDescriptor, ColumnSchema},
    storage::{iterators::KeysIterator, migration::DBMigrationErrorDetails as Error, DB},
};

/// Restricted interface to provide access to the database during migration.
pub struct DbAccess<'a> {
    db: &'a DB,
    writable: &'a [ColumnDescriptor],
}

impl<'a> DbAccess<'a> {
    pub fn new(db: &'a DB, writable: &'a [ColumnDescriptor]) -> Self {
        Self { db, writable }
    }

    fn is_writable<C: ColumnSchema>(&self) -> bool {
        let mut iter = self.writable.iter();
        iter.any(|d| d.name() == C::COLUMN_FAMILY_NAME)
    }

    /// The whole database, for reads that need more than [`Self::get`].
    /// Its column families are open with default options, so prefix
    /// iteration is not available, and writes through it bypass the writable
    /// column families enforced by [`Self::put`] and [`Self::put_all`].
    pub(crate) fn db(&self) -> &'a DB {
        self.db
    }

    /// Try to get the value for the given key.
    pub fn get<C: ColumnSchema>(&self, key: &C::Key) -> Result<Option<C::Value>, Error> {
        Ok(self.db.get::<C>(key)?)
    }

    /// List keys in given column.
    pub fn keys<C: ColumnSchema>(&self) -> Result<KeysIterator<'_, C>, Error> {
        Ok(self.db.keys::<C>()?)
    }

    fn ensure_writable<C: ColumnSchema>(&self) -> Result<(), Error> {
        if self.is_writable::<C>() {
            return Ok(());
        }
        let writing = C::COLUMN_FAMILY_NAME;
        error!(
            writable = ?self.writable,
            writing,
            "Writing a read-only CF {writing:?} during migration",
        );
        Err(Error::WritingReadOnlyCf(writing.to_string()))
    }

    /// Write all `rows` in one atomic batch if allowed.
    pub fn put_all<'b, C: ColumnSchema + 'b>(
        &self,
        rows: impl IntoIterator<Item = (&'b C::Key, &'b C::Value)>,
    ) -> Result<(), Error> {
        self.ensure_writable::<C>()?;
        Ok(self.db.multi_insert::<C>(rows)?)
    }

    /// Write value for given key if allowed.
    pub fn put<C: ColumnSchema>(&self, key: &C::Key, value: &C::Value) -> Result<(), Error> {
        self.ensure_writable::<C>()?;
        Ok(self.db.put::<C>(key, value)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        columns::network_info::NetworkInfoColumn, stores::state::StateStore, tests::TempDBDir,
    };

    #[test_log::test]
    fn put_all_refuses_a_column_family_that_is_not_writable() {
        let tmp = TempDBDir::new();
        let db = StateStore::init_db(&tmp.path, None).unwrap();
        let access = DbAccess::new(&db, &[]);

        assert!(matches!(
            access.put_all::<NetworkInfoColumn>([]),
            Err(Error::WritingReadOnlyCf(_))
        ));
    }
}
