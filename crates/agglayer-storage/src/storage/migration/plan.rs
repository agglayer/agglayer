use std::path::Path;

use super::{error::DBOpenError, step::MigrationStep, Migrator};
use crate::schema::ColumnDescriptor;

/// Complete migration specification.
#[derive(Debug)]
pub struct MigrationPlan<'a> {
    // Initial database schema.
    pub(super) initial_schema: &'a [ColumnDescriptor],

    // The sequence of migration steps.
    pub(super) steps: Vec<MigrationStep<'a>>,

    // Final schema, reserved for future use.
    #[expect(unused)]
    final_schema: &'a [ColumnDescriptor],
}

impl<'a> MigrationPlan<'a> {
    /// Opens and validates the database, returning a [Migrator].
    pub fn open(self, path: &Path) -> Result<Migrator<'a>, DBOpenError> {
        Migrator::open(path, self)
    }
}

/// Builder for declaring database migration steps.
#[derive(Debug)]
pub struct Builder<'a> {
    // Initial database schema (v0).
    initial_schema: &'a [ColumnDescriptor],

    // Collected migration steps to be executed later.
    steps: Vec<MigrationStep<'a>>,
}

impl<'a> Builder<'a> {
    /// Creates a new migration builder with the initial database schema.
    pub fn new(initial_schema: &'a [ColumnDescriptor]) -> Self {
        Self {
            initial_schema,
            steps: vec![MigrationStep::initialize()],
        }
    }

    /// Creates new column families and populates them with data.
    ///
    /// This is a migration step that creates column families and runs the
    /// provided migration function to populate them. The migration function
    /// should only write into the newly created column families.
    pub fn add_cfs<F: super::MigrateFn + 'a>(
        self,
        cfs: &'a [ColumnDescriptor],
        migrate_fn: F,
    ) -> Result<Self, DBOpenError> {
        Ok(self.add_step(MigrationStep::add_cfs(cfs, migrate_fn)))
    }

    /// Removes old column families from the database.
    pub fn drop_cfs(self, cfs: &'a [ColumnDescriptor]) -> Result<Self, DBOpenError> {
        Ok(self.add_step(MigrationStep::drop_cfs(cfs)))
    }

    /// Completes the declaration and creates a migration plan.
    pub fn finalize(
        self,
        final_schema: &'a [ColumnDescriptor],
    ) -> Result<MigrationPlan<'a>, DBOpenError> {
        Ok(MigrationPlan {
            initial_schema: self.initial_schema,
            final_schema,
            steps: self.steps,
        })
    }

    /// Helper method to add a migration step to the list.
    fn add_step(mut self, migration_step: MigrationStep<'a>) -> Self {
        self.steps.push(migration_step);
        self
    }
}
