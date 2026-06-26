//! Tokio runtime responsiveness metrics.
//!
//! These instruments surface scheduler stalls: if a worker thread is blocked
//! (for example by a synchronous RocksDB call on an async thread), ready tasks
//! are not polled and even a trivial endpoint such as `/health` appears to hang.
//! They are populated by a lightweight probe and require no `tokio_unstable`
//! build flags.

use lazy_static::lazy_static;
use opentelemetry::{global, metrics::*};

const AGGLAYER_RUNTIME_OTEL_SCOPE_NAME: &str = "agglayer_runtime";

lazy_static! {
    /// Scheduler lag in milliseconds: how late a fixed-interval probe tick
    /// fired relative to its deadline. Sustained lag while CPU is idle indicates
    /// worker threads were blocked and unable to make progress.
    pub static ref SCHEDULER_LAG_MS: Histogram<f64> =
        global::meter(AGGLAYER_RUNTIME_OTEL_SCOPE_NAME)
            .f64_histogram("runtime_scheduler_lag_ms")
            .with_description("Tokio scheduler lag (timer overshoot) in milliseconds")
            .build();

    /// Number of tasks waiting in the runtime's global (injector) run queue.
    ///
    /// A rising depth while CPU usage stays low means tasks are ready but not
    /// being polled, i.e. the worker pool is starved.
    pub static ref GLOBAL_QUEUE_DEPTH: Gauge<u64> =
        global::meter(AGGLAYER_RUNTIME_OTEL_SCOPE_NAME)
            .u64_gauge("runtime_global_queue_depth")
            .with_description("Tasks waiting in the Tokio global run queue")
            .build();
}
