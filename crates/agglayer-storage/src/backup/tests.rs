use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use agglayer_telemetry::{
    backup::{BACKUP_DURATION_SECONDS, BACKUP_FILES, BACKUP_REQUESTS},
    testutils::{sample, MetricsHarness},
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
    let state_db =
        Arc::new(StateStore::init_db(&tmp.path.join("state")).expect("state db should initialize"));
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

    let _backup_thread =
        tokio::task::spawn_blocking(move || started_receiver.recv_timeout(Duration::from_secs(1)))
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
    let state_db =
        Arc::new(StateStore::init_db(&tmp.path.join("state")).expect("state db should initialize"));
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
    // This test deliberately owns the process-global meter provider:
    // nextest runs one process per test, so nothing else can race it.
    let harness = MetricsHarness::install();

    let tmp = TempDBDir::new();
    let state_db =
        Arc::new(StateStore::init_db(&tmp.path.join("state")).expect("state db should initialize"));
    let pending_db = Arc::new(
        PendingStore::init_db(&tmp.path.join("pending")).expect("pending db should initialize"),
    );
    let epoch_db =
        Arc::new(StateStore::init_db(&tmp.path.join("epoch")).expect("epoch db should initialize"));
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
    backup_client
        .backup_state()
        .expect("a second request should coalesce into the queued one");
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

    // The same run proves the metrics come from the production call sites,
    // and that a coalesced request produces no second backup.
    let body = harness.gather();
    let requests = format!("{BACKUP_REQUESTS}_total");
    let durations = format!("{BACKUP_DURATION_SECONDS}_count");
    for (name, labels, expected) in [
        (
            &requests,
            [("queue", "state"), ("disposition", "queued")],
            1.0,
        ),
        (
            &requests,
            [("queue", "state"), ("disposition", "coalesced")],
            1.0,
        ),
        (
            &requests,
            [("queue", "epoch"), ("disposition", "queued")],
            EPOCHS as f64,
        ),
        (
            &durations,
            [("queue", "state"), ("outcome", "success")],
            1.0,
        ),
        (
            &durations,
            [("queue", "epoch"), ("outcome", "success")],
            EPOCHS as f64,
        ),
    ] {
        assert_eq!(
            sample(&body, name, &labels),
            Some(expected),
            "{name} {labels:?}, got:\n{body}"
        );
    }

    // A non-zero count does not prove the backup restores: a read-only
    // handle yields a plausible count and zero rows.
    for db in ["state", "pending", "epoch"] {
        let files = sample(&body, BACKUP_FILES, &[("db", db)])
            .unwrap_or_else(|| panic!("{db} file count should export, got:\n{body}"));
        assert!(files > 0.0, "{db} backup referenced no files");
    }
}

#[tokio::test]
async fn a_failed_backup_does_not_stop_the_engine() {
    let harness = MetricsHarness::install();

    let tmp = TempDBDir::new();
    let state_db =
        Arc::new(StateStore::init_db(&tmp.path.join("state")).expect("state db should initialize"));
    let pending_db = Arc::new(
        PendingStore::init_db(&tmp.path.join("pending")).expect("pending db should initialize"),
    );
    let epoch_db =
        Arc::new(StateStore::init_db(&tmp.path.join("epoch")).expect("epoch db should initialize"));
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

    // A plain file where the epoch 7 backup directory should go makes
    // that backup fail. The engine must log the failure and move on:
    // epoch 8 queued behind it still has to get its only backup.
    std::fs::write(backup_path.join("epochs").join("7"), b"not a directory")
        .expect("blocking file should be writable");

    backup_client
        .backup_epoch(epoch_db.clone(), EpochNumber::new(7))
        .expect("epoch backup request should be queued");
    backup_client
        .backup_epoch(epoch_db, EpochNumber::new(8))
        .expect("epoch backup request should be queued");
    cancellation_token.cancel();

    backup_engine
        .run()
        .await
        .expect("backup engine should survive a failed epoch backup");

    let epoch_8_backups = read_dir(backup_path.join("epochs").join("8").join("meta"))
        .expect("epoch 8 backup metadata should exist")
        .count();
    assert_eq!(
        epoch_8_backups, 1,
        "the epoch backup queued behind the failed one should still be taken"
    );

    // Epoch 7 failed and epoch 8 succeeded, and the outcome label has to
    // say so rather than counting two backups.
    let body = harness.gather();
    let durations = format!("{BACKUP_DURATION_SECONDS}_count");
    for (outcome, expected) in [("success", 1.0), ("failure", 1.0)] {
        assert_eq!(
            sample(
                &body,
                &durations,
                &[("queue", "epoch"), ("outcome", outcome)]
            ),
            Some(expected),
            "epoch backups should be counted one {outcome} each, got:\n{body}"
        );
    }
}
