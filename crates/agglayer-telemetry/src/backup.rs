//! Backup subsystem metrics: how long a request waits, how long the backup
//! takes, and whether it succeeded.
//!
//! Two queues, state and epoch. A write path raises a request, it waits,
//! then it runs against the RocksDB backup engines.

use std::{
    sync::{Arc, Mutex},
    time::{Instant, SystemTime, UNIX_EPOCH},
};

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

/// Gauge name: age of the backup request currently being served, in
/// seconds, by `queue`. Zero when nothing is outstanding.
pub const BACKUP_OUTSTANDING_AGE_SECONDS: &str = "agglayer_node_backup_outstanding_age_seconds";

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

impl BackupQueue {
    /// Every queue, so the outstanding-age gauge always exports both series.
    const ALL: &'static [BackupQueue] = &[BackupQueue::State, BackupQueue::Epoch];
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

/// Histogram buckets in seconds.
///
/// Not [`crate::certificate::DURATION_BUCKETS_SECONDS`], which starts at
/// 0.5 s: an idle engine serves a request almost immediately, so that set
/// would report every healthy queue wait in one bucket.
const BACKUP_DURATION_BUCKETS_SECONDS: &[f64] = &[
    0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 30.0, 60.0, 120.0, 300.0, 600.0, 1800.0,
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
}

/// Register the observable gauge reporting the age of the request being
/// served, read from `metrics`.
///
/// One series per queue, zero when idle: a missing series would make "idle"
/// indistinguishable from "no data". Only a [`Weak`](std::sync::Weak) is
/// held, so the series disappear when `metrics` drops instead of freezing.
///
/// # Runtime contract
///
/// The callback runs inside the `/metrics` handler. Call this after the
/// global meter provider is installed (see [`crate::ServerBuilder`]), and at
/// most once per provider.
pub fn register_backup_metrics(metrics: &Arc<BackupMetrics>) {
    let metrics = Arc::downgrade(metrics);

    // The instrument handle is intentionally dropped: the callback
    // registration lives in the meter provider, not in the handle.
    let _ = global::meter(AGGLAYER_NODE_BACKUP_OTEL_SCOPE_NAME)
        .f64_observable_gauge(BACKUP_OUTSTANDING_AGE_SECONDS)
        .with_description("Age of the backup request currently being served, in seconds")
        .with_callback(move |observer| {
            let Some(metrics) = metrics.upgrade() else {
                return;
            };

            for &queue in BackupQueue::ALL {
                observer.observe(
                    metrics.age_seconds(queue),
                    &[KeyValue::new(QUEUE_LABEL_NAME, queue.to_string())],
                );
            }
        })
        .build();
}

/// Metrics for the backup subsystem: one handle per backup engine.
///
/// This is the whole interface the observed subsystem talks to; every
/// instrument, label and timer stays behind it.
///
/// It also holds the one thing a scrape cannot derive from the event-driven
/// instruments: what each queue is serving. A backup that starts and never
/// finishes is invisible to those, and shows only as a climbing
/// [`BACKUP_OUTSTANDING_AGE_SECONDS`].
#[derive(Debug, Default)]
pub struct BackupMetrics {
    state: Mutex<Option<Instant>>,
    epoch: Mutex<Option<Instant>>,
}

impl BackupMetrics {
    #[must_use]
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Record a state and pending backup request, and what happened to it.
    pub fn state_requested(&self, disposition: RequestDisposition) {
        Self::requested(BackupQueue::State, disposition);
    }

    /// Record an epoch backup request, and what happened to it.
    pub fn epoch_requested(&self, disposition: RequestDisposition) {
        Self::requested(BackupQueue::Epoch, disposition);
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
    pub fn state_backup(self: &Arc<Self>, enqueued_at: Instant) -> BackupRun {
        BackupRun::start(self.clone(), BackupQueue::State, enqueued_at)
    }

    /// Start serving an epoch backup raised at `enqueued_at`.
    pub fn epoch_backup(self: &Arc<Self>, enqueued_at: Instant) -> BackupRun {
        BackupRun::start(self.clone(), BackupQueue::Epoch, enqueued_at)
    }

    fn cell(&self, queue: BackupQueue) -> &Mutex<Option<Instant>> {
        match queue {
            BackupQueue::State => &self.state,
            BackupQueue::Epoch => &self.epoch,
        }
    }

    /// Age of the request being served on `queue`, or zero when idle.
    ///
    /// A poisoned lock reports zero rather than panicking: this runs inside
    /// the metrics endpoint.
    fn age_seconds(&self, queue: BackupQueue) -> f64 {
        self.cell(queue)
            .lock()
            .ok()
            .and_then(|cell| *cell)
            .map_or(0.0, |enqueued_at| enqueued_at.elapsed().as_secs_f64())
    }

    fn set_serving(&self, queue: BackupQueue, enqueued_at: Option<Instant>) {
        if let Ok(mut cell) = self.cell(queue).lock() {
            *cell = enqueued_at;
        }
    }
}

/// One backup being served.
///
/// Created by [`BackupMetrics::state_backup`] or
/// [`BackupMetrics::epoch_backup`], which record its queue wait. Finish with
/// [`BackupRun::succeeded`] or [`BackupRun::failed`]; both consume the run.
#[must_use = "a started backup must be finished with `succeeded` or `failed`"]
pub struct BackupRun {
    metrics: Arc<BackupMetrics>,
    queue: BackupQueue,
    started_at: Instant,
}

impl BackupRun {
    fn start(metrics: Arc<BackupMetrics>, queue: BackupQueue, enqueued_at: Instant) -> Self {
        metrics.set_serving(queue, Some(enqueued_at));
        BACKUP_QUEUE_WAIT.record(
            enqueued_at.elapsed().as_secs_f64(),
            &[KeyValue::new(QUEUE_LABEL_NAME, queue.to_string())],
        );

        Self {
            metrics,
            queue,
            started_at: Instant::now(),
        }
    }

    /// How long this backup has been running, for the caller's own logging.
    ///
    /// Exposed so the log line and the duration histogram share one timer
    /// instead of drifting apart.
    #[must_use]
    pub fn elapsed_ms(&self) -> u128 {
        self.started_at.elapsed().as_millis()
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

    /// The backup was taken.
    pub fn succeeded(self) {
        self.finish(BackupOutcome::Success);

        if let Ok(since_epoch) = SystemTime::now().duration_since(UNIX_EPOCH) {
            BACKUP_LAST_SUCCESS.record(
                since_epoch.as_secs(),
                &[KeyValue::new(QUEUE_LABEL_NAME, self.queue.to_string())],
            );
        }
    }

    /// The backup was not taken.
    pub fn failed(self) {
        self.finish(BackupOutcome::Failure);
    }

    fn finish(&self, outcome: BackupOutcome) {
        BACKUP_DURATION.record(
            self.started_at.elapsed().as_secs_f64(),
            &[
                KeyValue::new(QUEUE_LABEL_NAME, self.queue.to_string()),
                KeyValue::new(OUTCOME_LABEL_NAME, outcome.to_string()),
            ],
        );

        self.metrics.set_serving(self.queue, None);
    }
}

#[cfg(test)]
mod tests;
