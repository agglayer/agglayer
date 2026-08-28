use std::{
    collections::BTreeMap,
    fs::read_dir,
    path::{Path, PathBuf},
    sync::Arc,
    time::Instant,
};

use agglayer_errors::ResultExt as _;
use agglayer_types::EpochNumber;
use rocksdb::backup::{
    BackupEngine as RocksBackupEngine, BackupEngineInfo as RocksBackupEngineInfo,
    BackupEngineOptions, RestoreOptions,
};
use serde::Serialize;
use tokio::sync;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

use crate::storage::DB;

/// Whether a backup of the state and pending databases first flushes the
/// memtables to disk.
///
/// This is deliberately off. A forced flush writes one small SST file per
/// column family that has taken a write since the previous backup, and those
/// files are never merged away: the settlement column families are keyed by
/// ULID, so each new file sorts above every file already on disk and RocksDB
/// moves it straight to the bottom level untouched. Backing up on every
/// settlement write therefore grew the live file count without bound, and the
/// backup engine stats every live file against the backup directory before it
/// copies anything, so backups got slower in proportion to how many had
/// already been taken.
///
/// With the flush off the write-ahead log is copied into the backup instead
/// and replayed on restore. Writes are already synced to that log
/// (`WriteOptions::set_sync`), so the recovery point is unchanged, and
/// [`crate::storage::MAX_TOTAL_WAL_SIZE`] bounds how much log a backup carries.
const FLUSH_BEFORE_BACKUP: bool = false;

/// Number of files the backup engine copies in parallel.
///
/// RocksDB defaults to one, serialising every file copy. That is felt most on
/// restore, which copies every file of the database back out of the backup
/// directory, typically a network filesystem.
const BACKUP_PARALLELISM: i32 = 16;

/// Build the backup engine options used for every backup directory.
fn backup_engine_options(path: impl AsRef<Path>) -> Result<BackupEngineOptions, rocksdb::Error> {
    let mut options = BackupEngineOptions::new(path)?;
    options.set_max_background_operations(BACKUP_PARALLELISM);

    Ok(options)
}

/// Run one step of a backup, reporting a failure and counting it.
fn step(failures: &mut usize, result: Result<(), rocksdb::Error>, message: &str) {
    if result.log_err(message).is_err() {
        *failures += 1;
    }
}

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

/// Work for the backup engine.
pub(crate) enum BackupRequest {
    /// Back up the state and pending databases.
    State,

    /// Back up one packed epoch database.
    ///
    /// The open handle travels with the request on purpose. Reopening the
    /// epoch database read-only to back it up later looks like it would work —
    /// the backup even reports success — but a read-only handle does not
    /// expose its write-ahead log to the backup engine, so the resulting
    /// backup silently omits everything not already flushed.
    Epoch {
        db: Arc<DB>,
        epoch_number: EpochNumber,
    },
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
///
/// The queue is unbounded and every send is infallible. The bounded queue this
/// replaces dropped whatever did not fit, which cost the epoch backups that
/// are only ever offered once.
#[derive(Clone)]
pub struct BackupClient {
    sender: Option<sync::mpsc::UnboundedSender<BackupRequest>>,
}

impl BackupClient {
    /// Create a new backup client that do nothing.
    /// This is useful for tests or when the backup is disabled.
    pub fn noop() -> BackupClient {
        BackupClient { sender: None }
    }

    /// Create a backup client whose requests are recorded instead of being
    /// executed, so tests can assert on the triggers.
    #[cfg(test)]
    pub(crate) fn observable() -> (BackupClient, sync::mpsc::UnboundedReceiver<BackupRequest>) {
        let (sender, receiver) = sync::mpsc::unbounded_channel();

        (
            BackupClient {
                sender: Some(sender),
            },
            receiver,
        )
    }

    /// Ask for a backup of the state and pending databases.
    ///
    /// Best effort by design and infallible. The receiver lives as long as the
    /// engine, and dropping the engine cancels the node, so a send only fails
    /// when shutdown is already under way.
    pub fn backup_state(&self) {
        if let Some(sender) = &self.sender {
            if sender.send(BackupRequest::State).is_err() {
                error!("Backup engine is gone, state databases will not be backed up");
            }
        }
    }

    /// Ask for a backup of a packed epoch database.
    pub fn backup_epoch(&self, db: Arc<DB>, epoch_number: EpochNumber) {
        if let Some(sender) = &self.sender {
            if sender
                .send(BackupRequest::Epoch { db, epoch_number })
                .is_err()
            {
                error!(%epoch_number, "Backup engine is gone, epoch database will not be backed up");
            }
        }
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
    requests: sync::mpsc::UnboundedReceiver<BackupRequest>,
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
        let pending_opts = backup_engine_options(&config.pending_backup_path)?;
        let state_opts = backup_engine_options(&config.state_backup_path)?;

        let (sender, requests) = sync::mpsc::unbounded_channel();

        std::fs::create_dir_all(&config.epochs_backup_path)?;

        Ok((
            Self {
                state_engine: RocksBackupEngine::open(&state_opts, &env)?,
                pending_engine: RocksBackupEngine::open(&pending_opts, &env)?,
                config,
                env,
                state_db,
                pending_db,
                requests,
                state_max_backup_number,
                pending_max_backup_number,
                cancellation_token,
            },
            BackupClient {
                sender: Some(sender),
            },
        ))
    }

    /// Run the backup engine, listen for new backup requests.
    pub async fn run(mut self) -> eyre::Result<()> {
        loop {
            let request = tokio::select! {
                _ = self.cancellation_token.cancelled() => break,
                request = self.requests.recv() => match request {
                    Some(request) => request,
                    None => break,
                }
            };

            let batch = self.take_queued(vec![request]);
            self = self.run_blocking(batch).await?;
        }

        info!("Backup engine stopping, finishing outstanding work");

        // Cancellation must not abandon what is queued: an epoch is only ever
        // offered once, and the writes made just before shutdown are the ones
        // least likely to be covered by a backup already. The trailing request
        // is unconditional so the last writes are always captured.
        let mut batch = self.take_queued(Vec::new());
        batch.push(BackupRequest::State);
        self.run_blocking(batch).await?;

        Ok(())
    }

    /// Add everything already queued to `batch`.
    fn take_queued(&mut self, mut batch: Vec<BackupRequest>) -> Vec<BackupRequest> {
        while let Ok(request) = self.requests.try_recv() {
            batch.push(request);
        }

        batch
    }

    /// Carry out a batch off the async runtime.
    ///
    /// Creating a backup blocks on file I/O for as long as it takes to walk the
    /// backup directory, so it must not run on an async worker.
    async fn run_blocking(self, batch: Vec<BackupRequest>) -> eyre::Result<Self> {
        Ok(tokio::task::spawn_blocking(move || {
            let mut engine = self;
            engine.run_batch(batch);

            engine
        })
        .await?)
    }

    /// Carry out one batch of requests.
    ///
    /// State backups are interchangeable, so however many the batch holds they
    /// collapse into a single run. Epoch backups are not, so each is carried
    /// out.
    fn run_batch(&mut self, batch: Vec<BackupRequest>) {
        let mut state_requested = false;

        for request in batch {
            match request {
                BackupRequest::Epoch { db, epoch_number } => self.backup_epoch(&db, epoch_number),
                BackupRequest::State => state_requested = true,
            }
        }

        if state_requested {
            self.backup_state_and_pending();
        }
    }

    /// Back up the state and pending databases and purge backups beyond the
    /// configured retention.
    ///
    /// Every step runs even when an earlier one fails, but the run is only
    /// reported as successful when all of them succeeded. Reporting success
    /// unconditionally, as this used to, made the count of successful backups
    /// indistinguishable from the count of attempts.
    fn backup_state_and_pending(&mut self) {
        #[cfg(test)]
        test_hooks::backup_started();

        let started = Instant::now();
        info!("Creating new backup of the state and pending databases");

        let mut failures = 0;

        step(
            &mut failures,
            self.state_engine
                .create_new_backup_flush(self.state_db.raw_rocksdb(), FLUSH_BEFORE_BACKUP),
            "Failed to create backup for state db",
        );
        step(
            &mut failures,
            self.state_engine
                .purge_old_backups(self.state_max_backup_number),
            "Failed to purge old backup for state db",
        );
        step(
            &mut failures,
            self.pending_engine
                .create_new_backup_flush(self.pending_db.raw_rocksdb(), FLUSH_BEFORE_BACKUP),
            "Failed to create backup for pending db",
        );
        step(
            &mut failures,
            self.pending_engine
                .purge_old_backups(self.pending_max_backup_number),
            "Failed to purge old backup for pending db",
        );

        let elapsed_ms = started.elapsed().as_millis();
        if failures == 0 {
            info!(elapsed_ms, "Backup successfully created");
        } else {
            error!(elapsed_ms, failures, "Backup completed with failures");
        }
    }

    /// Back up one packed epoch database.
    ///
    /// The flush is kept on here, unlike the state and pending databases: an
    /// epoch database is packed, small, and closed right after, so the flush
    /// costs a single file and leaves the backup with no write-ahead log to
    /// replay.
    fn backup_epoch(&mut self, db: &DB, epoch_number: EpochNumber) {
        let started = Instant::now();

        let backup = backup_engine_options(
            self.config
                .epochs_backup_path
                .join(format!("{epoch_number}")),
        )
        .and_then(|options| RocksBackupEngine::open(&options, &self.env))
        .and_then(|mut engine| engine.create_new_backup_flush(db.raw_rocksdb(), true));

        match backup {
            Ok(()) => {
                let elapsed_ms = started.elapsed().as_millis();
                info!(%epoch_number, elapsed_ms, "Epoch database backup created");
            }
            Err(error) => error!(
                %epoch_number, %error,
                "Failed to back up epoch database; this epoch has no backup"
            ),
        }
    }

    /// Restore the state database from the latest backup.
    pub fn restore(path: &Path, db_path: &Path) -> eyre::Result<()> {
        let env = rocksdb::Env::new()?;
        let opts = backup_engine_options(path)?;

        let mut engine = RocksBackupEngine::open(&opts, &env)?;

        std::fs::create_dir_all(db_path)?;

        Ok(engine.restore_from_latest_backup(db_path, db_path, &RestoreOptions::default())?)
    }

    /// Restore the state database from the defined backup version.
    pub fn restore_at(path: &Path, db_path: &Path, version: u32) -> eyre::Result<()> {
        let env = rocksdb::Env::new()?;
        let opts = backup_engine_options(path)?;

        let mut engine = RocksBackupEngine::open(&opts, &env)?;

        std::fs::create_dir_all(db_path)?;

        Ok(engine.restore_from_backup(db_path, db_path, &RestoreOptions::default(), version)?)
    }

    pub fn list_backups(path: &Path) -> eyre::Result<BackupReport> {
        let env = rocksdb::Env::new()?;

        let config: BackupEngineConfig = path.into();
        let opts = backup_engine_options(&config.state_backup_path)?;
        let engine = RocksBackupEngine::open(&opts, &env)?;

        let state = engine
            .get_backup_info()
            .into_iter()
            .map(BackupEngineInfo::from);

        let opts = backup_engine_options(&config.pending_backup_path)?;
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
                let opts = backup_engine_options(path)?;
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
mod tests;
