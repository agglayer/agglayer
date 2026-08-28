use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};

use agglayer_types::SettlementJobId;
use tokio_util::sync::CancellationToken;

use super::*;
use crate::{
    columns::settlement_jobs::SettlementJobsColumn,
    storage::DB,
    stores::{pending::PendingStore, state::StateStore},
    tests::TempDBDir,
};

struct Harness {
    engine: BackupEngine,
    client: BackupClient,
    cancellation_token: CancellationToken,
    state_db: Arc<DB>,
    state_path: PathBuf,
    backup_path: PathBuf,
}

fn harness(tmp: &TempDBDir) -> Harness {
    let state_path = tmp.path.join("state");
    let backup_path = tmp.path.join("backup");
    let state_db = Arc::new(StateStore::init_db(&state_path).expect("state db"));
    let pending_db =
        Arc::new(PendingStore::init_db(&tmp.path.join("pending")).expect("pending db"));
    let cancellation_token = CancellationToken::new();

    let (engine, client) = BackupEngine::new(
        &backup_path,
        state_db.clone(),
        pending_db,
        100,
        100,
        cancellation_token.clone(),
    )
    .expect("backup engine");

    Harness {
        engine,
        client,
        cancellation_token,
        state_db,
        state_path,
        backup_path,
    }
}

fn live_sst_count(db_path: &Path) -> usize {
    read_dir(db_path)
        .expect("read db dir")
        .flatten()
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "sst"))
        .count()
}

/// Write one settlement job row. Job ids are ULIDs, so successive rows sort
/// above every row already on disk — the write pattern that makes flushed
/// files pile up untouched at the bottom level.
fn write_settlement_job(db: &DB, id: u128) {
    db.put::<SettlementJobsColumn>(
        &SettlementJobId::from(id),
        &crate::types::generated::agglayer::storage::v0::SettlementJob::default(),
    )
    .expect("write settlement job");
}

async fn until<F: FnMut() -> bool>(label: &str, mut condition: F) {
    let deadline = Instant::now() + Duration::from_secs(20);
    while !condition() {
        assert!(Instant::now() < deadline, "timed out waiting for {label}");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

#[tokio::test(flavor = "current_thread")]
async fn backup_creation_does_not_block_the_async_runtime_worker() {
    let tmp = TempDBDir::new();
    let Harness {
        engine,
        client,
        cancellation_token,
        ..
    } = harness(&tmp);

    let (started_sender, started_receiver) = std::sync::mpsc::channel();
    test_hooks::observe_backup_started(started_sender);

    // A current-thread runtime drives the async work on this very thread, so
    // whichever thread the backup reports having run on tells us whether it
    // stayed off the runtime. Comparing threads rather than timing the backup
    // keeps this from turning into a flake under load.
    let runtime_thread = std::thread::current()
        .name()
        .unwrap_or("unnamed")
        .to_string();

    let backup_handle = tokio::spawn(engine.run());
    client.backup_state();

    let backup_thread =
        tokio::task::spawn_blocking(move || started_receiver.recv_timeout(Duration::from_secs(5)))
            .await
            .expect("backup started receiver task should complete")
            .expect("backup should start");

    assert_ne!(
        backup_thread, runtime_thread,
        "backup creation ran on the async runtime thread, so it would stall unrelated async work"
    );

    cancellation_token.cancel();
    backup_handle.abort();
}

/// Backing up repeatedly must not grow the live file count of the database.
///
/// Forcing a memtable flush per backup used to leave one small SST file behind
/// every time. Those files are never merged away, and the backup engine stats
/// every live file on each run, so the cost of a backup grew with the number
/// of backups already taken.
#[test]
fn repeated_backups_do_not_accumulate_sst_files() {
    let tmp = TempDBDir::new();
    let mut harness = harness(&tmp);

    // One write per backup, as a settling certificate would produce.
    for id in 0..25 {
        write_settlement_job(&harness.state_db, id);
        harness.engine.backup_state_and_pending();
    }

    let ssts = live_sst_count(&harness.state_path);
    assert!(
        ssts <= 2,
        "25 backups over 25 writes left {ssts} live SST files; backups are forcing a flush again \
         and the file count will grow without bound"
    );
}

/// Everything written before a backup must survive a restore from it, even
/// though the backup no longer forces a flush and so leans on the write-ahead
/// log to carry the most recent writes.
#[test]
fn backups_restore_every_write() {
    let tmp = TempDBDir::new();
    let mut harness = harness(&tmp);

    for id in 0..50 {
        write_settlement_job(&harness.state_db, id);
    }
    harness.engine.backup_state_and_pending();

    let restore_path = tmp.path.join("restored");
    BackupEngine::restore(&harness.backup_path.join("state"), &restore_path).expect("restore");

    let restored = StateStore::init_db(&restore_path).expect("open restored db");
    let restored_rows = (0..50)
        .filter(|id| {
            restored
                .get::<SettlementJobsColumn>(&SettlementJobId::from(*id))
                .expect("read restored row")
                .is_some()
        })
        .count();

    assert_eq!(restored_rows, 50, "restore lost rows held in the WAL");
}

/// State backup requests queued together collapse into one backup instead of
/// one backup each.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn queued_state_requests_collapse_into_one_backup() {
    let tmp = TempDBDir::new();
    let Harness {
        engine,
        client,
        cancellation_token,
        backup_path,
        ..
    } = harness(&tmp);

    // Raised before the engine polls, so they are all taken as one batch.
    for _ in 0..20 {
        client.backup_state();
    }

    let handle = tokio::spawn(engine.run());

    until("the batched backup to land", || {
        !BackupEngine::list_backups(&backup_path)
            .expect("list backups")
            .get_state()
            .is_empty()
    })
    .await;

    let taken = BackupEngine::list_backups(&backup_path)
        .expect("list backups")
        .get_state()
        .len();
    assert_eq!(taken, 1, "20 requests produced {taken} backups, expected 1");

    cancellation_token.cancel();
    handle.abort();
}

/// Writes made just before the engine is cancelled must still be backed up.
/// They are the ones least likely to be covered by a backup already.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn shutdown_finishes_outstanding_work() {
    let tmp = TempDBDir::new();
    let Harness {
        engine,
        client,
        cancellation_token,
        state_db,
        backup_path,
        ..
    } = harness(&tmp);

    let handle = tokio::spawn(engine.run());

    write_settlement_job(&state_db, 1);
    client.backup_state();

    // Cancel straight away: the request above may not have been picked up.
    cancellation_token.cancel();
    handle
        .await
        .expect("engine task should finish")
        .expect("engine should stop cleanly");

    assert!(
        !BackupEngine::list_backups(&backup_path)
            .expect("list backups")
            .get_state()
            .is_empty(),
        "state was not backed up at shutdown"
    );
}
