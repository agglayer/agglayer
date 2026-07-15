//! Certificate bridging-time metrics: end-to-end duration and a per-stage
//! breakdown, labeled by `network_id`.

use std::time::Instant;

use lazy_static::lazy_static;
use opentelemetry::{global, metrics::*, KeyValue};

const AGGLAYER_NODE_CERTIFICATE_OTEL_SCOPE_NAME: &str = "agglayer_node_certificate";

/// Name of the label carrying the certificate lifecycle stage.
pub(crate) const STAGE_LABEL_NAME: &str = "stage";

/// A certificate lifecycle stage, rendered as the `stage` label value.
///
/// The metric families each use a subset: the duration histograms time the
/// non-terminal stages (`Pending`, `Proven`, `Candidate`), while the
/// per-network height gauge reports pointer positions (`Pending`, `Proven`,
/// `Settled`). Sharing one enum keeps the label values consistent across
/// families.
#[derive(Clone, Copy, Debug, PartialEq, Eq, strum_macros::Display)]
#[strum(serialize_all = "snake_case")]
pub enum CertificateStage {
    Pending,
    Proven,
    Candidate,
    Settled,
}

/// Histogram buckets in seconds, from the sub-second submission stage to
/// multi-minute settlement. Settlement job durations live on the same
/// L1-inclusion timescale, so [`crate::settlement`] shares these buckets.
pub(crate) const DURATION_BUCKETS_SECONDS: &[f64] = &[
    0.5, 1.0, 2.5, 5.0, 10.0, 30.0, 60.0, 120.0, 300.0, 600.0, 900.0, 1800.0,
];

lazy_static! {
    static ref CERTIFICATE_DURATION: Histogram<f64> =
        global::meter(AGGLAYER_NODE_CERTIFICATE_OTEL_SCOPE_NAME)
            .f64_histogram("agglayer_certificate_duration_seconds")
            .with_description(
                "End-to-end certificate bridging time, Pending to Settled, in seconds"
            )
            .with_boundaries(DURATION_BUCKETS_SECONDS.to_vec())
            .build();
    static ref CERTIFICATE_STAGE_DURATION: Histogram<f64> =
        global::meter(AGGLAYER_NODE_CERTIFICATE_OTEL_SCOPE_NAME)
            .f64_histogram("agglayer_certificate_stage_duration_seconds")
            .with_description("Time spent in each certificate lifecycle stage, in seconds")
            .with_boundaries(DURATION_BUCKETS_SECONDS.to_vec())
            .build();
}

fn labels(network_id: u32, extra: &[KeyValue]) -> Vec<KeyValue> {
    let mut labels = Vec::with_capacity(1 + extra.len());
    labels.push(KeyValue::new("network_id", network_id.to_string()));
    labels.extend_from_slice(extra);
    labels
}

/// Records the duration of one lifecycle `stage`.
#[inline]
pub fn record_certificate_stage_completed(network_id: u32, stage: CertificateStage, seconds: f64) {
    CERTIFICATE_STAGE_DURATION.record(
        seconds,
        &labels(
            network_id,
            &[KeyValue::new(STAGE_LABEL_NAME, stage.to_string())],
        ),
    );
}

/// Records the total end-to-end bridging time.
#[inline]
pub fn record_certificate_total_duration(network_id: u32, seconds: f64) {
    CERTIFICATE_DURATION.record(seconds, &labels(network_id, &[]));
}

/// Times a certificate's bridging: `start` on pickup, `complete_stage` at each
/// transition, `complete` once settled.
pub struct CertificateTimer {
    network_id: u32,
    overall_start: Instant,
    stage_start: Instant,
}

impl CertificateTimer {
    pub fn start(network_id: u32) -> Self {
        let now = Instant::now();
        Self {
            network_id,
            overall_start: now,
            stage_start: now,
        }
    }

    /// Records the finished `stage` and resets the stage clock.
    pub fn complete_stage(&mut self, stage: CertificateStage) {
        record_certificate_stage_completed(
            self.network_id,
            stage,
            self.stage_start.elapsed().as_secs_f64(),
        );
        self.stage_start = Instant::now();
    }

    pub fn complete(&self) {
        record_certificate_total_duration(
            self.network_id,
            self.overall_start.elapsed().as_secs_f64(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helpers_do_not_panic() {
        record_certificate_stage_completed(1, CertificateStage::Pending, 1.5);
        record_certificate_total_duration(1, 43.7);

        let mut timer = CertificateTimer::start(1);
        timer.complete_stage(CertificateStage::Proven);
        timer.complete();
    }
}
