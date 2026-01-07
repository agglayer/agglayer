use tracing::{debug, warn};

use crate::{
    schema::ColumnDescriptor,
    storage::{DBError, DB},
};

use super::{error::DBMigrationErrorDetails, Builder};

/// Represents a single migration step to be executed.
pub enum MigrationStep<'a> {
    /// Initialize migration tracking (step 0).
    Initialize,

    /// Add column families and populate them with data.
    AddColumnFamilies {
        cfs: &'a [ColumnDescriptor],
        migrate_fn: Box<dyn FnOnce(&mut DB) -> Result<(), DBMigrationErrorDetails> + 'a>,
    },

    /// Drop column families from the database.
    DropColumnFamilies { cfs: &'a [ColumnDescriptor] },
}

impl std::fmt::Debug for MigrationStep<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Initialize => f.debug_struct("Initialize").finish(),
            Self::AddColumnFamilies { cfs, .. } => f
                .debug_struct("AddColumnFamilies")
                .field("cfs", cfs)
                .field("migrate_fn", &"<closure>")
                .finish(),
            Self::DropColumnFamilies { cfs } => f
                .debug_struct("DropColumnFamilies")
                .field("cfs", cfs)
                .finish(),
        }
    }
}

impl MigrationStep<'_> {
    /// Execute this migration step, modifying the database as needed.
    pub fn execute(
        self,
        builder: &mut Builder,
    ) -> Result<(), DBMigrationErrorDetails> {
        match self {
            MigrationStep::Initialize => {
                // No-op: step 0 is just for tracking initialization
                Ok(())
            }

            MigrationStep::AddColumnFamilies { cfs, migrate_fn } => {
                // Create the column families first
                for descriptor in cfs {
                    let cf = descriptor.name();
                    let opts = descriptor.options().to_rocksdb_options();

                    if builder.cf_exists(cf) {
                        warn!(
                            "Column family {cf:?} already exists, dropping to create a fresh one"
                        );
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
                migrate_fn(&mut builder.db)?;

                Ok(())
            }

            MigrationStep::DropColumnFamilies { cfs } => {
                for descriptor in cfs {
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
    }
}
