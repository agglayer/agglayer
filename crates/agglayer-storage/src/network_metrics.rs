//! Native Prometheus metrics mirroring authoritative per-network pointers.

use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use agglayer_types::{CertificateId, CertificateStatus, Height, NetworkId};
use parking_lot::{Mutex, MutexGuard};
use prometheus::{IntGaugeVec, Opts, Registry};
use tracing::warn;

use crate::{
    columns::{
        latest_pending_certificate_per_network::PendingCertificate,
        latest_proven_certificate_per_network::ProvenCertificate,
        latest_settled_certificate_per_network::SettledCertificate,
    },
    error::Error,
    stores::{PendingCertificateReader, StateReader},
};

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
    /// Whether the out-of-range height warning has been logged in this run.
    saturation_warned: AtomicBool,
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
                saturation_warned: AtomicBool::new(false),
            }),
        })
    }

    /// Seed every native series from a complete, fallible storage snapshot.
    ///
    /// Storage read and decode errors abort hydration. Heights beyond the
    /// gauge range do not: they saturate, see [`Self::prometheus_height`].
    pub fn hydrate<P, S>(&self, pending: &P, state: &S) -> Result<(), Error>
    where
        P: PendingCertificateReader,
        S: StateReader,
    {
        let pending_pointers = pending
            .get_current_pending_heights()?
            .into_iter()
            .map(|(network_id, PendingCertificate(certificate_id, height))| {
                (
                    network_id,
                    certificate_id,
                    self.prometheus_height(network_id, NetworkStage::Pending, height),
                )
            })
            .collect::<Vec<_>>();
        let proven_pointers = pending
            .get_current_proven_height()?
            .into_iter()
            .map(|ProvenCertificate(_, network_id, height)| {
                (
                    network_id,
                    self.prometheus_height(network_id, NetworkStage::Proven, height),
                )
            })
            .collect::<Vec<_>>();
        let settled_pointers = state
            .get_current_settled_height()?
            .into_iter()
            .map(|(network_id, SettledCertificate(_, height, _, _))| {
                (
                    network_id,
                    self.prometheus_height(network_id, NetworkStage::Settled, height),
                )
            })
            .collect::<Vec<_>>();
        let pending_statuses = pending_pointers
            .iter()
            .map(|(network_id, certificate_id, _)| {
                Ok((
                    *network_id,
                    *certificate_id,
                    state
                        .get_certificate_header(certificate_id)?
                        .map(|header| header.status),
                ))
            })
            .collect::<Result<Vec<_>, Error>>()?;

        let mut metrics = self.mutation();
        metrics.reset();
        for (network_id, certificate_id, height) in pending_pointers {
            metrics.pending_written(network_id, certificate_id, height);
        }
        for (network_id, height) in proven_pointers {
            metrics.height_written(network_id, NetworkStage::Proven, height);
        }
        for (network_id, height) in settled_pointers {
            metrics.height_written(network_id, NetworkStage::Settled, height);
        }
        for (network_id, certificate_id, status) in pending_statuses {
            if let Some(status) = status {
                metrics.header_written(network_id, certificate_id, &status);
            }
        }

        Ok(())
    }

    /// Refresh the error series after a direct admin latest-pending write.
    ///
    /// The authoritative pointer and header are reread while holding the same
    /// mutation mutex used by both concrete stores. A newer pointer wins and is
    /// left untouched.
    pub fn reconcile_pending_error<P, S>(
        &self,
        pending: &P,
        state: &S,
        network_id: NetworkId,
        expected_certificate_id: CertificateId,
    ) -> Result<(), Error>
    where
        P: PendingCertificateReader,
        S: StateReader,
    {
        let mut metrics = self.mutation();
        let Some((certificate_id, height)) =
            pending.get_latest_pending_certificate_for_network(&network_id)?
        else {
            return Ok(());
        };
        if certificate_id != expected_certificate_id {
            return Ok(());
        }

        let height = self.prometheus_height(network_id, NetworkStage::Pending, height);
        let header = state.get_certificate_header(&certificate_id)?;
        metrics.pending_written(network_id, certificate_id, height);
        if let Some(header) = header {
            metrics.header_written(network_id, certificate_id, &header.status);
        }

        Ok(())
    }

    /// Convert a stored height into the `i64` domain of the height gauge.
    ///
    /// Heights are `u64` in storage. A stored height above `i64::MAX` is a
    /// metrics limitation, not a reason to refuse a write or abort startup
    /// (bali carried such pending pointers and v0.6.1-rc.1 crash-looped on
    /// them at hydration), so the value saturates to `i64::MAX`, which the
    /// exposition format renders as `2^63`. The first occurrence per run is
    /// logged; later ones are silent, and saturated series are recognisable
    /// by sitting at that maximum.
    pub(crate) fn prometheus_height(
        &self,
        network_id: NetworkId,
        stage: NetworkStage,
        height: Height,
    ) -> i64 {
        match i64::try_from(height.as_u64()) {
            Ok(height) => height,
            Err(_) => {
                if !self.inner.saturation_warned.swap(true, Ordering::Relaxed) {
                    warn!(
                        network_id = network_id.to_u32(),
                        stage = stage.as_str(),
                        height = height.as_u64(),
                        saturated_to = i64::MAX,
                        "Certificate height exceeds the range of the native Prometheus height \
                         gauge, exporting the gauge maximum instead. Further occurrences in this \
                         run are not logged: saturated series are the ones pinned at 2^63."
                    );
                }
                i64::MAX
            }
        }
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
