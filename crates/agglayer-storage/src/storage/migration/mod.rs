use std::{collections::BTreeSet, path::Path};

use rocksdb::ColumnFamilyDescriptor;
use tracing::{debug, info};

pub use self::error::{DBMigrationError, DBMigrationErrorDetails, DBOpenError};
use self::{migration_cf::MigrationRecordColumn, record::MigrationRecord, step::MigrationStep};
use crate::{
    schema::{ColumnDescriptor, ColumnSchema},
    storage::{DBError, DB},
};

mod error;
mod migration_cf;
mod record;
mod step;

/// Database builder taking care of database migrations.
pub struct Builder {
    // The database itself.
    db: DB,

    // The step number where migration should start.
    start_step: u32,

    // Collected migration steps to be executed in finalize().
    steps: Vec<MigrationStep>,
}

impl Builder {
    fn new_internal(db: DB, start_step: u32) -> Self {
        Self {
            db,
            start_step,
            steps: Vec::new(),
        }
    }

    /// Opens a database at the given path with migration tracking.
    ///
    /// This method initializes or opens an existing database with the provided
    /// initial schema (v0). It automatically sets up migration tracking and
    /// validates the database schema.
    pub fn open(path: &Path, cfs_v0: &[ColumnDescriptor]) -> Result<Self, DBOpenError> {
        debug!("Preparing database for initialization and migration");
        let desc_v0: Vec<_> = cfs_v0.iter().map(|cd| cd.to_rocksdb_descriptor()).collect();
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

        // Add step 0 (Initialize) to the steps list, but don't execute yet.
        Ok(Self::new_internal(db, start_step).add_step(MigrationStep::Initialize))
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

    /// Creates new column families and populates them with data.
    ///
    /// This is a migration step that creates column families and runs the
    /// provided migration function to populate them. The migration function
    /// should only write into the newly created column families.
    pub fn add_cfs(
        self,
        cfs: &[ColumnDescriptor],
        migrate_fn: impl FnOnce(&mut DB) -> Result<(), DBMigrationErrorDetails> + 'static,
    ) -> Result<Self, DBOpenError> {
        Ok(self.add_step(MigrationStep::AddColumnFamilies {
            cfs: cfs.to_vec(),
            migrate_fn: Box::new(migrate_fn),
        }))
    }

    /// Removes old column families from the database.
    pub fn drop_cfs(self, cfs: &[ColumnDescriptor]) -> Result<Self, DBOpenError> {
        Ok(self.add_step(MigrationStep::DropColumnFamilies { cfs: cfs.to_vec() }))
    }

    /// Completes the migration process and returns the database.
    ///
    /// This method validates migration steps, executes all collected steps,
    /// and returns the fully migrated database ready for use.
    pub fn finalize(mut self, _expected_schema: &[ColumnDescriptor]) -> Result<DB, DBOpenError> {
        // Validate that we have at least as many declared steps as recorded steps
        let declared_steps = self.steps.len() as u32;
        if declared_steps < self.start_step {
            return Err(DBOpenError::FewerStepsDeclared {
                declared: declared_steps,
                recorded: self.start_step,
            });
        }

        // Execute all collected steps
        let start_step = self.start_step as usize;

        if start_step > 0 {
            debug!("Skipping {start_step} already-recorded migration steps");
        }

        // Take ownership of steps to iterate through them
        let steps = std::mem::take(&mut self.steps);

        // Execute steps that need to run
        for (step_no, step) in steps.into_iter().enumerate().skip(start_step) {
            let step_no = step_no as u32;

            let span = tracing::debug_span!("Storage migration step", %step_no);
            let _span_guard = span.enter();

            // Execute the step
            info!("Running storage migration step {step_no}");
            step.execute(&mut self)
                .map_err(|details| DBMigrationError { step_no, details })?;

            // Record the step in migration record
            debug!("Recording the completion of migration step {step_no}");
            self.db
                .put::<MigrationRecordColumn>(&step_no, &MigrationRecord::default())?;
        }

        Ok(self.db)
    }

    fn cf_exists(&self, cf: &str) -> bool {
        self.db.rocksdb.cf_handle(cf).is_some()
    }

    /// Helper method to add a migration step to the list.
    fn add_step(mut self, migration_step: MigrationStep) -> Self {
        self.steps.push(migration_step);
        self
    }
}

impl std::fmt::Debug for Builder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Builder")
            .field("start_step", &self.start_step)
            .field("steps", &self.steps)
            .finish()
    }
}

#[cfg(test)]
mod test;
