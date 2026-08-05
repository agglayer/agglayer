//! Per-network certificate state metrics.
//!
//! These gauges expose, for every network with a recorded certificate
//! pointer, the height of the latest certificate at each lifecycle stage
//! (`stage="pending" | "proven" | "settled"` label) plus a flag reporting
//! whether the latest known certificate is in error. They are *observable*
//! gauges: the closures given to [`register_network_state_metrics`] run on
//! every scrape of the `/metrics` endpoint, so exported values always
//! reflect the current storage content, including right after a restart.

use opentelemetry::{global, metrics::AsyncInstrument, KeyValue};

use crate::certificate::{CertificateStage, STAGE_LABEL_NAME};

const AGGLAYER_NODE_NETWORK_OTEL_SCOPE_NAME: &str = "agglayer_node_network";

/// Label carrying the network (rollup) id on every per-network series.
const NETWORK_ID_LABEL: &str = "network_id";

/// Gauge name: height of the latest certificate per network and lifecycle
/// stage (`stage` label).
pub const NETWORK_HEIGHT: &str = "agglayer_node_network_height";

/// Gauge name: whether the latest known certificate is in error (0/1).
pub const NETWORK_LATEST_CERTIFICATE_IN_ERROR: &str =
    "agglayer_node_network_latest_certificate_in_error";

/// One per-network height observation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NetworkHeightSample {
    pub network_id: u32,
    pub height: u64,
}

/// One per-network error-flag observation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NetworkErrorSample {
    pub network_id: u32,
    pub in_error: bool,
}

/// The set of closures feeding the per-network gauges.
///
/// Named fields prevent accidentally transposing the three height samplers,
/// which share a type and often report equal values in steady state.
pub struct NetworkStateSamplers {
    pub pending: Box<dyn Fn() -> Vec<NetworkHeightSample> + Send + Sync>,
    pub proven: Box<dyn Fn() -> Vec<NetworkHeightSample> + Send + Sync>,
    pub settled: Box<dyn Fn() -> Vec<NetworkHeightSample> + Send + Sync>,
    pub in_error: Box<dyn Fn() -> Vec<NetworkErrorSample> + Send + Sync>,
}

impl std::fmt::Debug for NetworkStateSamplers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NetworkStateSamplers")
            .finish_non_exhaustive()
    }
}

/// Register the per-network observable gauges: one height gauge carrying a
/// `stage` label (`pending` / `proven` / `settled`), plus the error flag.
///
/// The stage is a label rather than part of the metric name so dashboards
/// can aggregate `by (stage)` and future lifecycle stages extend the label
/// set without introducing new metric names.
///
/// Each closure is invoked on every `/metrics` scrape and must return one
/// sample per network that has a value. Networks without a value must be
/// omitted: a height of zero is a valid height (the first certificate of a
/// network) and must never be used as a placeholder.
///
/// # Runtime contract
///
/// The closures run synchronously inside the `/metrics` HTTP handler on
/// every scrape: they must not panic (a panic would break the metrics
/// endpoint on every scrape) and should be cheap and non-blocking.
///
/// On a data-source read error, closures should log upstream and return an
/// empty [`Vec`]. An empty result removes the series for that scrape, which
/// interacts with `absent()`-style alerting.
///
/// # Ordering
///
/// Call this only after the global meter provider has been installed (see
/// [`crate::ServerBuilder`]). Instruments created earlier silently bind to
/// the no-op meter and never export anything.
///
/// Call this function at most once per meter provider: repeated
/// registration accumulates duplicate instruments and callbacks.
pub fn register_network_state_metrics(samplers: NetworkStateSamplers) {
    let NetworkStateSamplers {
        pending,
        proven,
        settled,
        in_error,
    } = samplers;

    // The returned instrument handles are intentionally dropped: the callback
    // registrations live in the meter provider, not in the handles.
    //
    // The sampler-to-stage mapping happens here and only here, so the named
    // struct fields keep protecting against transposition.
    let _ = global::meter(AGGLAYER_NODE_NETWORK_OTEL_SCOPE_NAME)
        .u64_observable_gauge(NETWORK_HEIGHT)
        .with_description("Height of the latest certificate per network and lifecycle stage")
        .with_callback(observe_heights(CertificateStage::Pending, pending))
        .with_callback(observe_heights(CertificateStage::Proven, proven))
        .with_callback(observe_heights(CertificateStage::Settled, settled))
        .build();

    let _ = global::meter(AGGLAYER_NODE_NETWORK_OTEL_SCOPE_NAME)
        .u64_observable_gauge(NETWORK_LATEST_CERTIFICATE_IN_ERROR)
        .with_description(
            "Whether the latest known certificate of the network is in error (1) or not (0)",
        )
        .with_callback(move |observer| {
            for sample in in_error() {
                observer.observe(
                    u64::from(sample.in_error),
                    &[KeyValue::new(
                        NETWORK_ID_LABEL,
                        sample.network_id.to_string(),
                    )],
                );
            }
        })
        .build();
}

/// Builds the observation callback for one lifecycle stage of the height
/// gauge.
fn observe_heights(
    stage: CertificateStage,
    samples: Box<dyn Fn() -> Vec<NetworkHeightSample> + Send + Sync>,
) -> impl Fn(&dyn AsyncInstrument<u64>) + Send + Sync + 'static {
    move |observer| {
        for sample in samples() {
            observer.observe(
                sample.height,
                &[
                    KeyValue::new(NETWORK_ID_LABEL, sample.network_id.to_string()),
                    KeyValue::new(STAGE_LABEL_NAME, stage.to_string()),
                ],
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::*;
    use crate::testutils::MetricsHarness;

    /// Extract the value of the sample line for `name`, `network_id` and,
    /// when given, the `stage` label.
    fn sample_value(
        metrics: &str,
        name: &str,
        network_id: u32,
        stage: Option<&str>,
    ) -> Option<u64> {
        let network_label = format!("network_id=\"{network_id}\"");
        let stage_label = stage.map(|stage| format!("stage=\"{stage}\""));
        metrics
            .lines()
            .find(|line| {
                line.starts_with(&format!("{name}{{"))
                    && line.contains(&network_label)
                    && stage_label
                        .as_ref()
                        .is_none_or(|label| line.contains(label))
            })
            .and_then(|line| line.rsplit(' ').next())
            .map(|value| value.parse().unwrap())
    }

    #[test]
    fn network_state_gauges_export_per_network_series() {
        // This test deliberately owns the process-global meter provider:
        // nextest runs one process per test, so nothing else can race it.
        // Future exporter-based tests in this crate must keep that isolation
        // in mind (each test must install its own harness).
        let harness = MetricsHarness::install();

        let pending = Arc::new(Mutex::new(vec![
            NetworkHeightSample {
                network_id: 1,
                height: 0,
            },
            NetworkHeightSample {
                network_id: 14,
                height: 35,
            },
        ]));
        let proven = Arc::new(Mutex::new(vec![NetworkHeightSample {
            network_id: 14,
            height: 35,
        }]));
        let settled = Arc::new(Mutex::new(vec![NetworkHeightSample {
            network_id: 14,
            height: 34,
        }]));
        let in_error = Arc::new(Mutex::new(vec![
            NetworkErrorSample {
                network_id: 1,
                in_error: true,
            },
            NetworkErrorSample {
                network_id: 14,
                in_error: false,
            },
        ]));

        register_network_state_metrics(NetworkStateSamplers {
            pending: Box::new({
                let pending = pending.clone();
                move || pending.lock().unwrap().clone()
            }),
            proven: Box::new({
                let proven = proven.clone();
                move || proven.lock().unwrap().clone()
            }),
            settled: Box::new({
                let settled = settled.clone();
                move || settled.lock().unwrap().clone()
            }),
            in_error: Box::new({
                let in_error = in_error.clone();
                move || in_error.lock().unwrap().clone()
            }),
        });

        let metrics = harness.gather();

        assert_eq!(
            sample_value(&metrics, NETWORK_HEIGHT, 1, Some("pending")),
            Some(0)
        );
        assert_eq!(
            sample_value(&metrics, NETWORK_HEIGHT, 14, Some("pending")),
            Some(35)
        );
        assert_eq!(
            sample_value(&metrics, NETWORK_HEIGHT, 14, Some("proven")),
            Some(35)
        );
        assert_eq!(
            sample_value(&metrics, NETWORK_HEIGHT, 14, Some("settled")),
            Some(34)
        );
        // Stages are independent series: network 1 has no proven or settled
        // pointer, so those stage series must be absent, not zero.
        assert_eq!(
            sample_value(&metrics, NETWORK_HEIGHT, 1, Some("proven")),
            None
        );
        assert_eq!(
            sample_value(&metrics, NETWORK_HEIGHT, 1, Some("settled")),
            None
        );
        assert_eq!(
            sample_value(&metrics, NETWORK_LATEST_CERTIFICATE_IN_ERROR, 1, None),
            Some(1)
        );
        assert_eq!(
            sample_value(&metrics, NETWORK_LATEST_CERTIFICATE_IN_ERROR, 14, None),
            Some(0)
        );

        // Closures returning no samples must produce no sample lines: an
        // absent pointer means "no series", never a placeholder value.
        pending.lock().unwrap().clear();
        proven.lock().unwrap().clear();
        settled.lock().unwrap().clear();
        in_error.lock().unwrap().clear();

        let metrics = harness.gather();
        assert!(
            metrics
                .lines()
                .all(|line| !line.starts_with("agglayer_node_network_")),
            "expected no per-network sample lines, got:\n{metrics}"
        );
    }
}
