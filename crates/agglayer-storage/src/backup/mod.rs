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
use tokio::sync::{self, mpsc};
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

/// Request to back up one epoch database.
pub struct EpochBackupRequest {
    /// Epoch db to back up.
    pub db: Arc<DB>,
    /// Number of the epoch stored in `db`.
    pub epoch_number: EpochNumber,
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

/// Sending halves of the backup queues.
///
/// State and epoch requests travel on separate queues so that a burst of
/// state requests can never push an epoch request out of the queue.
#[derive(Clone)]
struct BackupSenders {
    /// Single-slot queue for state+pending backups. A backup snapshots the
    /// databases when it runs, not when it is requested, so one queued
    /// request covers every write made until it is dequeued and extra
    /// requests can be dropped safely.
    state: sync::mpsc::Sender<()>,
    /// Unbounded queue for epoch backups. Each epoch is packed exactly once,
    /// so a dropped request would mean that epoch is never backed up.
    epoch: sync::mpsc::UnboundedSender<EpochBackupRequest>,
}

/// Client used to request a backup.
#[derive(Clone)]
pub struct BackupClient {
    senders: Option<BackupSenders>,
}

impl BackupClient {
    /// Create a new backup client that do nothing.
    /// This is useful for tests or when the backup is disabled.
    pub fn noop() -> BackupClient {
        BackupClient { senders: None }
    }

    /// Create a backup client whose requests are collected instead of being
    /// executed, so tests can assert on the triggers.
    #[cfg(test)]
    pub(crate) fn observable() -> (BackupClient, ObservedBackupRequests) {
        // Unlike the production queue created by `BackupEngine::new`, the
        // state queue is large enough for tests to observe every trigger
        // instead of seeing them coalesced.
        let (state_sender, state) = sync::mpsc::channel(10);
        let (epoch_sender, epoch) = sync::mpsc::unbounded_channel();

        (
            BackupClient {
                senders: Some(BackupSenders {
                    state: state_sender,
                    epoch: epoch_sender,
                }),
            },
            ObservedBackupRequests { state, epoch },
        )
    }

    /// Request a backup of the state and pending databases.
    ///
    /// If a request is already queued, this one is coalesced into it: the
    /// queued backup will run after the write that triggered this request,
    /// so it covers this write too.
    pub fn backup_state(&self) -> eyre::Result<()> {
        if let Some(senders) = &self.senders {
            match senders.state.try_send(()) {
                Ok(()) | Err(mpsc::error::TrySendError::Full(())) => {}
                Err(mpsc::error::TrySendError::Closed(())) => {
                    Err(eyre!("Unable to send state backup request"))?
                }
            }
        }

        Ok(())
    }

    /// Request a backup of one epoch database.
    ///
    /// Epoch requests are queued unbounded and never dropped: each epoch is
    /// packed exactly once, so this is its only chance to get backed up.
    pub fn backup_epoch(&self, db: Arc<DB>, epoch_number: EpochNumber) -> eyre::Result<()> {
        if let Some(senders) = &self.senders {
            senders
                .epoch
                .send(EpochBackupRequest { db, epoch_number })
                .map_err(|_| eyre!("Unable to send epoch backup request"))?;
        }

        Ok(())
    }
}

/// Receiving halves of an observable [`BackupClient`], so tests can assert
/// on the requested backups.
#[cfg(test)]
pub(crate) struct ObservedBackupRequests {
    pub(crate) state: sync::mpsc::Receiver<()>,
    pub(crate) epoch: sync::mpsc::UnboundedReceiver<EpochBackupRequest>,
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
    state_backup_request: sync::mpsc::Receiver<()>,
    epoch_backup_request: sync::mpsc::UnboundedReceiver<EpochBackupRequest>,
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

        // State requests coalesce into a single queue slot while epoch
        // requests are queued unbounded; see [`BackupSenders`].
        let (state_sender, state_backup_request) = sync::mpsc::channel(1);
        let (epoch_sender, epoch_backup_request) = sync::mpsc::unbounded_channel();

        std::fs::create_dir_all(&config.epochs_backup_path)?;

        Ok((
            Self {
                state_engine: RocksBackupEngine::open(&state_opts, &env)?,
                pending_engine: RocksBackupEngine::open(&pending_opts, &env)?,
                config,
                env,
                state_db,
                pending_db,
                state_backup_request,
                epoch_backup_request,
                state_max_backup_number,
                pending_max_backup_number,
                cancellation_token,
            },
            BackupClient {
                senders: Some(BackupSenders {
                    state: state_sender,
                    epoch: epoch_sender,
                }),
            },
        ))
    }

    /// Create a new backup for the state and pending databases.
    /// This function will also purge old backups as configured.
    pub fn backup_state(&mut self) -> eyre::Result<()> {
        #[cfg(test)]
        test_hooks::backup_started();

        info!("Creating new state backup");

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

        info!("State backup successfully created");

        Ok(())
    }

    /// Create a new backup for one epoch database.
    pub fn backup_epoch(&self, request: &EpochBackupRequest) -> eyre::Result<()> {
        #[cfg(test)]
        test_hooks::backup_started();

        let EpochBackupRequest { db, epoch_number } = request;

        info!("Creating new backup for epoch {epoch_number}");

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

        info!("Epoch backup successfully created");

        Ok(())
    }

    /// Run the backup engine, listen for new backup requests.
    ///
    /// On cancellation, requests that are already queued are drained before
    /// exiting, so that no epoch backup is ever lost to a shutdown.
    pub async fn run(mut self) -> eyre::Result<()> {
        loop {
            tokio::select! {
                _ = self.cancellation_token.cancelled() => {
                    info!("Backup engine cancelled");
                    break;
                }
                Some(()) = self.state_backup_request.recv() => {
                    self = self.run_blocking(|engine| engine.backup_state()).await?;
                }
                Some(request) = self.epoch_backup_request.recv() => {
                    self = self.run_blocking(move |engine| engine.backup_epoch(&request)).await?;
                }
            }
        }

        self.drain().await
    }

    /// Run one backup on a blocking task, handing the engine back once done.
    async fn run_blocking(
        self,
        backup: impl FnOnce(&mut Self) -> eyre::Result<()> + Send + 'static,
    ) -> eyre::Result<Self> {
        let (backup_engine, result) = tokio::task::spawn_blocking(move || {
            let mut backup_engine = self;
            let result = backup(&mut backup_engine);

            (backup_engine, result)
        })
        .await?;

        result?;

        Ok(backup_engine)
    }

    /// Process every request still queued, rejecting new ones.
    ///
    /// State requests coalesce so at most one can be pending, but a burst of
    /// epoch requests (which must never be dropped) can leave a backlog.
    async fn drain(mut self) -> eyre::Result<()> {
        // Closing the queues rejects new requests while keeping the already
        // queued ones receivable.
        self.state_backup_request.close();
        self.epoch_backup_request.close();

        info!("Draining the queued backup requests");

        tokio::task::spawn_blocking(move || {
            if self.state_backup_request.try_recv().is_ok() {
                self.backup_state()?;
            }

            while let Ok(request) = self.epoch_backup_request.try_recv() {
                self.backup_epoch(&request)?;
            }

            Ok(())
        })
        .await?
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

        // We need to resort the epochs since the directory listing is not
        // correctly ordered.
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
            .backup_state()
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

    #[tokio::test]
    async fn state_backup_requests_coalesce_when_the_queue_is_full() {
        let tmp = TempDBDir::new();
        let state_db = Arc::new(
            StateStore::init_db(&tmp.path.join("state")).expect("state db should initialize"),
        );
        let pending_db = Arc::new(
            PendingStore::init_db(&tmp.path.join("pending")).expect("pending db should initialize"),
        );
        let (_backup_engine, backup_client) = BackupEngine::new(
            &tmp.path.join("backup"),
            state_db,
            pending_db,
            10,
            10,
            CancellationToken::new(),
        )
        .expect("backup engine should initialize");

        // The engine is not running, so the first request fills the single
        // queue slot and the following ones coalesce into it.
        for _ in 0..3 {
            backup_client
                .backup_state()
                .expect("a full state queue should coalesce the request, not fail");
        }
    }

    #[tokio::test]
    async fn queued_backup_requests_are_drained_on_shutdown() {
        let tmp = TempDBDir::new();
        let state_db = Arc::new(
            StateStore::init_db(&tmp.path.join("state")).expect("state db should initialize"),
        );
        let pending_db = Arc::new(
            PendingStore::init_db(&tmp.path.join("pending")).expect("pending db should initialize"),
        );
        let epoch_db = Arc::new(
            StateStore::init_db(&tmp.path.join("epoch")).expect("epoch db should initialize"),
        );
        let cancellation_token = CancellationToken::new();
        let backup_path = tmp.path.join("backup");
        let (backup_engine, backup_client) = BackupEngine::new(
            &backup_path,
            state_db,
            pending_db,
            10,
            10,
            cancellation_token.clone(),
        )
        .expect("backup engine should initialize");

        // Queue everything before the engine runs, then cancel right away:
        // the engine must drain the queues instead of dropping the requests.
        const EPOCHS: u64 = 12;

        backup_client
            .backup_state()
            .expect("state backup request should be queued");
        for epoch in 0..EPOCHS {
            backup_client
                .backup_epoch(epoch_db.clone(), EpochNumber::new(epoch))
                .expect("epoch backup requests should never be rejected");
        }
        cancellation_token.cancel();

        backup_engine
            .run()
            .await
            .expect("backup engine should drain the queues and exit cleanly");

        let report = BackupEngine::list_backups(&backup_path).expect("backups should be listable");
        assert_eq!(report.get_state().len(), 1, "state backup should be taken");
        assert_eq!(
            report.get_pending().len(),
            1,
            "pending backup should be taken"
        );
        assert_eq!(
            report.get_epochs().len(),
            EPOCHS as usize,
            "every queued epoch backup should be taken"
        );
        for (epoch, backups) in report.get_epochs() {
            assert_eq!(backups.len(), 1, "epoch {epoch} should have one backup");
        }
    }
}
