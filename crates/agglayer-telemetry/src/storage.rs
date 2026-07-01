//! Storage layer metrics.
//!
//! These instruments make it possible to detect synchronous RocksDB calls that
//! block Tokio worker threads. A slow operation recorded here, correlated with
//! a [`crate::runtime`] scheduler-lag spike while CPU is idle, is the signature
//! of blocking I/O running on the async runtime.

use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use lazy_static::lazy_static;
use opentelemetry::{global, metrics::*};

const AGGLAYER_STORAGE_OTEL_SCOPE_NAME: &str = "agglayer_storage";

/// Slow-op threshold in milliseconds.
///
/// RocksDB point operations are expected to complete in well under a
/// millisecond; an operation exceeding this budget while running on an async
/// runtime thread is a strong signal that it is blocking a Tokio worker (for
/// example a disk stall or compaction back-pressure).
///
/// Defaults to 25ms and is overridden once at startup from configuration via
/// [`set_slow_op_threshold`]. A process-global is used (rather than threading
/// the value through every `DB` constructor) to stay consistent with this
/// crate's global instruments: it is written once during startup and only read
/// afterwards.
static SLOW_OP_THRESHOLD_MS: AtomicU64 = AtomicU64::new(25);

/// Sets the storage slow-op threshold. Intended to be called once at startup,
/// before databases are opened.
pub fn set_slow_op_threshold(threshold: Duration) {
    SLOW_OP_THRESHOLD_MS.store(threshold.as_millis() as u64, Ordering::Relaxed);
}

/// Returns the currently configured storage slow-op threshold.
pub fn slow_op_threshold() -> Duration {
    Duration::from_millis(SLOW_OP_THRESHOLD_MS.load(Ordering::Relaxed))
}

lazy_static! {
    /// Duration of synchronous RocksDB operations, in milliseconds.
    ///
    /// Milliseconds (rather than seconds) are recorded deliberately: values then
    /// spread across the OpenTelemetry default histogram buckets
    /// (`5, 10, 25, 50, 100, 250, 500, 1000, ...`) without configuring a custom
    /// View. Healthy point operations land in the first bucket and stalls stand
    /// out. Tagged with `op` (e.g. `get`, `put`) and `cf` (column family).
    pub static ref OP_DURATION_MS: Histogram<f64> =
        global::meter(AGGLAYER_STORAGE_OTEL_SCOPE_NAME)
            .f64_histogram("storage_rocksdb_op_duration_ms")
            .with_description("Duration of synchronous RocksDB operations in milliseconds")
            .build();

    /// Count of RocksDB operations exceeding the slow-op threshold.
    ///
    /// Tagged with `op` and `cf`, matching [`OP_DURATION_MS`], so a rising rate
    /// can be attributed to a specific operation and column family.
    pub static ref SLOW_OP: Counter<u64> =
        global::meter(AGGLAYER_STORAGE_OTEL_SCOPE_NAME)
            .u64_counter("storage_rocksdb_slow_op_total")
            .with_description("RocksDB operations exceeding the slow-op threshold")
            .build();
}
