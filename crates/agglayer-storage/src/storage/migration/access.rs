use std::collections::BTreeSet;

use tracing::error;

use crate::{
    schema::ColumnSchema,
    storage::{iterators::KeysIterator, migration::DBMigrationErrorDetails as Error, DB},
};

/// Restricted interface to provide access to the database during migration.
pub struct DbAccess<'a> {
    db: &'a DB,
    writable: &'a BTreeSet<&'a str>,
}

impl<'a> DbAccess<'a> {
    pub fn new(db: &'a DB, writable: &'a BTreeSet<&'a str>) -> Self {
        Self { db, writable }
    }

    fn is_writable<C: ColumnSchema>(&self) -> bool {
        self.writable.contains(C::COLUMN_FAMILY_NAME)
    }

    /// Try to get the value for the given key.
    pub fn get<C: ColumnSchema>(&self, key: &C::Key) -> Result<Option<C::Value>, Error> {
        Ok(self.db.get::<C>(key)?)
    }

    /// List keys in given column.
    pub fn keys<C: ColumnSchema>(&self) -> Result<KeysIterator<'_, C>, Error> {
        Ok(self.db.keys::<C>()?)
    }

    /// Write value for given key if allowed.
    pub fn put<C: ColumnSchema>(&self, key: &C::Key, value: &C::Value) -> Result<(), Error> {
        if !self.is_writable::<C>() {
            let writing = C::COLUMN_FAMILY_NAME;
            error!(
                writable = ?self.writable,
                writing,
                "Writing a read-only CF {writing:?} during migration",
            );
            return Err(Error::WritingReadOnlyCf(writing.to_string()));
        }

        Ok(self.db.put::<C>(key, value)?)
    }
}
