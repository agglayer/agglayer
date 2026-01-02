use std::{collections::BTreeSet, path::Path};

use rocksdb::ColumnFamilyDescriptor;
use tracing::{debug, info, instrument, warn};

use crate::{
    schema::ColumnSchema,
    storage::{DBError, DB},
};

mod access;
mod error;
mod migration_cf;
mod record;

pub use access::DbAccess;
pub use error::{DBMigrationError, DBMigrationErrorDetails, DBOpenError};
use migration_cf::MigrationRecordColumn;
use record::MigrationRecord;

/// Database builder taking care of database migrations.
pub struct Builder {
    // The database itself.
    db: DB,

    // The number of the current/next migration step.
    step: u32,

    // The step number where migration should start.
    start_step: u32,
}

impl Builder {
    fn new_internal(db: DB, start_step: u32) -> Self {
        Self {
            db,
            step: 0,
            start_step,
        }
    }

    /// Opens a database at the given path with migration tracking.
    ///
    /// This method initializes or opens an existing database with the provided
    /// initial schema (v0). It automatically sets up migration tracking and
    /// validates the database schema.
    pub fn open(
        path: &Path,
        cfs_v0: impl IntoIterator<Item = ColumnFamilyDescriptor>,
    ) -> Result<Self, DBOpenError> {
        debug!("Preparing database for initialization and migration");
        let desc_v0: Vec<_> = cfs_v0.into_iter().collect();
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
            cfs.push(ColumnFamilyDescriptor::new(
                MigrationRecordColumn::COLUMN_FAMILY_NAME,
                rocksdb::Options::default(),
            ));
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

        Self::new_internal(db, start_step)
            // Initialize migration record CF with step 0.
            .perform_step(|_| Ok(()))
            .map_err(DBOpenError::Migration)
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
    pub fn add_cfs<'a>(
        self,
        cfs: impl IntoIterator<Item = &'a str>,
        migrate_fn: impl FnOnce(&DbAccess) -> Result<(), DBMigrationErrorDetails>,
    ) -> Result<Self, DBOpenError> {
        let cfs: BTreeSet<_> = cfs.into_iter().collect();

        Ok(self.perform_step(move |db| {
            // Create the columns first.
            for cf in &cfs {
                if db.cf_exists(cf) {
                    warn!("Column family {cf:?} already exists, dropping to create a fresh one");
                    db.db.rocksdb.drop_cf(cf).map_err(DBError::from)?;
                }

                debug!("Creating column family {cf:?}");
                db.db
                    .rocksdb
                    .create_cf(cf, &rocksdb::Options::default())
                    .map_err(DBError::from)?;
            }

            // Use the provided closure to populate it with data.
            debug!("Populating new column families with data");
            let access = DbAccess::new(&db.db, &cfs);
            migrate_fn(&access)?;

            Ok(())
        })?)
    }

    /// Removes old column families from the database.
    pub fn drop_cfs<S: AsRef<str>>(
        self,
        cfs: impl IntoIterator<Item = S>,
    ) -> Result<Self, DBOpenError> {
        Ok(self.perform_step(move |db| {
            for cf in cfs {
                let cf = cf.as_ref();

                if db.cf_exists(cf) {
                    debug!("Dropping column family {cf:?}");
                    db.db.rocksdb.drop_cf(cf).map_err(DBError::from)?;
                } else {
                    warn!("Asked to remove a non-existing column family {cf:?}");
                }
            }

            Ok(())
        })?)
    }

    /// Completes the migration process and returns the database.
    ///
    /// This method validates that all declared migration steps have been
    /// executed and returns the fully migrated database ready for use.
    pub fn finalize<'a>(
        self,
        _expected_schema: impl IntoIterator<Item = &'a str>,
    ) -> Result<DB, DBOpenError> {
        if self.step < self.start_step {
            return Err(DBOpenError::FewerStepsDeclared {
                declared: self.step,
                recorded: self.start_step,
            });
        }

        Ok(self.db)
    }

    fn perform_step(
        self,
        step_fn: impl FnOnce(&mut Self) -> Result<(), DBMigrationErrorDetails>,
    ) -> Result<Self, DBMigrationError> {
        let step = self.step;
        self.perform_step_impl(step_fn)
            .map_err(|details| DBMigrationError { step, details })
    }

    #[instrument(skip(self, step_fn), fields(step = self.step))]
    fn perform_step_impl(
        mut self,
        step_fn: impl FnOnce(&mut Self) -> Result<(), DBMigrationErrorDetails>,
    ) -> Result<Self, DBMigrationErrorDetails> {
        let step = self.step;

        if step >= self.start_step {
            info!("Running migration step {step}");
            step_fn(&mut self)?;
            self.db
                .put::<MigrationRecordColumn>(&self.step, &MigrationRecord::default())?;
        } else {
            debug!("Step already recorded, skipping");
        }

        self.step += 1;
        Ok(self)
    }

    fn cf_exists(&self, cf: &str) -> bool {
        self.db.rocksdb.cf_handle(cf).is_some()
    }
}

impl std::fmt::Debug for Builder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            db: _,
            step,
            start_step,
        } = self;

        f.debug_struct("Builder")
            .field("step", step)
            .field("start_step", start_step)
            .finish()
    }
}

#[cfg(test)]
mod test;
