use std::{
    collections::BTreeMap,
    fs::read_dir,
    path::{Path, PathBuf},
    sync::Arc,
};

use rocksdb::backup::{
    BackupEngine as RocksBackupEngine, BackupEngineInfo as RocksBackupEngineInfo,
    BackupEngineOptions, RestoreOptions,
};
use serde::{Deserialize, Serialize};
use tokio::sync;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

use super::{BackupError, DB};

/// Request to create a new backup.
pub struct BackupRequest {
    /// Optional epoch db to backup.
    pub epoch_db: Option<(Arc<DB>, u64)>,
}

/// Client used to request a backup.
#[derive(Clone)]
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

        let epochs_path = path.join("epochs");
        std::fs::create_dir_all(&epochs_path)?;

        Ok((
            Self {
                state_engine: RocksBackupEngine::open(&state_opts, &env)?,
                pending_engine: RocksBackupEngine::open(&pending_opts, &env)?,
                env,
                state_db,
                pending_db,
                epochs_path,
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
        } else {
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
        }

        info!("Backup successfully created");

        Ok(())
    }

    /// Run the backup engine, listen for new backup requests.
    pub async fn run(mut self, cancellation_token: CancellationToken) -> Result<(), BackupError> {
        loop {
            tokio::select! {
                _ = cancellation_token.cancelled() => {
                    info!("Backup engine cancelled");
                    break;
                }
                Some(request) = self.backup_request.recv() =>{
                    self.create_new_backup(&request)?;
                }
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

    /// Restore the state database from the defined backup version.
    pub fn restore_at(path: &Path, db_path: &Path, version: u32) -> Result<(), BackupError> {
        let env = rocksdb::Env::new()?;
        let opts = rocksdb::backup::BackupEngineOptions::new(path)?;

        let mut engine = RocksBackupEngine::open(&opts, &env)?;

        std::fs::create_dir_all(db_path).unwrap();

        Ok(engine.restore_from_backup(db_path, db_path, &RestoreOptions::default(), version)?)
    }

    pub fn list_backups(path: &Path) -> Result<BackupReport, BackupError> {
        let env = rocksdb::Env::new()?;

        let mut report = BackupReport::default();

        let opts = BackupEngineOptions::new(path.join("state"))?;
        let engine = RocksBackupEngine::open(&opts, &env)?;

        report.state(engine.get_backup_info());

        let opts = BackupEngineOptions::new(path.join("pending"))?;
        let engine = RocksBackupEngine::open(&opts, &env)?;

        report.pending(engine.get_backup_info());

        let epoch_path = path.join("epochs");
        let mut epochs = (read_dir(&epoch_path)?)
            .flatten()
            .filter_map(|d| {
                d.file_name()
                    .to_string_lossy()
                    .parse::<u64>()
                    .map(|r| (r, d.path()))
                    .ok()
            })
            .collect::<Vec<_>>();

        epochs.sort_by(|(p, _), (n, _)| p.cmp(n));

        for (epoch_number, path) in epochs {
            let opts = BackupEngineOptions::new(path)?;
            let engine = RocksBackupEngine::open(&opts, &env)?;
            report.epochs(epoch_number, engine.get_backup_info());
        }

        Ok(report)
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

#[derive(Default, Serialize, Deserialize)]
pub struct BackupReport {
    epochs: BTreeMap<u64, Vec<BackupEngineInfo>>,
    state: Vec<BackupEngineInfo>,
    pending: Vec<BackupEngineInfo>,
}

#[derive(Serialize, Deserialize)]
pub struct BackupEngineInfo {
    pub backup_id: u32,
    #[serde(serialize_with = "timestamp_to_readable")]
    pub timestamp: i64,
    pub size: u64,
    pub num_files: u32,
}

fn timestamp_to_readable<S>(timestamp: &i64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    chrono::DateTime::<chrono::Utc>::from_timestamp(*timestamp, 0)
        .unwrap_or_default()
        .to_rfc2822()
        .serialize(serializer)
}

impl From<RocksBackupEngineInfo> for BackupEngineInfo {
    fn from(info: RocksBackupEngineInfo) -> Self {
        Self {
            backup_id: info.backup_id,
            timestamp: info.timestamp,
            size: info.size,
            num_files: info.num_files,
        }
    }
}

impl BackupReport {
    pub fn state<V: IntoIterator<Item = T>, T: Into<BackupEngineInfo>>(&mut self, value: V) {
        self.state = value.into_iter().map(Into::into).collect();
    }

    pub fn pending<V: IntoIterator<Item = T>, T: Into<BackupEngineInfo>>(&mut self, value: V) {
        self.pending = value.into_iter().map(Into::into).collect();
    }

    pub fn epochs<V: IntoIterator<Item = T>, T: Into<BackupEngineInfo>>(
        &mut self,
        key: u64,
        value: V,
    ) {
        self.epochs
            .insert(key, value.into_iter().map(Into::into).collect());
    }

    pub fn get_state(&self) -> &[BackupEngineInfo] {
        self.state.as_slice()
    }

    pub fn get_pending(&self) -> &[BackupEngineInfo] {
        self.pending.as_slice()
    }

    pub fn get_epochs(&self) -> &BTreeMap<u64, Vec<BackupEngineInfo>> {
        &self.epochs
    }

    pub fn get_epoch(&self, epoch: u64) -> Option<&Vec<BackupEngineInfo>> {
        self.epochs.get(&epoch)
    }
}
