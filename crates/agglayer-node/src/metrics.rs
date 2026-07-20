//! Registration of the per-network certificate state metrics.
//!
//! The gauges defined in [`agglayer_telemetry::network`] are observable: the
//! closures registered here run on every `/metrics` scrape and read the
//! per-network pointer column families, so the exported values always match
//! the storage content, including after restarts and admin edits.

use std::sync::Arc;

use agglayer_storage::{
    columns::{
        latest_pending_certificate_per_network::PendingCertificate,
        latest_proven_certificate_per_network::ProvenCertificate,
        latest_settled_certificate_per_network::SettledCertificate,
    },
    stores::{
        pending::PendingStore, state::StateStore, PendingCertificateReader as _, StateReader as _,
    },
};
use agglayer_telemetry::network::{NetworkErrorSample, NetworkHeightSample, NetworkStateSamplers};
use agglayer_types::CertificateStatus;
use tracing::warn;

/// Register the per-network certificate state gauges backed by the node
/// stores.
///
/// Must run after the metrics server has installed the global meter provider
/// (the node does this before `Node::start` runs).
///
/// The closures deliberately hold [`Weak`](std::sync::Weak) store references
/// and upgrade them on every scrape: the global meter provider outlives
/// in-process node shutdowns, so a registration must not keep the stores
/// alive past the owning node instance (a retained RocksDB handle would
/// keep the storage path locked, e.g. during backup restores). The weak
/// references extend the store lifetime by at most one in-flight collect
/// (an `upgrade()` pins a store only for the duration of a single sampling
/// pass); once the stores drop, the series simply disappear.
pub(crate) fn register_network_state_metrics(
    pending_store: &Arc<PendingStore>,
    state_store: &Arc<StateStore>,
) {
    let pending = Arc::downgrade(pending_store);
    let proven = Arc::downgrade(pending_store);
    let settled = Arc::downgrade(state_store);
    let in_error_pending = Arc::downgrade(pending_store);
    let in_error_state = Arc::downgrade(state_store);

    agglayer_telemetry::network::register_network_state_metrics(NetworkStateSamplers {
        pending: Box::new(move || match pending.upgrade() {
            Some(store) => collect_pending_heights(&store),
            None => Vec::new(),
        }),
        proven: Box::new(move || match proven.upgrade() {
            Some(store) => collect_proven_heights(&store),
            None => Vec::new(),
        }),
        settled: Box::new(move || match settled.upgrade() {
            Some(store) => collect_settled_heights(&store),
            None => Vec::new(),
        }),
        in_error: Box::new(
            move || match (in_error_pending.upgrade(), in_error_state.upgrade()) {
                (Some(pending_store), Some(state_store)) => {
                    collect_error_flags(&pending_store, &state_store)
                }
                _ => Vec::new(),
            },
        ),
    });
}

fn collect_pending_heights(store: &PendingStore) -> Vec<NetworkHeightSample> {
    match store.get_current_pending_heights() {
        Ok(pointers) => pointers
            .into_iter()
            .map(
                |(network_id, PendingCertificate(_, height))| NetworkHeightSample {
                    network_id: network_id.to_u32(),
                    height: height.as_u64(),
                },
            )
            .collect(),
        Err(error) => {
            warn!(?error, "Failed to read latest pending heights for metrics");
            Vec::new()
        }
    }
}

fn collect_proven_heights(store: &PendingStore) -> Vec<NetworkHeightSample> {
    match store.get_current_proven_height() {
        Ok(pointers) => pointers
            .into_iter()
            .map(
                |ProvenCertificate(_, network_id, height)| NetworkHeightSample {
                    network_id: network_id.to_u32(),
                    height: height.as_u64(),
                },
            )
            .collect(),
        Err(error) => {
            warn!(?error, "Failed to read latest proven heights for metrics");
            Vec::new()
        }
    }
}

fn collect_settled_heights(store: &StateStore) -> Vec<NetworkHeightSample> {
    match store.get_current_settled_height() {
        Ok(pointers) => pointers
            .into_iter()
            .map(
                |(network_id, SettledCertificate(_, height, _, _))| NetworkHeightSample {
                    network_id: network_id.to_u32(),
                    height: height.as_u64(),
                },
            )
            .collect(),
        Err(error) => {
            warn!(?error, "Failed to read latest settled heights for metrics");
            Vec::new()
        }
    }
}

/// A network is flagged in-error when its latest pending certificate header
/// has the `InError` status — the same definition `get_network_info` uses
/// for `NetworkStatus::Error`.
///
/// Unlike `get_network_info` (where `Disabled` takes precedence over
/// `Error`), this metric reports storage truth and does not consult the
/// disabled-networks list: a disabled network with an in-error pending
/// certificate still exports `in_error=1`, and the height gauges keep
/// exporting for disabled networks. Alert authors must account for
/// disabled networks separately.
fn collect_error_flags(
    pending_store: &PendingStore,
    state_store: &StateStore,
) -> Vec<NetworkErrorSample> {
    let pointers = match pending_store.get_current_pending_heights() {
        Ok(pointers) => pointers,
        Err(error) => {
            warn!(
                ?error,
                "Failed to read latest pending certificates for error metrics"
            );
            return Vec::new();
        }
    };

    pointers
        .into_iter()
        .filter_map(|(network_id, PendingCertificate(certificate_id, _))| {
            match state_store.get_certificate_header(&certificate_id) {
                Ok(Some(header)) => Some(NetworkErrorSample {
                    network_id: network_id.to_u32(),
                    in_error: matches!(header.status, CertificateStatus::InError { .. }),
                }),
                Ok(None) => {
                    warn!(
                        %network_id,
                        %certificate_id,
                        "Latest pending certificate has no header, skipping error metric"
                    );
                    None
                }
                Err(error) => {
                    warn!(
                        ?error,
                        %network_id,
                        %certificate_id,
                        "Failed to read certificate header for error metric"
                    );
                    None
                }
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use agglayer_storage::{
        backup::BackupClient,
        stores::{PendingCertificateWriter as _, StateWriter as _},
        tests::TempDBDir,
    };
    use agglayer_telemetry::testutils::MetricsHarness;
    use agglayer_types::{
        Certificate, CertificateIndex, CertificateStatus, CertificateStatusError, EpochNumber,
        Height, NetworkId,
    };

    use super::*;

    fn stores() -> (TempDBDir, TempDBDir, PendingStore, StateStore) {
        let pending_tmp = TempDBDir::new();
        let state_tmp = TempDBDir::new();
        let pending_store = PendingStore::new_with_path(&pending_tmp.path).unwrap();
        let state_store = StateStore::new_with_path(&state_tmp.path, BackupClient::noop()).unwrap();
        (pending_tmp, state_tmp, pending_store, state_store)
    }

    #[test]
    fn pending_heights_reflect_latest_pointer() {
        let (_pending_tmp, _state_tmp, pending_store, _state_store) = stores();
        let network_id = NetworkId::new(2);
        let certificate = Certificate::new_for_test(network_id, Height::ZERO);

        assert_eq!(collect_pending_heights(&pending_store), vec![]);

        pending_store
            .insert_pending_certificate(network_id, Height::ZERO, &certificate)
            .unwrap();

        assert_eq!(
            collect_pending_heights(&pending_store),
            vec![NetworkHeightSample {
                network_id: 2,
                height: 0
            }],
        );
    }

    #[test]
    fn proven_heights_reflect_latest_pointer() {
        let (_pending_tmp, _state_tmp, pending_store, _state_store) = stores();
        let network_id = NetworkId::new(2);
        let certificate = Certificate::new_for_test(network_id, Height::new(7));

        assert_eq!(collect_proven_heights(&pending_store), vec![]);

        pending_store
            .set_latest_proven_certificate_per_network(
                &network_id,
                &Height::new(7),
                &certificate.hash(),
            )
            .unwrap();

        assert_eq!(
            collect_proven_heights(&pending_store),
            vec![NetworkHeightSample {
                network_id: 2,
                height: 7
            }],
        );
    }

    #[test]
    fn settled_heights_reflect_latest_pointer() {
        let (_pending_tmp, _state_tmp, _pending_store, state_store) = stores();
        let network_id = NetworkId::new(2);
        let certificate = Certificate::new_for_test(network_id, Height::ZERO);

        assert_eq!(collect_settled_heights(&state_store), vec![]);

        state_store
            .set_latest_settled_certificate_for_network(
                &network_id,
                &Height::ZERO,
                &certificate.hash(),
                &EpochNumber::ZERO,
                &CertificateIndex::ZERO,
            )
            .unwrap();

        assert_eq!(
            collect_settled_heights(&state_store),
            vec![NetworkHeightSample {
                network_id: 2,
                height: 0
            }],
        );
    }

    #[test]
    fn error_flags_track_latest_pending_certificate_status() {
        let (_pending_tmp, _state_tmp, pending_store, state_store) = stores();

        // Network 2: latest pending certificate is in error.
        let errored_network = NetworkId::new(2);
        let errored_certificate = Certificate::new_for_test(errored_network, Height::ZERO);
        pending_store
            .insert_pending_certificate(errored_network, Height::ZERO, &errored_certificate)
            .unwrap();
        state_store
            .insert_certificate_header(
                &errored_certificate,
                CertificateStatus::error(CertificateStatusError::InternalError("test".to_string())),
            )
            .unwrap();

        // Network 3: latest pending certificate is healthy.
        let healthy_network = NetworkId::new(3);
        let healthy_certificate = Certificate::new_for_test(healthy_network, Height::ZERO);
        pending_store
            .insert_pending_certificate(healthy_network, Height::ZERO, &healthy_certificate)
            .unwrap();
        state_store
            .insert_certificate_header(&healthy_certificate, CertificateStatus::Pending)
            .unwrap();

        // Network 4: pointer exists but the header is missing; skipped.
        let headerless_network = NetworkId::new(4);
        let headerless_certificate = Certificate::new_for_test(headerless_network, Height::ZERO);
        pending_store
            .insert_pending_certificate(headerless_network, Height::ZERO, &headerless_certificate)
            .unwrap();

        assert_eq!(
            collect_error_flags(&pending_store, &state_store),
            vec![
                NetworkErrorSample {
                    network_id: 2,
                    in_error: true
                },
                NetworkErrorSample {
                    network_id: 3,
                    in_error: false
                },
            ],
        );
    }

    #[test]
    fn registration_does_not_extend_store_lifetime() {
        // A real meter provider must be installed, otherwise the no-op meter
        // discards the callbacks and this test cannot observe retention. The
        // harness owns the process-global meter provider and relies on
        // nextest's process-per-test isolation.
        let harness = MetricsHarness::install();

        let pending_tmp = TempDBDir::new();
        let state_tmp = TempDBDir::new();
        let pending_store = Arc::new(PendingStore::new_with_path(&pending_tmp.path).unwrap());
        let state_store =
            Arc::new(StateStore::new_with_path(&state_tmp.path, BackupClient::noop()).unwrap());

        register_network_state_metrics(&pending_store, &state_store);

        // Positive control: the registration must land on the provider this
        // test installed, otherwise the retention check below is vacuous.
        let network_id = NetworkId::new(2);
        let certificate = Certificate::new_for_test(network_id, Height::ZERO);
        pending_store
            .insert_pending_certificate(network_id, Height::ZERO, &certificate)
            .unwrap();
        let metrics = harness.gather();
        assert!(
            metrics.lines().any(|line| {
                line.starts_with("agglayer_node_network_height{")
                    && line.contains("network_id=\"2\"")
                    && line.contains("stage=\"pending\"")
            }),
            "expected a pending height sample for network 2, got:\n{metrics}"
        );

        drop(pending_store);
        drop(state_store);

        // Reopening the same paths only succeeds if the registration did not
        // keep the RocksDB instances alive (RocksDB holds a lock per path).
        PendingStore::new_with_path(&pending_tmp.path).unwrap();
        StateStore::new_with_path(&state_tmp.path, BackupClient::noop()).unwrap();

        // With the original stores gone the weak upgrades fail and every
        // per-network series disappears from the scrape.
        let metrics = harness.gather();
        assert!(
            metrics
                .lines()
                .all(|line| !line.starts_with("agglayer_node_network_")),
            "expected no per-network sample lines, got:\n{metrics}"
        );
    }
}
