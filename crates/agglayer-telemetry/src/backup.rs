//! Backup subsystem metrics: how long a request waits, how long the backup
//! takes, and whether it succeeded.
//!
//! Two queues, state and epoch. A write path raises a request, it waits,
//! then it runs against the RocksDB backup engines.
//!
//! Every instrument is pushed at the event, so nothing here reads node state
//! on a scrape.

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use lazy_static::lazy_static;
use opentelemetry::{global, metrics::*, KeyValue};

const AGGLAYER_NODE_BACKUP_OTEL_SCOPE_NAME: &str = "agglayer_node_backup";

/// Name of the label carrying the queue a request travelled on.
const QUEUE_LABEL_NAME: &str = "queue";

/// Name of the label carrying what happened to a request when it was raised.
const DISPOSITION_LABEL_NAME: &str = "disposition";

/// Name of the label carrying the backed up database.
const DB_LABEL_NAME: &str = "db";

/// Name of the label carrying the backup outcome.
const OUTCOME_LABEL_NAME: &str = "outcome";

/// Counter name: backup requests raised, by `queue` and `disposition`.
///
/// No `_total` suffix here; the prometheus exporter adds it, so this exports
/// as `agglayer_node_backup_requests_total`.
pub const BACKUP_REQUESTS: &str = "agglayer_node_backup_requests";

/// Histogram name: time a request spent queued before the engine picked it
/// up, in seconds, by `queue`.
pub const BACKUP_QUEUE_WAIT_SECONDS: &str = "agglayer_node_backup_queue_wait_seconds";

/// Histogram name: time one backup took to run once picked up, in seconds,
/// by `queue` and `outcome`.
pub const BACKUP_DURATION_SECONDS: &str = "agglayer_node_backup_duration_seconds";

/// Gauge name: unix time at which the request currently being served was
/// raised, by `queue`. Zero when nothing is being served.
///
/// A dashboard subtracts this from `time()` to get the age, so a backup that
/// started and never finished shows as an age that keeps growing.
pub const BACKUP_SERVING_SINCE_TIMESTAMP_SECONDS: &str =
    "agglayer_node_backup_serving_since_timestamp_seconds";

/// Gauge name: unix time at which the last backup on this `queue` succeeded.
pub const BACKUP_LAST_SUCCESS_TIMESTAMP_SECONDS: &str =
    "agglayer_node_backup_last_success_timestamp_seconds";

/// Gauge name: number of files in the last successful backup of each `db`.
pub const BACKUP_FILES: &str = "agglayer_node_backup_files";

/// A backup queue, rendered as the `queue` label value.
///
/// A unit of work, not a database: one state request backs up the state
/// *and* pending databases. Which database a figure is about is [`BackupDb`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, strum_macros::Display)]
#[strum(serialize_all = "snake_case")]
enum BackupQueue {
    State,
    Epoch,
}

/// What happened to a request when it was raised, rendered as the
/// `disposition` label value.
///
/// `Coalesced` is normal, not a loss: the queued backup also covers the
/// coalesced write. `Rejected` means the queue is closed, which is either
/// the shutdown drain or a dead engine.
#[derive(Clone, Copy, Debug, PartialEq, Eq, strum_macros::Display)]
#[strum(serialize_all = "snake_case")]
pub enum RequestDisposition {
    Queued,
    Coalesced,
    Rejected,
}

/// A backed up database, rendered as the `db` label value.
///
/// Three separate databases, not nested: `state` holds certificate headers
/// and the trees, `pending` holds queued certificates and their proofs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, strum_macros::Display)]
#[strum(serialize_all = "snake_case")]
pub enum BackupDb {
    State,
    Pending,
    Epoch,
}

/// The outcome of one backup, rendered as the `outcome` label value.
///
/// Decided by whether a new backup was created, not by whether every step
/// succeeded: failing to purge old backups leaves the new one intact and is
/// reported on its own rather than as a failure to back up.
#[derive(Clone, Copy, Debug, PartialEq, Eq, strum_macros::Display)]
#[strum(serialize_all = "snake_case")]
enum BackupOutcome {
    Success,
    Failure,
}

/// Histogram buckets in seconds, shared by the queue-wait and duration
/// histograms.
///
/// Sub-second resolution is pointless here: the fast case is a backup of
/// seconds and the case worth watching is one of minutes. So the bottom is
/// coarse, with 10 s and 30 s only to keep a healthy backup distinguishable
/// from a slow one, and every minute is its own bucket through 15 min so a
/// backup creeping towards the multi-minute range is visible as it happens.
const BACKUP_DURATION_BUCKETS_SECONDS: &[f64] = &[
    1.0, 10.0, 30.0, 60.0, 120.0, 180.0, 240.0, 300.0, 360.0, 420.0, 480.0, 540.0, 600.0, 660.0,
    720.0, 780.0, 840.0, 900.0, 1200.0, 1500.0, 1800.0,
];

lazy_static! {
    static ref BACKUP_REQUESTS_COUNTER: Counter<u64> =
        global::meter(AGGLAYER_NODE_BACKUP_OTEL_SCOPE_NAME)
            .u64_counter(BACKUP_REQUESTS)
            .with_description("Number of backup requests raised, by queue and disposition")
            .build();
    static ref BACKUP_QUEUE_WAIT: Histogram<f64> =
        global::meter(AGGLAYER_NODE_BACKUP_OTEL_SCOPE_NAME)
            .f64_histogram(BACKUP_QUEUE_WAIT_SECONDS)
            .with_description(
                "Time a backup request spent queued before being picked up, in seconds"
            )
            .with_boundaries(BACKUP_DURATION_BUCKETS_SECONDS.to_vec())
            .build();
    static ref BACKUP_DURATION: Histogram<f64> =
        global::meter(AGGLAYER_NODE_BACKUP_OTEL_SCOPE_NAME)
            .f64_histogram(BACKUP_DURATION_SECONDS)
            .with_description("Time one backup took once picked up, in seconds")
            .with_boundaries(BACKUP_DURATION_BUCKETS_SECONDS.to_vec())
            .build();
    static ref BACKUP_LAST_SUCCESS: Gauge<u64> =
        global::meter(AGGLAYER_NODE_BACKUP_OTEL_SCOPE_NAME)
            .u64_gauge(BACKUP_LAST_SUCCESS_TIMESTAMP_SECONDS)
            .with_description("Unix time at which the last backup on this queue succeeded")
            .build();
    static ref BACKUP_FILE_COUNT: Gauge<u64> = global::meter(AGGLAYER_NODE_BACKUP_OTEL_SCOPE_NAME)
        .u64_gauge(BACKUP_FILES)
        .with_description("Number of files in the last successful backup of this database")
        .build();
    static ref BACKUP_SERVING_SINCE: Gauge<u64> =
        global::meter(AGGLAYER_NODE_BACKUP_OTEL_SCOPE_NAME)
            .u64_gauge(BACKUP_SERVING_SINCE_TIMESTAMP_SECONDS)
            .with_description("Unix time at which the request being served was raised, 0 when idle")
            .build();
}

/// Record a state and pending backup request, and what happened to it.
pub fn state_requested(disposition: RequestDisposition) {
    requested(BackupQueue::State, disposition);
}

/// Record an epoch backup request, and what happened to it.
pub fn epoch_requested(disposition: RequestDisposition) {
    requested(BackupQueue::Epoch, disposition);
}

fn requested(queue: BackupQueue, disposition: RequestDisposition) {
    BACKUP_REQUESTS_COUNTER.add(
        1,
        &[
            KeyValue::new(QUEUE_LABEL_NAME, queue.to_string()),
            KeyValue::new(DISPOSITION_LABEL_NAME, disposition.to_string()),
        ],
    );
}

/// Start serving the state and pending backup raised at `enqueued_at`.
pub fn state_backup(enqueued_at: Instant) -> BackupRun {
    BackupRun::start(BackupQueue::State, enqueued_at)
}

/// Start serving an epoch backup raised at `enqueued_at`.
pub fn epoch_backup(enqueued_at: Instant) -> BackupRun {
    BackupRun::start(BackupQueue::Epoch, enqueued_at)
}

/// Publish the unix time a request was raised, or `0` once it is served.
fn set_serving_since(queue: BackupQueue, unix_seconds: u64) {
    BACKUP_SERVING_SINCE.record(
        unix_seconds,
        &[KeyValue::new(QUEUE_LABEL_NAME, queue.to_string())],
    );
}

/// Unix time `elapsed` ago, or `0` if the clock is before the epoch.
fn unix_seconds_ago(elapsed: Duration) -> u64 {
    SystemTime::now()
        .checked_sub(elapsed)
        .and_then(|at| at.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |since_epoch| since_epoch.as_secs())
}

/// One backup being served.
///
/// Created by [`state_backup`] or [`epoch_backup`], which record its queue
/// wait. Finish with [`BackupRun::succeeded`] or [`BackupRun::failed`]; both
/// consume the run and hand back how long it took, for the caller's log.
#[must_use = "a started backup must be finished with `succeeded` or `failed`"]
pub struct BackupRun {
    queue: BackupQueue,
    started_at: Instant,
}

impl BackupRun {
    fn start(queue: BackupQueue, enqueued_at: Instant) -> Self {
        let waited = enqueued_at.elapsed();

        set_serving_since(queue, unix_seconds_ago(waited));
        BACKUP_QUEUE_WAIT.record(
            waited.as_secs_f64(),
            &[KeyValue::new(QUEUE_LABEL_NAME, queue.to_string())],
        );

        Self {
            queue,
            started_at: Instant::now(),
        }
    }

    /// Report the file count of the backup just taken of `db`.
    ///
    /// This is the multiplier on backup duration: the engine checks every
    /// file it references against the backup filesystem before copying.
    /// `None` reports nothing rather than a misleading zero.
    pub fn files(&self, db: BackupDb, files: Option<u64>) {
        if let Some(files) = files {
            BACKUP_FILE_COUNT.record(files, &[KeyValue::new(DB_LABEL_NAME, db.to_string())]);
        }
    }

    /// The backup was taken. Returns how long it took, in milliseconds.
    #[must_use]
    pub fn succeeded(self) -> u128 {
        if let Ok(since_epoch) = SystemTime::now().duration_since(UNIX_EPOCH) {
            BACKUP_LAST_SUCCESS.record(
                since_epoch.as_secs(),
                &[KeyValue::new(QUEUE_LABEL_NAME, self.queue.to_string())],
            );
        }

        self.finish(BackupOutcome::Success)
    }

    /// The backup was not taken. Returns how long it took, in milliseconds.
    #[must_use]
    pub fn failed(self) -> u128 {
        self.finish(BackupOutcome::Failure)
    }

    fn finish(&self, outcome: BackupOutcome) -> u128 {
        let elapsed = self.started_at.elapsed();

        BACKUP_DURATION.record(
            elapsed.as_secs_f64(),
            &[
                KeyValue::new(QUEUE_LABEL_NAME, self.queue.to_string()),
                KeyValue::new(OUTCOME_LABEL_NAME, outcome.to_string()),
            ],
        );

        set_serving_since(self.queue, 0);

        elapsed.as_millis()
    }
}

#[cfg(test)]
mod tests;
