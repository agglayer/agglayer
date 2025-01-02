use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use rocksdb::backup::{BackupEngine as RocksBackupEngine, RestoreOptions};
use tokio::sync;
use tracing::{error, info};

use super::{BackupError, DB};

/// Request to create a new backup.
pub struct BackupRequest {
    /// Optional epoch db to backup.
    pub epoch_db: Option<(Arc<DB>, u64)>,
}

/// Client used to request a backup.
pub struct BackupClient {
    sender: Option<sync::mpsc::Sender<BackupRequest>>,
}

impl BackupClient {
    /// Create a new backup client that do nothing.
    /// This is useful for tests or when the backup is disabled.
    pub fn noop() -> BackupClient {
        BackupClient { sender: None }
    }

    /// Send a backup request.
    ///
    /// This function will send the request to the backup engine.
    pub fn backup(&self, request: BackupRequest) -> Result<(), BackupError> {
        if let Some(sender) = &self.sender {
            let slot = sender
                .try_reserve()
                .map_err(|_| BackupError::UnableToSendBackupRequest)?;

            slot.send(request);
        }

        Ok(())
    }
}

/// Backup engine that creates backups for the state, pending and epochs
/// databases.
pub struct BackupEngine {
    env: rocksdb::Env,
    pending_engine: RocksBackupEngine,
    state_engine: RocksBackupEngine,
    state_db: Arc<DB>,
    pending_db: Arc<DB>,
    epochs_path: PathBuf,
    backup_request: sync::mpsc::Receiver<BackupRequest>,
    state_max_backup_number: usize,
    pending_max_backup_number: usize,
}

// # Safety
//
// RocksBackupEngine is a simple pointer wrapper, so it's safe to send to
// another thread since the underlying RocksDB backup engine is thread-safe.
unsafe impl Send for BackupEngine {}

impl BackupEngine {
    /// Create a new backup engine, return the engine and a client to request
    /// backups.
    pub fn new(
        path: &Path,
        state_db: Arc<DB>,
        pending_db: Arc<DB>,
        state_max_backup_number: usize,
        pending_max_backup_number: usize,
    ) -> Result<(Self, BackupClient), BackupError> {
        let env = rocksdb::Env::new()?;
        let pending_opts = rocksdb::backup::BackupEngineOptions::new(path.join("pending"))?;
        let state_opts = rocksdb::backup::BackupEngineOptions::new(path.join("state"))?;

        let (sender, backup_request) = sync::mpsc::channel(10);

        Ok((
            Self {
                state_engine: RocksBackupEngine::open(&state_opts, &env)?,
                pending_engine: RocksBackupEngine::open(&pending_opts, &env)?,
                env,
                state_db,
                pending_db,
                epochs_path: path.join("epochs"),
                backup_request,
                state_max_backup_number,
                pending_max_backup_number,
            },
            BackupClient {
                sender: Some(sender),
            },
        ))
    }

    /// Create a new backup for the state, pending and epochs databases.
    /// This function will also purge old backups as configured.
    pub fn create_new_backup(&mut self, request: &BackupRequest) -> Result<(), BackupError> {
        info!("Creating new backup");

        if let Err(error) = self
            .state_engine
            .create_new_backup_flush(&self.state_db.rocksdb, true)
        {
            error!("Failed to create backup for state db: {:?}", error);
        }

        if let Err(error) = self
            .state_engine
            .purge_old_backups(self.state_max_backup_number)
        {
            error!("Failed to purge old backup for state db: {:?}", error);
        }

        if let Err(error) = self
            .pending_engine
            .create_new_backup_flush(&self.pending_db.rocksdb, true)
        {
            error!("Failed to create backup for pending db: {:?}", error);
        }

        if let Err(error) = self
            .pending_engine
            .purge_old_backups(self.pending_max_backup_number)
        {
            error!("Failed to purge old backup for pending db: {:?}", error);
        }

        if let Some((db, epoch_number)) = request.epoch_db.as_ref() {
            let epochs_opts = rocksdb::backup::BackupEngineOptions::new(
                self.epochs_path.join(format!("{}", epoch_number)),
            )?;

            match RocksBackupEngine::open(&epochs_opts, &self.env) {
                Err(error) => {
                    error!("Failed to open backup engine for epoch db: {:?}", error);
                }
                Ok(mut engine) => {
                    if let Err(error) = engine.create_new_backup_flush(&db.rocksdb, true) {
                        error!("Failed to create backup for epoch db: {:?}", error);
                    }
                }
            }
        }

        info!("Backup successfully created");

        Ok(())
    }

    /// Run the backup engine, listen for new backup requests.
    pub async fn run(mut self) -> Result<(), BackupError> {
        loop {
            tokio::select! {
                Some(request) = self.backup_request.recv() =>{
                    self.create_new_backup(&request)?;
                }
                else => break
            }
        }

        Ok(())
    }

    /// Restore the state database from the latest backup.
    pub fn restore(path: &Path, db_path: &Path) -> Result<(), BackupError> {
        let env = rocksdb::Env::new()?;
        let opts = rocksdb::backup::BackupEngineOptions::new(path)?;

        let mut engine = RocksBackupEngine::open(&opts, &env)?;

        std::fs::create_dir_all(db_path).unwrap();

        Ok(engine.restore_from_latest_backup(db_path, db_path, &RestoreOptions::default())?)
    }
}

impl Drop for BackupEngine {
    fn drop(&mut self) {
        self.env.set_background_threads(0);
        self.env.set_low_priority_background_threads(0);
        self.env.set_high_priority_background_threads(0);
        self.env.set_bottom_priority_background_threads(0);

        self.env.join_all_threads();
    }
}
