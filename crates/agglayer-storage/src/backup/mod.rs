use std::{
    collections::BTreeMap,
    fs::read_dir,
    path::{Path, PathBuf},
    sync::Arc,
};

use agglayer_errors::ResultExt as _;
use agglayer_types::EpochNumber;
use eyre::eyre;
use rocksdb::backup::{
    BackupEngine as RocksBackupEngine, BackupEngineInfo as RocksBackupEngineInfo,
    BackupEngineOptions, RestoreOptions,
};
use serde::Serialize;
use tokio::sync;
use tokio_util::sync::CancellationToken;
use tracing::info;

use crate::storage::DB;

#[cfg(test)]
mod test_hooks {
    use std::{sync::Mutex, thread, time::Duration};

    static BACKUP_STARTED: Mutex<Option<std::sync::mpsc::Sender<String>>> = Mutex::new(None);

    pub(super) fn observe_backup_started(sender: std::sync::mpsc::Sender<String>) {
        *BACKUP_STARTED.lock().expect("backup hook lock poisoned") = Some(sender);
    }

    pub(super) fn backup_started() {
        let Some(sender) = BACKUP_STARTED
            .lock()
            .expect("backup hook lock poisoned")
            .take()
        else {
            return;
        };

        let thread_name = thread::current().name().unwrap_or("unnamed").to_string();
        sender
            .send(thread_name)
            .expect("backup hook receiver should be alive");

        thread::sleep(Duration::from_millis(200));
    }
}

/// Request to create a new backup.
pub struct BackupRequest {
    /// Optional epoch db to backup.
    pub epoch_db: Option<(Arc<DB>, EpochNumber)>,
}

struct BackupEngineConfig {
    state_backup_path: PathBuf,
    pending_backup_path: PathBuf,
    epochs_backup_path: PathBuf,
}

impl BackupEngineConfig {
    const DEFAULT_EPOCHS_DIR: &'static str = "epochs";
    const DEFAULT_PENDING_DIR: &'static str = "pending";
    const DEFAULT_STATE_DIR: &'static str = "state";
}

impl From<&Path> for BackupEngineConfig {
    fn from(path: &Path) -> Self {
        Self {
            state_backup_path: path.join(Self::DEFAULT_STATE_DIR),
            pending_backup_path: path.join(Self::DEFAULT_PENDING_DIR),
            epochs_backup_path: path.join(Self::DEFAULT_EPOCHS_DIR),
        }
    }
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
    pub fn backup(&self, request: BackupRequest) -> eyre::Result<()> {
        if let Some(sender) = &self.sender {
            sender
                .try_send(request)
                .map_err(|_| eyre!("Unable to send backup request"))?;
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
    config: BackupEngineConfig,
    backup_request: sync::mpsc::Receiver<BackupRequest>,
    state_max_backup_number: usize,
    pending_max_backup_number: usize,
    cancellation_token: CancellationToken,
}

impl BackupEngine {
    /// Create a new backup engine, return the engine and a client to request
    /// backups.
    pub fn new(
        path: &Path,
        state_db: Arc<DB>,
        pending_db: Arc<DB>,
        state_max_backup_number: usize,
        pending_max_backup_number: usize,
        cancellation_token: CancellationToken,
    ) -> eyre::Result<(Self, BackupClient)> {
        let env = rocksdb::Env::new()?;
        let config: BackupEngineConfig = path.into();
        let pending_opts = rocksdb::backup::BackupEngineOptions::new(&config.pending_backup_path)?;
        let state_opts = rocksdb::backup::BackupEngineOptions::new(&config.state_backup_path)?;

        let (sender, backup_request) = sync::mpsc::channel(10);

        std::fs::create_dir_all(&config.epochs_backup_path)?;

        Ok((
            Self {
                state_engine: RocksBackupEngine::open(&state_opts, &env)?,
                pending_engine: RocksBackupEngine::open(&pending_opts, &env)?,
                config,
                env,
                state_db,
                pending_db,
                backup_request,
                state_max_backup_number,
                pending_max_backup_number,
                cancellation_token,
            },
            BackupClient {
                sender: Some(sender),
            },
        ))
    }

    /// Create a new backup for the state, pending and epochs databases.
    /// This function will also purge old backups as configured.
    pub fn create_new_backup(&mut self, request: &BackupRequest) -> eyre::Result<()> {
        #[cfg(test)]
        test_hooks::backup_started();

        info!("Creating new backup");

        if let Some((db, epoch_number)) = request.epoch_db.as_ref() {
            let epochs_opts = rocksdb::backup::BackupEngineOptions::new(
                self.config
                    .epochs_backup_path
                    .join(format!("{epoch_number}")),
            )?;

            if let Ok(mut engine) = RocksBackupEngine::open(&epochs_opts, &self.env)
                .log_err("Failed to open backup engine for epoch db")
            {
                let _ = engine
                    .create_new_backup_flush(db.raw_rocksdb(), true)
                    .log_err("Failed to create backup for epoch db");
            }
        } else {
            let _ = self
                .state_engine
                .create_new_backup_flush(self.state_db.raw_rocksdb(), true)
                .log_err("Failed to create backup for state db");

            let _ = self
                .state_engine
                .purge_old_backups(self.state_max_backup_number)
                .log_err("Failed to purge old backup for state db");

            let _ = self
                .pending_engine
                .create_new_backup_flush(self.pending_db.raw_rocksdb(), true)
                .log_err("Failed to create backup for pending db");

            let _ = self
                .pending_engine
                .purge_old_backups(self.pending_max_backup_number)
                .log_err("Failed to purge old backup for pending db");
        }

        info!("Backup successfully created");

        Ok(())
    }

    /// Run the backup engine, listen for new backup requests.
    pub async fn run(mut self) -> eyre::Result<()> {
        loop {
            tokio::select! {
                _ = self.cancellation_token.cancelled() => {
                    info!("Backup engine cancelled");
                    break;
                }
                Some(request) = self.backup_request.recv() =>{
                    let (backup_engine, result) = tokio::task::spawn_blocking(move || {
                        let mut backup_engine = self;
                        let result = backup_engine.create_new_backup(&request);

                        (backup_engine, result)
                    })
                    .await?;

                    self = backup_engine;
                    result?;
                }
            }
        }

        Ok(())
    }

    /// Restore the state database from the latest backup.
    pub fn restore(path: &Path, db_path: &Path) -> eyre::Result<()> {
        let env = rocksdb::Env::new()?;
        let opts = rocksdb::backup::BackupEngineOptions::new(path)?;

        let mut engine = RocksBackupEngine::open(&opts, &env)?;

        std::fs::create_dir_all(db_path)?;

        Ok(engine.restore_from_latest_backup(db_path, db_path, &RestoreOptions::default())?)
    }

    /// Restore the state database from the defined backup version.
    pub fn restore_at(path: &Path, db_path: &Path, version: u32) -> eyre::Result<()> {
        let env = rocksdb::Env::new()?;
        let opts = rocksdb::backup::BackupEngineOptions::new(path)?;

        let mut engine = RocksBackupEngine::open(&opts, &env)?;

        std::fs::create_dir_all(db_path)?;

        Ok(engine.restore_from_backup(db_path, db_path, &RestoreOptions::default(), version)?)
    }

    pub fn list_backups(path: &Path) -> eyre::Result<BackupReport> {
        let env = rocksdb::Env::new()?;

        let config: BackupEngineConfig = path.into();
        let opts = BackupEngineOptions::new(&config.state_backup_path)?;
        let engine = RocksBackupEngine::open(&opts, &env)?;

        let state = engine
            .get_backup_info()
            .into_iter()
            .map(BackupEngineInfo::from);

        let opts = BackupEngineOptions::new(&config.pending_backup_path)?;
        let engine = RocksBackupEngine::open(&opts, &env)?;

        let pending = engine
            .get_backup_info()
            .into_iter()
            .map(BackupEngineInfo::from);

        let mut epochs = (read_dir(&config.epochs_backup_path)?)
            .flatten()
            .filter_map(|d| {
                d.file_name()
                    .to_string_lossy()
                    .parse::<u64>()
                    .map(|r| (r, d.path()))
                    .ok()
            })
            .collect::<Vec<_>>();

        // We need to resort the epochs since the directory listing is not correctly
        // ordered.
        epochs.sort();

        let epochs = epochs
            .into_iter()
            .map(|(epoch_number, path)| -> eyre::Result<_> {
                let opts = BackupEngineOptions::new(path)?;
                let engine = RocksBackupEngine::open(&opts, &env)?;

                Ok((
                    epoch_number,
                    engine
                        .get_backup_info()
                        .into_iter()
                        .map(BackupEngineInfo::from)
                        .collect::<Vec<_>>(),
                ))
            })
            .collect::<eyre::Result<Vec<_>>>()?;

        Ok(BackupReport::new(state, pending, epochs))
    }
}

impl Drop for BackupEngine {
    fn drop(&mut self) {
        info!("Waiting for all requested backups to complete");

        self.env.set_background_threads(0);
        self.env.set_low_priority_background_threads(0);
        self.env.set_high_priority_background_threads(0);
        self.env.set_bottom_priority_background_threads(0);

        self.env.join_all_threads();
        self.cancellation_token.cancel();
    }
}

#[derive(Serialize)]
pub struct BackupEngineInfo {
    pub backup_id: u32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub size: u64,
    pub num_files: u32,
}

impl From<RocksBackupEngineInfo> for BackupEngineInfo {
    fn from(info: RocksBackupEngineInfo) -> Self {
        Self {
            backup_id: info.backup_id,
            // We use the default timestamp if the conversion fails as this timestamp is purely
            // informative.
            timestamp: chrono::DateTime::<chrono::Utc>::from_timestamp(info.timestamp, 0)
                .unwrap_or_default(),
            size: info.size,
            num_files: info.num_files,
        }
    }
}

#[derive(Default, Serialize)]
pub struct BackupReport {
    epochs: BTreeMap<u64, Vec<BackupEngineInfo>>,
    state: Vec<BackupEngineInfo>,
    pending: Vec<BackupEngineInfo>,
}

impl BackupReport {
    pub fn new(
        state: impl Iterator<Item = BackupEngineInfo>,
        pending: impl Iterator<Item = BackupEngineInfo>,
        epochs: impl IntoIterator<Item = (u64, Vec<BackupEngineInfo>)>,
    ) -> Self {
        Self {
            state: state.collect(),
            pending: pending.collect(),
            epochs: BTreeMap::from_iter(epochs),
        }
    }
}

impl BackupReport {
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

#[cfg(test)]
mod tests {
    use std::{
        sync::Arc,
        time::{Duration, Instant},
    };

    use tokio_util::sync::CancellationToken;

    use super::*;
    use crate::{
        stores::{pending::PendingStore, state::StateStore},
        tests::TempDBDir,
    };

    #[tokio::test(flavor = "current_thread")]
    async fn backup_creation_does_not_block_the_async_runtime_worker() {
        let tmp = TempDBDir::new();
        let state_db = Arc::new(
            StateStore::init_db(&tmp.path.join("state")).expect("state db should initialize"),
        );
        let pending_db = Arc::new(
            PendingStore::init_db(&tmp.path.join("pending")).expect("pending db should initialize"),
        );
        let cancellation_token = CancellationToken::new();
        let (backup_engine, backup_client) = BackupEngine::new(
            &tmp.path.join("backup"),
            state_db,
            pending_db,
            10,
            10,
            cancellation_token.clone(),
        )
        .expect("backup engine should initialize");

        let (started_sender, started_receiver) = std::sync::mpsc::channel();
        test_hooks::observe_backup_started(started_sender);

        let backup_handle = tokio::spawn(backup_engine.run());
        let started_at = Instant::now();
        backup_client
            .backup(BackupRequest { epoch_db: None })
            .expect("backup request should be queued");

        let _backup_thread = tokio::task::spawn_blocking(move || {
            started_receiver.recv_timeout(Duration::from_secs(1))
        })
        .await
        .expect("backup started receiver task should complete")
        .expect("backup should start");

        assert!(
            started_at.elapsed() < Duration::from_millis(100),
            "backup creation ran on the async runtime worker and delayed unrelated async tasks"
        );

        cancellation_token.cancel();
        backup_handle.abort();
    }
}
