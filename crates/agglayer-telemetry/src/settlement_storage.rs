use std::time::Duration;

use lazy_static::lazy_static;
use opentelemetry::{global, metrics::*};

use crate::constant::AGGLAYER_STORAGE_OTEL_SCOPE_NAME;

lazy_static! {
    pub static ref LATEST_ATTEMPT_READS_TOTAL: Counter<u64> =
        global::meter(AGGLAYER_STORAGE_OTEL_SCOPE_NAME)
            .u64_counter("settlement_latest_attempt_reads_total")
            .with_description("Total number of latest-settlement-attempt read operations")
            .build();
    pub static ref ATTEMPT_WRITES_TOTAL: Counter<u64> =
        global::meter(AGGLAYER_STORAGE_OTEL_SCOPE_NAME)
            .u64_counter("settlement_attempt_writes_total")
            .with_description("Total number of settlement-attempt write operations")
            .build();
    pub static ref LATEST_ATTEMPT_READ_DURATION_MS: Histogram<f64> =
        global::meter(AGGLAYER_STORAGE_OTEL_SCOPE_NAME)
            .f64_histogram("settlement_latest_attempt_read_duration_ms")
            .with_description("Latency for latest-settlement-attempt reads in milliseconds")
            .build();
    pub static ref ATTEMPT_WRITE_DURATION_MS: Histogram<f64> =
        global::meter(AGGLAYER_STORAGE_OTEL_SCOPE_NAME)
            .f64_histogram("settlement_attempt_write_duration_ms")
            .with_description("Latency for settlement-attempt writes in milliseconds")
            .build();
}

#[inline]
pub fn record_latest_attempt_read(duration: Duration, found: bool, success: bool) {
    let attrs = &[
        opentelemetry::KeyValue::new("found", found.to_string()),
        opentelemetry::KeyValue::new("success", success.to_string()),
    ];

    LATEST_ATTEMPT_READS_TOTAL.add(1, attrs);
    LATEST_ATTEMPT_READ_DURATION_MS.record(duration.as_secs_f64() * 1000.0, attrs);
}

#[inline]
pub fn record_attempt_write(duration: Duration, success: bool) {
    let attrs = &[opentelemetry::KeyValue::new("success", success.to_string())];

    ATTEMPT_WRITES_TOTAL.add(1, attrs);
    ATTEMPT_WRITE_DURATION_MS.record(duration.as_secs_f64() * 1000.0, attrs);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settlement_storage_metrics_record_without_panicking() {
        record_latest_attempt_read(Duration::from_millis(3), true, true);
        record_latest_attempt_read(Duration::from_millis(5), false, false);
        record_attempt_write(Duration::from_millis(4), true);
        record_attempt_write(Duration::from_millis(6), false);
    }
}
