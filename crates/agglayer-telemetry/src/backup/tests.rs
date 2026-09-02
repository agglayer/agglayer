use std::{thread, time::Duration};

use super::*;
use crate::testutils::{sample, MetricsHarness};

/// Only instrument *shape* is asserted here. That the production call sites
/// emit at all is proved in `agglayer-storage`'s own backup tests, against
/// the real engine, which is strictly stronger.
#[test]
fn instrument_shapes_are_what_the_docs_promise() {
    // This test deliberately owns the process-global meter provider:
    // nextest runs one process per test, so nothing else can race it.
    let harness = MetricsHarness::install();

    let metrics = BackupMetrics::new();

    let run = metrics.state_backup(Instant::now());
    run.files(BackupDb::State, Some(23_798));
    run.files(BackupDb::Pending, None);
    run.succeeded();

    metrics.epoch_backup(Instant::now()).failed();

    let body = harness.gather();

    // 0.25 is not an OTel default, so this bucket exists only with the
    // backup-specific boundaries. The shared ones start at 0.5, which would
    // report every healthy queue wait in one bucket.
    assert_eq!(
        sample(
            &body,
            &format!("{BACKUP_QUEUE_WAIT_SECONDS}_bucket"),
            &[("queue", "state"), ("le", "0.25")],
        ),
        Some(1.0),
        "backup-specific buckets were not applied, got:\n{body}"
    );

    // An absent file count must report nothing, not a misleading zero.
    assert_eq!(
        sample(&body, BACKUP_FILES, &[("db", "state")]),
        Some(23_798.0),
        "file count gauge, got:\n{body}"
    );
    assert_eq!(
        sample(&body, BACKUP_FILES, &[("db", "pending")]),
        None,
        "an absent file count must not export a zero, got:\n{body}"
    );

    // Only a successful run stamps the timestamp, and the epoch run failed.
    // Staleness alerting computes `time() - metric`, so millis or a
    // monotonic instant would give a nonsense age, not a missing series;
    // 2020-01-01 is a floor no correct unix-seconds value can fall below.
    let stamped = sample(
        &body,
        BACKUP_LAST_SUCCESS_TIMESTAMP_SECONDS,
        &[("queue", "state")],
    )
    .unwrap_or_else(|| panic!("a successful run stamps its kind, got:\n{body}"));
    assert!(
        stamped > 1_577_836_800.0,
        "expected seconds since the unix epoch, got {stamped}"
    );
    assert_eq!(
        sample(
            &body,
            BACKUP_LAST_SUCCESS_TIMESTAMP_SECONDS,
            &[("queue", "epoch")]
        ),
        None,
        "a failed run must not stamp a success, got:\n{body}"
    );
}

#[test]
fn outstanding_age_gauge_tracks_progress_and_disappears_with_it() {
    let harness = MetricsHarness::install();

    let metrics = BackupMetrics::new();
    register_backup_metrics(&metrics);

    // An idle subsystem must still export zero, or alerting cannot tell
    // "nothing outstanding" from "no data".
    let body = harness.gather();
    for kind in ["state", "epoch"] {
        assert_eq!(
            sample(&body, BACKUP_OUTSTANDING_AGE_SECONDS, &[("queue", kind)]),
            Some(0.0),
            "an idle {kind} queue must export zero, got:\n{body}"
        );
    }

    let run = metrics.state_backup(Instant::now());
    thread::sleep(Duration::from_millis(20));

    let body = harness.gather();
    let state_age = sample(&body, BACKUP_OUTSTANDING_AGE_SECONDS, &[("queue", "state")])
        .unwrap_or_else(|| panic!("state age should export, got:\n{body}"));
    assert!(
        state_age > 0.0,
        "a request being served should report a non-zero age, got {state_age}"
    );
    assert_eq!(
        sample(&body, BACKUP_OUTSTANDING_AGE_SECONDS, &[("queue", "epoch")]),
        Some(0.0),
        "the untouched epoch queue must stay at zero, got:\n{body}"
    );

    // The callback re-reads the handle, so no re-registration is needed.
    run.succeeded();
    let body = harness.gather();
    assert_eq!(
        sample(&body, BACKUP_OUTSTANDING_AGE_SECONDS, &[("queue", "state")]),
        Some(0.0),
        "going idle should be reflected on the next scrape, got:\n{body}"
    );

    // Only a weak reference is held, so the series must not freeze on a
    // stale age.
    drop(metrics);
    let body = harness.gather();
    assert!(
        body.lines()
            .all(|line| !line.starts_with(BACKUP_OUTSTANDING_AGE_SECONDS)),
        "the gauge must not outlive the handle it reads, got:\n{body}"
    );
}
