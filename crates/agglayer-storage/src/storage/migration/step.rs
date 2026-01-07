use tracing::{debug, warn};

use crate::{
    schema::ColumnDescriptor,
    storage::{DBError, DB},
};

use super::{error::DBMigrationErrorDetails, Builder};

/// Initialize migration tracking (step 0).
pub struct Initialize;

impl std::fmt::Debug for Initialize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Initialize").finish()
    }
}

impl Initialize {
    pub(super) fn execute(self, _builder: &mut Builder) -> Result<(), DBMigrationErrorDetails> {
        // No-op: step 0 is just for tracking initialization
        Ok(())
    }
}

/// Add column families and populate them with data.
pub struct AddColumnFamilies<'a> {
    pub cfs: &'a [ColumnDescriptor],
    pub migrate_fn: Box<dyn FnOnce(&mut DB) -> Result<(), DBMigrationErrorDetails> + 'a>,
}

impl std::fmt::Debug for AddColumnFamilies<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AddColumnFamilies")
            .field("cfs", &self.cfs)
            .field("migrate_fn", &"<closure>")
            .finish()
    }
}

impl AddColumnFamilies<'_> {
    pub(super) fn execute(self, builder: &mut Builder) -> Result<(), DBMigrationErrorDetails> {
        // Create the column families first
        for descriptor in self.cfs {
            let cf = descriptor.name();
            let opts = descriptor.options().to_rocksdb_options();

            if builder.cf_exists(cf) {
                warn!("Column family {cf:?} already exists, dropping to create a fresh one");
                builder.db.rocksdb.drop_cf(cf).map_err(DBError::from)?;
            }

            debug!("Creating column family {cf:?}");
            builder
                .db
                .rocksdb
                .create_cf(cf, &opts)
                .map_err(DBError::from)?;
        }

        // Use the provided closure to populate it with data
        debug!("Populating new column families with data");
        (self.migrate_fn)(&mut builder.db)?;

        Ok(())
    }
}

/// Drop column families from the database.
pub struct DropColumnFamilies<'a> {
    pub cfs: &'a [ColumnDescriptor],
}

impl std::fmt::Debug for DropColumnFamilies<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DropColumnFamilies")
            .field("cfs", &self.cfs)
            .finish()
    }
}

impl DropColumnFamilies<'_> {
    pub(super) fn execute(self, builder: &mut Builder) -> Result<(), DBMigrationErrorDetails> {
        for descriptor in self.cfs {
            let cf = descriptor.name();

            if builder.cf_exists(cf) {
                debug!("Dropping column family {cf:?}");
                builder.db.rocksdb.drop_cf(cf).map_err(DBError::from)?;
            } else {
                warn!("Asked to remove a non-existing column family {cf:?}");
            }
        }
        Ok(())
    }
}

/// Represents a single migration step to be executed.
pub enum MigrationStep<'a> {
    /// Initialize migration tracking (step 0).
    Initialize(Initialize),

    /// Add column families and populate them with data.
    AddColumnFamilies(AddColumnFamilies<'a>),

    /// Drop column families from the database.
    DropColumnFamilies(DropColumnFamilies<'a>),
}

impl std::fmt::Debug for MigrationStep<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Initialize(step) => step.fmt(f),
            Self::AddColumnFamilies(step) => step.fmt(f),
            Self::DropColumnFamilies(step) => step.fmt(f),
        }
    }
}

impl<'a> MigrationStep<'a> {
    /// Creates an Initialize migration step.
    pub(super) fn initialize() -> Self {
        MigrationStep::Initialize(Initialize)
    }

    /// Creates an AddColumnFamilies migration step.
    pub(super) fn add_cfs(
        cfs: &'a [ColumnDescriptor],
        migrate_fn: Box<dyn FnOnce(&mut DB) -> Result<(), DBMigrationErrorDetails> + 'a>,
    ) -> Self {
        MigrationStep::AddColumnFamilies(AddColumnFamilies { cfs, migrate_fn })
    }

    /// Creates a DropColumnFamilies migration step.
    pub(super) fn drop_cfs(cfs: &'a [ColumnDescriptor]) -> Self {
        MigrationStep::DropColumnFamilies(DropColumnFamilies { cfs })
    }

    /// Execute this migration step, modifying the database as needed.
    pub(super) fn execute(self, builder: &mut Builder) -> Result<(), DBMigrationErrorDetails> {
        match self {
            MigrationStep::Initialize(step) => step.execute(builder),
            MigrationStep::AddColumnFamilies(step) => step.execute(builder),
            MigrationStep::DropColumnFamilies(step) => step.execute(builder),
        }
    }
}
