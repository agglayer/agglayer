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

    let run = state_backup(Instant::now());
    run.files(BackupDb::State, Some(23_798));
    run.files(BackupDb::Pending, None);
    let _ = run.succeeded();

    let _ = epoch_backup(Instant::now()).failed();

    let body = harness.gather();

    // 540 s is the 9 minute backup the incident measured, and is not an OTel
    // default boundary, so this bucket exists only with the backup-specific
    // set applied.
    assert_eq!(
        sample(
            &body,
            &format!("{BACKUP_QUEUE_WAIT_SECONDS}_bucket"),
            &[("queue", "state"), ("le", "540")],
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
fn serving_since_gauge_publishes_the_enqueue_time_and_clears_when_done() {
    let harness = MetricsHarness::install();

    let run = state_backup(Instant::now());

    // While a request is being served the gauge carries the unix time it was
    // raised, so a dashboard gets its age from `time() - metric`.
    let body = harness.gather();
    let serving_since = sample(
        &body,
        BACKUP_SERVING_SINCE_TIMESTAMP_SECONDS,
        &[("queue", "state")],
    )
    .unwrap_or_else(|| panic!("serving-since should export, got:\n{body}"));
    assert!(
        serving_since > 1_577_836_800.0,
        "expected seconds since the unix epoch, got {serving_since}"
    );

    // Finishing clears it to zero, which is how a dashboard tells "nothing
    // is being served" from a request that has been outstanding for ages.
    let _ = run.succeeded();
    let body = harness.gather();
    assert_eq!(
        sample(
            &body,
            BACKUP_SERVING_SINCE_TIMESTAMP_SECONDS,
            &[("queue", "state")]
        ),
        Some(0.0),
        "an idle queue must report zero, got:\n{body}"
    );
}
