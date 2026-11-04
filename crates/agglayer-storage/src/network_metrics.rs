//! Native Prometheus metrics mirroring authoritative per-network pointers.

use std::{collections::HashMap, sync::Arc};

use agglayer_types::{CertificateId, CertificateStatus, Height, NetworkId};
use parking_lot::{Mutex, MutexGuard};
use prometheus::{IntGaugeVec, Opts, Registry};
use tracing::warn;

use crate::{error::Error, stores::NetworkInfoReader};

#[cfg(test)]
mod tests;

/// OpenTelemetry scope preserved as a constant label on the native collectors.
pub const AGGLAYER_NODE_NETWORK_OTEL_SCOPE_NAME: &str = "agglayer_node_network";

/// Height of the latest certificate per network and lifecycle stage.
pub const NETWORK_HEIGHT: &str = "agglayer_node_network_height";

/// Whether the latest pending certificate for a network is in error.
pub const NETWORK_LATEST_CERTIFICATE_IN_ERROR: &str =
    "agglayer_node_network_latest_certificate_in_error";

const NETWORK_HEIGHT_HELP: &str =
    "Height of the latest certificate per network and lifecycle stage";
const NETWORK_LATEST_CERTIFICATE_IN_ERROR_HELP: &str =
    "Whether the latest known certificate of the network is in error (1) or not (0)";
const NETWORK_ID_LABEL: &str = "network_id";
const STAGE_LABEL: &str = "stage";
const OTEL_SCOPE_NAME_LABEL: &str = "otel_scope_name";

/// Native collectors and mutation coordination shared by the pending and state
/// stores.
#[derive(Clone)]
pub struct NetworkMetrics {
    inner: Arc<Inner>,
}

struct Inner {
    height: IntGaugeVec,
    in_error: IntGaugeVec,
    mutation: Mutex<MutationState>,
}

#[derive(Default)]
struct MutationState {
    current_pending: HashMap<NetworkId, CertificateId>,
}

#[derive(Clone, Copy)]
pub(crate) enum NetworkStage {
    Pending,
    Proven,
    Settled,
}

impl NetworkStage {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Proven => "proven",
            Self::Settled => "settled",
        }
    }
}

impl NetworkMetrics {
    /// Create and register one shared height collector and one error collector.
    pub fn new(registry: &Registry) -> Result<Self, prometheus::Error> {
        let metrics = Self::try_unregistered()?;
        registry.register(Box::new(metrics.inner.height.clone()))?;
        registry.register(Box::new(metrics.inner.in_error.clone()))?;
        Ok(metrics)
    }

    /// Create isolated collectors for storage users that do not expose a
    /// registry.
    pub fn unregistered() -> Self {
        Self::try_unregistered().expect("static network metric descriptors must be valid")
    }

    fn try_unregistered() -> Result<Self, prometheus::Error> {
        let height = IntGaugeVec::new(
            Opts::new(NETWORK_HEIGHT, NETWORK_HEIGHT_HELP)
                .const_label(OTEL_SCOPE_NAME_LABEL, AGGLAYER_NODE_NETWORK_OTEL_SCOPE_NAME),
            &[NETWORK_ID_LABEL, STAGE_LABEL],
        )?;
        let in_error = IntGaugeVec::new(
            Opts::new(
                NETWORK_LATEST_CERTIFICATE_IN_ERROR,
                NETWORK_LATEST_CERTIFICATE_IN_ERROR_HELP,
            )
            .const_label(OTEL_SCOPE_NAME_LABEL, AGGLAYER_NODE_NETWORK_OTEL_SCOPE_NAME),
            &[NETWORK_ID_LABEL],
        )?;

        Ok(Self {
            inner: Arc::new(Inner {
                height,
                in_error,
                mutation: Mutex::new(MutationState::default()),
            }),
        })
    }

    /// Seed every native series from a complete, fallible storage snapshot.
    ///
    /// Storage read and decode errors abort hydration, as does a cached
    /// pointer without its header; out-of-range heights do not, they
    /// saturate.
    pub fn hydrate<S: NetworkInfoReader>(&self, state: &S) -> Result<(), Error> {
        let networks = state.get_all_latest_certificate_headers()?;
        // A series sitting at the gauge maximum saturated: no real network
        // reaches that height.
        let saturated: Vec<String> = networks
            .iter()
            .flat_map(|(network_id, headers)| {
                [
                    (NetworkStage::Pending, &headers.pending),
                    (NetworkStage::Proven, &headers.proven),
                    (NetworkStage::Settled, &headers.settled),
                ]
                .into_iter()
                .filter_map(move |(stage, header)| {
                    header
                        .as_ref()
                        .map(|header| (network_id, stage, Self::prometheus_height(header.height)))
                })
            })
            .filter(|(_, _, height)| *height == i64::MAX)
            .map(|(network_id, stage, _)| format!("{}/{}", network_id.to_u32(), stage.as_str()))
            .collect();
        if !saturated.is_empty() {
            warn!(
                pointers = saturated.join(" "),
                "Stored certificate heights exceed the Prometheus height gauge range, exporting \
                 the gauge maximum"
            );
        }

        let mut metrics = self.mutation();
        metrics.reset();
        for (network_id, headers) in networks {
            if let Some(header) = headers.pending {
                metrics.pending_written(
                    network_id,
                    header.certificate_id,
                    Self::prometheus_height(header.height),
                );
                metrics.header_written(network_id, header.certificate_id, &header.status);
            }
            if let Some(header) = headers.proven {
                metrics.height_written(
                    network_id,
                    NetworkStage::Proven,
                    Self::prometheus_height(header.height),
                );
            }
            if let Some(header) = headers.settled {
                metrics.height_written(
                    network_id,
                    NetworkStage::Settled,
                    Self::prometheus_height(header.height),
                );
            }
        }

        Ok(())
    }

    /// Refresh the error series after a direct admin latest-pending write.
    ///
    /// The cached pointer and its header are reread while holding the same
    /// mutation mutex used by both concrete stores. A newer pointer wins and
    /// is left untouched. Fails, after the override has already been written,
    /// when any of the network's cached pointers has no header.
    pub fn reconcile_pending_error<S: NetworkInfoReader>(
        &self,
        state: &S,
        network_id: NetworkId,
        expected_certificate_id: CertificateId,
    ) -> Result<(), Error> {
        let mut metrics = self.mutation();
        let Some(header) = state.get_latest_certificate_headers(network_id)?.pending else {
            return Ok(());
        };
        if header.certificate_id != expected_certificate_id {
            return Ok(());
        }

        metrics.pending_written(
            network_id,
            header.certificate_id,
            Self::prometheus_height(header.height),
        );
        metrics.header_written(network_id, header.certificate_id, &header.status);

        Ok(())
    }

    /// Convert a stored height into the `i64` domain of the height gauge.
    ///
    /// Heights are `u64` in storage. A height the gauge cannot hold saturates
    /// to its maximum: a metrics limitation is no reason to refuse a write or
    /// abort startup.
    pub(crate) fn prometheus_height(height: Height) -> i64 {
        i64::try_from(height.as_u64()).unwrap_or(i64::MAX)
    }

    pub(crate) fn mutation(&self) -> NetworkMetricsGuard<'_> {
        NetworkMetricsGuard {
            metrics: self,
            state: self.inner.mutation.lock(),
        }
    }
}

pub(crate) struct NetworkMetricsGuard<'a> {
    metrics: &'a NetworkMetrics,
    state: MutexGuard<'a, MutationState>,
}

impl NetworkMetricsGuard<'_> {
    pub(crate) fn pending_written(
        &mut self,
        network_id: NetworkId,
        certificate_id: CertificateId,
        height: i64,
    ) {
        self.height_written(network_id, NetworkStage::Pending, height);
        self.state
            .current_pending
            .insert(network_id, certificate_id);
        self.remove_error(network_id);
    }

    pub(crate) fn height_written(
        &mut self,
        network_id: NetworkId,
        stage: NetworkStage,
        height: i64,
    ) {
        let network_id = network_id.to_u32().to_string();
        self.metrics
            .inner
            .height
            .with_label_values(&[network_id.as_str(), stage.as_str()])
            .set(height);
    }

    pub(crate) fn header_written(
        &mut self,
        network_id: NetworkId,
        certificate_id: CertificateId,
        status: &CertificateStatus,
    ) {
        if self.state.current_pending.get(&network_id) != Some(&certificate_id) {
            return;
        }

        let network_id = network_id.to_u32().to_string();
        self.metrics
            .inner
            .in_error
            .with_label_values(&[&network_id])
            .set(i64::from(matches!(
                status,
                CertificateStatus::InError { .. }
            )));
    }

    fn remove_error(&mut self, network_id: NetworkId) {
        let network_id = network_id.to_u32().to_string();
        // Removing a series that was never created (a network's first pending
        // pointer) reports an error; only genuine removals matter here.
        let _ = self
            .metrics
            .inner
            .in_error
            .remove_label_values(&[&network_id]);
    }

    fn reset(&mut self) {
        self.metrics.inner.height.reset();
        self.metrics.inner.in_error.reset();
        self.state.current_pending.clear();
    }
}
