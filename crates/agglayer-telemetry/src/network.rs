//! Per-network certificate state metrics.
//!
//! These gauges expose, for every network with a recorded certificate
//! pointer, the height of the latest pending / proven / settled certificate
//! plus a flag reporting whether the latest known certificate is in error. They
//! are *observable* gauges: the closures given to
//! [`register_network_state_metrics`] run on every scrape of the `/metrics`
//! endpoint, so exported values always reflect the current storage content,
//! including right after a restart.

use opentelemetry::{global, KeyValue};

const AGGLAYER_NODE_NETWORK_OTEL_SCOPE_NAME: &str = "agglayer_node_network";

/// Label carrying the network (rollup) id on every per-network series.
const NETWORK_ID_LABEL: &str = "network_id";

/// Gauge name: height of the latest pending certificate per network.
pub const NETWORK_PENDING_HEIGHT: &str = "agglayer_node_network_pending_height";

/// Gauge name: height of the latest proven certificate per network.
pub const NETWORK_PROVEN_HEIGHT: &str = "agglayer_node_network_proven_height";

/// Gauge name: height of the latest settled certificate per network.
pub const NETWORK_SETTLED_HEIGHT: &str = "agglayer_node_network_settled_height";

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

/// Register the four per-network observable gauges.
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

    register_height_gauge(
        NETWORK_PENDING_HEIGHT,
        "Height of the latest pending certificate per network",
        pending,
    );
    register_height_gauge(
        NETWORK_PROVEN_HEIGHT,
        "Height of the latest proven certificate per network",
        proven,
    );
    register_height_gauge(
        NETWORK_SETTLED_HEIGHT,
        "Height of the latest settled certificate per network",
        settled,
    );

    // The returned instrument handle is intentionally dropped: the callback
    // registration lives in the meter provider, not in the handle.
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

fn register_height_gauge(
    name: &'static str,
    description: &'static str,
    samples: Box<dyn Fn() -> Vec<NetworkHeightSample> + Send + Sync>,
) {
    let _ = global::meter(AGGLAYER_NODE_NETWORK_OTEL_SCOPE_NAME)
        .u64_observable_gauge(name)
        .with_description(description)
        .with_callback(move |observer| {
            for sample in samples() {
                observer.observe(
                    sample.height,
                    &[KeyValue::new(
                        NETWORK_ID_LABEL,
                        sample.network_id.to_string(),
                    )],
                );
            }
        })
        .build();
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use opentelemetry::global;
    use opentelemetry_sdk::metrics::SdkMeterProvider;
    use prometheus::{Encoder as _, Registry, TextEncoder};

    use super::*;

    fn gather(registry: &Registry) -> String {
        let encoder = TextEncoder::new();
        let mut out = Vec::new();
        encoder.encode(&registry.gather(), &mut out).unwrap();
        String::from_utf8(out).unwrap()
    }

    /// Extract the value of the sample line for `name` and `network_id`.
    fn sample_value(metrics: &str, name: &str, network_id: u32) -> Option<u64> {
        let label = format!("network_id=\"{network_id}\"");
        metrics
            .lines()
            .find(|line| line.starts_with(&format!("{name}{{")) && line.contains(&label))
            .and_then(|line| line.rsplit(' ').next())
            .map(|value| value.parse().unwrap())
    }

    #[test]
    fn network_state_gauges_export_per_network_series() {
        let registry = Registry::new();
        let exporter = opentelemetry_prometheus::exporter()
            .with_registry(registry.clone())
            .build()
            .unwrap();
        let provider = SdkMeterProvider::builder().with_reader(exporter).build();
        // This test deliberately owns the process-global meter provider:
        // nextest runs one process per test, so nothing else can race it.
        // Future exporter-based tests in this crate must keep that isolation
        // in mind (each test must install its own provider and registry).
        global::set_meter_provider(provider);

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

        let metrics = gather(&registry);

        assert_eq!(sample_value(&metrics, NETWORK_PENDING_HEIGHT, 1), Some(0));
        assert_eq!(sample_value(&metrics, NETWORK_PENDING_HEIGHT, 14), Some(35));
        assert_eq!(sample_value(&metrics, NETWORK_PROVEN_HEIGHT, 14), Some(35));
        assert_eq!(sample_value(&metrics, NETWORK_SETTLED_HEIGHT, 14), Some(34));
        assert_eq!(
            sample_value(&metrics, NETWORK_LATEST_CERTIFICATE_IN_ERROR, 1),
            Some(1)
        );
        assert_eq!(
            sample_value(&metrics, NETWORK_LATEST_CERTIFICATE_IN_ERROR, 14),
            Some(0)
        );

        // Closures returning no samples must produce no sample lines: an
        // absent pointer means "no series", never a placeholder value.
        pending.lock().unwrap().clear();
        proven.lock().unwrap().clear();
        settled.lock().unwrap().clear();
        in_error.lock().unwrap().clear();

        let metrics = gather(&registry);
        assert!(
            metrics
                .lines()
                .all(|line| !line.starts_with("agglayer_node_network_")),
            "expected no per-network sample lines, got:\n{metrics}"
        );
    }
}
