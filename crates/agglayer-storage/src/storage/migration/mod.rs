use std::{collections::BTreeSet, path::Path};

use rocksdb::ColumnFamilyDescriptor;
use tracing::{debug, info};

pub use self::{
    access::DbAccess,
    error::{DBMigrationError, DBMigrationErrorDetails, DBOpenError},
    plan::{Builder, MigrationPlan},
    step::MigrateFn,
};
use self::{migration_cf::MigrationRecordColumn, record::MigrationRecord, step::MigrationStep};
use crate::{
    schema::{ColumnDescriptor, ColumnSchema},
    storage::{DBError, DB},
};

mod access;
mod error;
mod migration_cf;
mod plan;
mod record;
mod step;

/// Database migrator that holds an open database and executes migration steps.
pub struct Migrator<'a> {
    db: DB,
    start_step: u32,
    steps: Vec<MigrationStep<'a>>,
}

impl std::fmt::Debug for Migrator<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Migrator")
            .field("start_step", &self.start_step)
            .field("steps", &self.steps)
            .finish_non_exhaustive()
    }
}

impl<'a> Migrator<'a> {
    /// Opens the database and validates the migration plan.
    pub fn open(path: &Path, plan: MigrationPlan<'a>) -> Result<Self, DBOpenError> {
        debug!("Preparing database for initialization and migration");
        let desc_v0: Vec<_> = plan
            .initial_schema
            .iter()
            .map(|cd| cd.to_rocksdb_descriptor())
            .collect();
        let cfs_v0: BTreeSet<_> = desc_v0.iter().map(|d| d.name()).collect();

        // Try to extract the current database schema.
        let cfs_db = match rocksdb::DB::list_cf(&rocksdb::Options::default(), path) {
            Ok(cfs) => {
                let mut cfs = BTreeSet::from_iter(cfs);
                cfs.remove(rocksdb::DEFAULT_COLUMN_FAMILY_NAME);
                debug!(?cfs, "Extracted existing database schema");
                cfs
            }
            Err(error) => {
                debug!(%error, "Failed to extract database schema, assuming empty");
                BTreeSet::new()
            }
        };

        // Figure out whether schema matches one of the expected patterns
        // and the set of column families the db should be open with.
        let db = if cfs_db.is_empty() {
            // We are initializing a new database.
            let mut cfs = desc_v0;
            cfs.push(ColumnDescriptor::new::<MigrationRecordColumn>().to_rocksdb_descriptor());
            Self::open_rocksdb_fresh(path, cfs)?
        } else if cfs_db.contains(MigrationRecordColumn::COLUMN_FAMILY_NAME) {
            // Move on to migration as usual.
            Self::open_rocksdb_existing(path, &cfs_db)?
        } else if cfs_db.iter().eq(cfs_v0.iter()) {
            // Initialize migration record.
            let mut cfs = cfs_db;
            cfs.insert(MigrationRecordColumn::COLUMN_FAMILY_NAME.into());
            Self::open_rocksdb_existing(path, &cfs)?
        } else {
            // Unexpected schema.
            return Err(DBOpenError::UnexpectedSchema);
        };

        {
            // Check the default CF is empty. If not, it is an indication that the database
            // file is being used for something else.
            let mut default_cf_iter = db.rocksdb.iterator(rocksdb::IteratorMode::Start);
            let default_cf_has_data = default_cf_iter.next().is_some();
            if default_cf_has_data {
                return Err(DBOpenError::DefaultCfNotEmpty);
            }
        }

        // Check migration record for gaps, and get the corresponding value.
        let start_step = {
            let mut step = 0_u32;
            for stored_step in db.keys::<MigrationRecordColumn>()? {
                if stored_step? != step {
                    return Err(DBOpenError::MigrationRecordGap(step));
                }
                step += 1;
            }
            step
        };

        // Validate that we have enough steps declared for the recorded migrations
        let declared_steps = plan.steps.len() as u32;
        if declared_steps < start_step {
            return Err(DBOpenError::FewerStepsDeclared {
                declared: declared_steps,
                recorded: start_step,
            });
        }

        Ok(Migrator {
            db,
            start_step,
            steps: plan.steps,
        })
    }

    /// Helper to check if a column family exists in the database.
    pub(crate) fn cf_exists(&self, cf: &str) -> bool {
        self.db.rocksdb.cf_handle(cf).is_some()
    }

    fn writeopts() -> rocksdb::WriteOptions {
        let mut writeopts = rocksdb::WriteOptions::default();
        writeopts.set_sync(true);
        writeopts
    }

    fn open_rocksdb_fresh(path: &Path, cfs: Vec<ColumnFamilyDescriptor>) -> Result<DB, DBError> {
        debug!("Opening fresh database");

        let mut options = rocksdb::Options::default();
        options.create_if_missing(true);
        options.create_missing_column_families(true);

        Ok(DB {
            rocksdb: rocksdb::DB::open_cf_descriptors(&options, path, cfs)?,
            default_write_options: Some(Self::writeopts()),
        })
    }

    fn open_rocksdb_existing(path: &Path, cfs: &BTreeSet<impl AsRef<str>>) -> Result<DB, DBError> {
        debug!("Opening existing database");

        let mut options = rocksdb::Options::default();
        options.create_missing_column_families(true);

        Ok(DB {
            rocksdb: rocksdb::DB::open_cf(&options, path, cfs.iter().map(AsRef::as_ref))?,
            default_write_options: Some(Self::writeopts()),
        })
    }

    /// Executes all pending migration steps and returns the migrated database.
    pub fn migrate(mut self) -> Result<DB, DBOpenError> {
        let start_step = self.start_step as usize;

        if start_step > 0 {
            debug!("Skipping {start_step} already-recorded migration steps");
        }

        let steps = std::mem::take(&mut self.steps);

        for (step_no, step) in steps.into_iter().enumerate().skip(start_step) {
            let step_no = step_no as u32;
            info!("Running migration step {step_no}");

            // Execute step (modify DB)
            step.execute(&mut self)
                .map_err(|details| DBMigrationError { step_no, details })?;

            // Record step completion
            self.db
                .put::<MigrationRecordColumn>(&step_no, &MigrationRecord::default())?;
        }

        Ok(self.db)
    }
}

#[cfg(test)]
mod test;
