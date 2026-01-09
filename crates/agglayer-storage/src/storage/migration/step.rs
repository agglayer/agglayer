use tracing::{debug, warn};

use super::{DBMigrationErrorDetails, DbAccess, Migrator};
use crate::{schema::ColumnDescriptor, storage::DBError};

/// Trait alias for migration functions that populate column families with data.
pub trait MigrateFn: FnOnce(&DbAccess) -> Result<(), DBMigrationErrorDetails> {}

impl<F> MigrateFn for F where F: FnOnce(&DbAccess) -> Result<(), DBMigrationErrorDetails> {}

/// Initialize migration tracking (step 0).
#[derive(Debug)]
pub struct Initialize;

impl Initialize {
    pub fn execute(self, _migrator: &mut Migrator) -> Result<(), DBMigrationErrorDetails> {
        // No-op: step 0 is just for tracking initialization
        Ok(())
    }
}

/// Add column families and populate them with data.
pub struct AddColumnFamilies<'a> {
    pub cfs: &'a [ColumnDescriptor],
    pub migrate_fn: Box<dyn MigrateFn + 'a>,
}

impl std::fmt::Debug for AddColumnFamilies<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AddColumnFamilies")
            .field("cfs", &self.cfs)
            .finish_non_exhaustive()
    }
}

impl AddColumnFamilies<'_> {
    pub fn execute(self, migrator: &mut Migrator) -> Result<(), DBMigrationErrorDetails> {
        // Create the column families first
        for descriptor in self.cfs {
            let cf = descriptor.name();
            let opts = descriptor.options().to_rocksdb_options();

            if migrator.cf_exists(cf) {
                warn!("Column family {cf:?} already exists, dropping to create a fresh one");
                migrator.db.rocksdb.drop_cf(cf).map_err(DBError::from)?;
            }

            debug!("Creating column family {cf:?}");
            migrator
                .db
                .rocksdb
                .create_cf(cf, &opts)
                .map_err(DBError::from)?;
        }

        // Use the provided closure to populate it with data
        debug!("Populating new column families with data");
        let access = DbAccess::new(&migrator.db, self.cfs);
        (self.migrate_fn)(&access)?;

        Ok(())
    }
}

/// Drop column families from the database.
#[derive(Debug)]
pub struct DropColumnFamilies<'a> {
    pub cfs: &'a [ColumnDescriptor],
}

impl DropColumnFamilies<'_> {
    pub fn execute(self, migrator: &mut Migrator) -> Result<(), DBMigrationErrorDetails> {
        for descriptor in self.cfs {
            let cf = descriptor.name();

            if migrator.cf_exists(cf) {
                debug!("Dropping column family {cf:?}");
                migrator.db.rocksdb.drop_cf(cf).map_err(DBError::from)?;
            } else {
                warn!("Asked to remove a non-existing column family {cf:?}");
            }
        }
        Ok(())
    }
}

/// Represents a single migration step to be executed.
#[derive(Debug)]
pub enum MigrationStep<'a> {
    /// Initialize migration tracking (step 0).
    Initialize(Initialize),

    /// Add column families and populate them with data.
    AddColumnFamilies(AddColumnFamilies<'a>),

    /// Drop column families from the database.
    DropColumnFamilies(DropColumnFamilies<'a>),
}

impl<'a> MigrationStep<'a> {
    /// Creates an Initialize migration step.
    pub fn initialize() -> Self {
        MigrationStep::Initialize(Initialize)
    }

    /// Creates an AddColumnFamilies migration step.
    pub fn add_cfs<F: MigrateFn + 'a>(cfs: &'a [ColumnDescriptor], migrate_fn: F) -> Self {
        let migrate_fn = Box::new(migrate_fn);
        MigrationStep::AddColumnFamilies(AddColumnFamilies { cfs, migrate_fn })
    }

    /// Creates a DropColumnFamilies migration step.
    pub fn drop_cfs(cfs: &'a [ColumnDescriptor]) -> Self {
        MigrationStep::DropColumnFamilies(DropColumnFamilies { cfs })
    }

    /// Execute this migration step, modifying the database as needed.
    pub fn execute(self, migrator: &mut Migrator) -> Result<(), DBMigrationErrorDetails> {
        match self {
            MigrationStep::Initialize(step) => step.execute(migrator),
            MigrationStep::AddColumnFamilies(step) => step.execute(migrator),
            MigrationStep::DropColumnFamilies(step) => step.execute(migrator),
        }
    }
}
