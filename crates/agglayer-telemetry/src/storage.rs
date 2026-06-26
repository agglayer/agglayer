//! Storage layer metrics.
//!
//! These instruments make it possible to detect synchronous RocksDB calls that
//! block Tokio worker threads. A slow operation recorded here, correlated with
//! a [`crate::runtime`] scheduler-lag spike while CPU is idle, is the signature
//! of blocking I/O running on the async runtime.

use lazy_static::lazy_static;
use opentelemetry::{global, metrics::*};

const AGGLAYER_STORAGE_OTEL_SCOPE_NAME: &str = "agglayer_storage";

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
