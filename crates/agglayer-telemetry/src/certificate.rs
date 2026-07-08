//! Certificate bridging-time metrics.
//!
//! Kicks off certificate observability with the end-to-end bridging time, a
//! per-stage breakdown of where that time is spent, and the latest settled
//! height -- all labeled by `network_id`.
//!
//! All metrics are recorded through the `record_*` helpers below, which are the
//! single extension point: adding a new certificate metric is one instrument
//! plus one helper here, with no changes to the emitters. See the Observability
//! page in `docs/knowledge-base` for the exposed names and semantics.

use lazy_static::lazy_static;
use opentelemetry::{global, metrics::*, KeyValue};

use crate::constant::AGGLAYER_OTEL_SCOPE_NAME;

/// Histogram bucket boundaries in seconds, shared by the total and per-stage
/// duration histograms. Covers the sub-second `submission` stage through
/// multi-minute settlement; tune once real distributions are observed.
const DURATION_BUCKETS_SECONDS: &[f64] = &[
    0.5, 1.0, 2.5, 5.0, 10.0, 30.0, 60.0, 120.0, 300.0, 600.0, 900.0, 1800.0,
];

/// `stage` label values for [`record_certificate_stage_completed`], one per
/// non-terminal certificate status (`Pending`, `Proven`, `Candidate`). Each
/// measures the time a certificate spends in that state before its next
/// transition. Adding or splitting a stage later is a new constant here plus a
/// record call at the transition -- no new metric required.
pub mod stage {
    /// Time in the `Pending` state (awaiting its proof), ending at `Proven`.
    pub const PENDING: &str = "pending";
    /// Time in the `Proven` state (settlement-job submission), ending at
    /// `Candidate`.
    pub const PROVEN: &str = "proven";
    /// Time in the `Candidate` state (L1 inclusion and confirmation), ending at
    /// `Settled`.
    pub const CANDIDATE: &str = "candidate";
}

lazy_static! {
    /// Total end-to-end certificate bridging time (`Pending` -> `Settled`).
    static ref CERTIFICATE_DURATION: Histogram<f64> = global::meter(AGGLAYER_OTEL_SCOPE_NAME)
        .f64_histogram("agglayer_certificate_duration_seconds")
        .with_description("End-to-end certificate bridging time from Pending to Settled, in seconds")
        .with_boundaries(DURATION_BUCKETS_SECONDS.to_vec())
        .build();

    /// Time spent in each certificate lifecycle stage (see [`stage`]).
    static ref CERTIFICATE_STAGE_DURATION: Histogram<f64> = global::meter(AGGLAYER_OTEL_SCOPE_NAME)
        .f64_histogram("agglayer_certificate_stage_duration_seconds")
        .with_description("Time spent in each certificate lifecycle stage, in seconds")
        .with_boundaries(DURATION_BUCKETS_SECONDS.to_vec())
        .build();

    /// Height of the latest settled certificate for a network.
    static ref CERTIFICATE_SETTLED_HEIGHT: Gauge<u64> = global::meter(AGGLAYER_OTEL_SCOPE_NAME)
        .u64_gauge("agglayer_certificate_settled_height")
        .with_description("Height of the latest settled certificate for a network")
        .build();

    /// Certificates that moved to `InError`, by the stage they errored from.
    static ref CERTIFICATE_ERRORS: Counter<u64> = global::meter(AGGLAYER_OTEL_SCOPE_NAME)
        .u64_counter("agglayer_certificate_errors")
        .with_description("Certificates that moved to InError, by the stage they errored from")
        .build();
}

/// Builds the common `[network_id, ..extra]` label set shared by every
/// certificate metric.
fn labels(network_id: u32, extra: &[KeyValue]) -> Vec<KeyValue> {
    let mut labels = Vec::with_capacity(1 + extra.len());
    labels.push(KeyValue::new("network_id", network_id.to_string()));
    labels.extend_from_slice(extra);
    labels
}

/// Records how long a certificate spent in a single lifecycle `stage` (one of
/// the [`stage`] constants).
#[inline]
pub fn record_certificate_stage_completed(network_id: u32, stage: &'static str, seconds: f64) {
    CERTIFICATE_STAGE_DURATION.record(
        seconds,
        &labels(network_id, &[KeyValue::new("stage", stage)]),
    );
}

/// Records the total end-to-end bridging time of a certificate.
#[inline]
pub fn record_certificate_total_duration(network_id: u32, seconds: f64) {
    CERTIFICATE_DURATION.record(seconds, &labels(network_id, &[]));
}

/// Records the height of the latest settled certificate for a network.
#[inline]
pub fn record_certificate_settled_height(network_id: u32, height: u64) {
    CERTIFICATE_SETTLED_HEIGHT.record(height, &labels(network_id, &[]));
}

/// Records a certificate moving to `InError`, labeled by the `stage` (the
/// status it held) it errored from.
#[inline]
pub fn record_certificate_error(network_id: u32, stage: &'static str) {
    CERTIFICATE_ERRORS.add(1, &labels(network_id, &[KeyValue::new("stage", stage)]));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_helper_functions() {
        record_certificate_stage_completed(1, stage::PENDING, 1.5);
        record_certificate_stage_completed(1, stage::PROVEN, 0.2);
        record_certificate_stage_completed(1, stage::CANDIDATE, 42.0);
        record_certificate_total_duration(1, 43.7);
        record_certificate_settled_height(1, 100);
        record_certificate_error(1, stage::PROVEN);
    }
}
