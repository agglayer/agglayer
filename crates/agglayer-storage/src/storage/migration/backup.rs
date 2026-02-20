use std::path::Path;

use agglayer_config::storage::backup::BackupConfig;
use tracing::info;

use super::{DBOpenError, Migrator};

/// When should a backup on startup be taken.

impl Migrator<'_> {
    /// Creates a versioned backup of the database before running migrations.
    ///
    /// Returns `Self` for method chaining. Call before `migrate()` to capture
    /// the pre-migration state as a recovery point.
    #[tracing::instrument(skip(self))]
    pub fn backup(self, backup_path: &Path) -> Result<Self, DBOpenError> {
        use rocksdb::backup::{BackupEngine, BackupEngineOptions};

        info!("Creating startup backup");

        // Open RocksDB backup engine (creates directory if needed)
        let env = rocksdb::Env::new().map_err(DBOpenError::Backup)?;
        let opts = BackupEngineOptions::new(backup_path).map_err(DBOpenError::Backup)?;
        let mut engine = BackupEngine::open(&opts, &env).map_err(DBOpenError::Backup)?;

        // Create new backup version with flush (ensures all data is written)
        engine
            .create_new_backup_flush(self.db.raw_rocksdb(), true)
            .map_err(DBOpenError::Backup)?;

        // Get info about the backup we just created
        let backup_info = engine
            .get_backup_info()
            .pop()
            .ok_or(DBOpenError::BackupInfoMissing)?;

        info!(
            backup_id = backup_info.backup_id,
            size = backup_info.size,
            "Startup backup created"
        );

        Ok(self)
    }

    /// Create a backup if migration is needed.
    pub fn backup_if_migration_needed(self, backup_path: &Path) -> Result<Self, DBOpenError> {
        if self.migration_needed() {
            self.backup(backup_path)
        } else {
            Ok(self)
        }
    }

    /// Backup according to mode settings.
    pub fn backup_with_config(
        self,
        config: &BackupConfig,
        subdir: impl AsRef<Path>,
    ) -> Result<Self, DBOpenError> {
        use agglayer_config::storage::backup::StartupBackupMode as BM;

        match config {
            BackupConfig::Disabled => Ok(self),
            BackupConfig::Enabled {
                path, on_startup, ..
            } => {
                let backup_path = path.join(subdir.as_ref());
                match on_startup {
                    BM::Never => Ok(self),
                    BM::IfMigrating => self.backup_if_migration_needed(&backup_path),
                    BM::Always => self.backup(&backup_path),
                }
            }
        }
    }

    /// Check if there are pending migration steps to execute.
    pub fn migration_needed(&self) -> bool {
        (self.start_step as usize) < self.steps.len()
    }
}
