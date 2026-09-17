use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread,
};

use agglayer_types::{
    Certificate, CertificateId, CertificateIndex, CertificateStatus, CertificateStatusError,
    Digest, EpochNumber, Height, NetworkId, SettlementTxHash,
};
use prometheus::{Encoder as _, Registry, TextEncoder};

use super::{NetworkMetrics, NETWORK_HEIGHT, NETWORK_LATEST_CERTIFICATE_IN_ERROR};
use crate::{
    backup::BackupClient,
    storage::DB,
    stores::{
        pending::PendingStore, state::StateStore, PendingCertificateReader as _,
        PendingCertificateWriter as _, StateWriter as _, UpdateEvenIfAlreadyPresent,
        UpdateStatusToCandidate,
    },
    tests::TempDBDir,
};

fn network_metrics() -> (Registry, NetworkMetrics) {
    let registry = Registry::new();
    let metrics = NetworkMetrics::new(&registry).unwrap();
    (registry, metrics)
}

fn stores(metrics: &NetworkMetrics) -> (TempDBDir, TempDBDir, PendingStore, StateStore) {
    let pending_dir = TempDBDir::new();
    let state_dir = TempDBDir::new();
    let pending =
        PendingStore::new_with_path_and_metrics(&pending_dir.path, metrics.clone()).unwrap();
    let state = StateStore::new_with_path_and_metrics(
        &state_dir.path,
        BackupClient::noop(),
        metrics.clone(),
    )
    .unwrap();
    (pending_dir, state_dir, pending, state)
}

fn gather(registry: &Registry) -> String {
    let mut buffer = Vec::new();
    TextEncoder::new()
        .encode(&registry.gather(), &mut buffer)
        .unwrap();
    String::from_utf8(buffer).unwrap()
}

/// Extract the raw value text of the sample line for `metric`, `network_id`
/// and, when given, the `stage` label.
fn sample_text<'a>(
    body: &'a str,
    metric: &str,
    network_id: u32,
    stage: Option<&str>,
) -> Option<&'a str> {
    let prefix = format!("{metric}{{");
    let network_label = format!("network_id=\"{network_id}\"");
    let stage_label = stage.map(|stage| format!("stage=\"{stage}\""));
    body.lines()
        .find(|line| {
            line.starts_with(&prefix)
                && line.contains(&network_label)
                && stage_label
                    .as_ref()
                    .is_none_or(|label| line.contains(label))
        })
        .and_then(|line| line.rsplit(' ').next())
}

/// Extract the value of the sample line for `metric`, `network_id` and, when
/// given, the `stage` label.
fn sample_value(body: &str, metric: &str, network_id: u32, stage: Option<&str>) -> Option<i64> {
    sample_text(body, metric, network_id, stage)?.parse().ok()
}

fn height_of(registry: &Registry, network_id: u32, stage: &str) -> Option<i64> {
    sample_value(&gather(registry), NETWORK_HEIGHT, network_id, Some(stage))
}

/// Whether the exported height of `network_id` at `stage` is the saturated
/// gauge maximum. The text format renders the `i64` gauge through `f64`, so
/// `i64::MAX` shows up as `2^63`, which no integer parse accepts.
fn height_is_saturated(registry: &Registry, network_id: u32, stage: &str) -> bool {
    sample_text(&gather(registry), NETWORK_HEIGHT, network_id, Some(stage))
        == Some((i64::MAX as f64).to_string().as_str())
}

fn error_of(registry: &Registry, network_id: u32) -> Option<i64> {
    sample_value(
        &gather(registry),
        NETWORK_LATEST_CERTIFICATE_IN_ERROR,
        network_id,
        None,
    )
}

fn in_error_status() -> CertificateStatus {
    CertificateStatus::error(CertificateStatusError::InternalError("test".to_string()))
}

/// Minimal subscriber counting the `WARN` events of the metrics module.
#[derive(Clone, Default)]
struct WarningCounter(Arc<AtomicUsize>);

impl WarningCounter {
    fn count(&self) -> usize {
        self.0.load(Ordering::SeqCst)
    }

    /// Run `f` with this counter installed as the thread's subscriber.
    fn observe<T>(&self, f: impl FnOnce() -> T) -> T {
        tracing::subscriber::with_default(self.clone(), f)
    }
}

impl tracing::Subscriber for WarningCounter {
    fn enabled(&self, metadata: &tracing::Metadata<'_>) -> bool {
        *metadata.level() == tracing::Level::WARN
            && metadata.target() == "agglayer_storage::network_metrics"
    }

    fn new_span(&self, _: &tracing::span::Attributes<'_>) -> tracing::span::Id {
        tracing::span::Id::from_u64(1)
    }

    fn record(&self, _: &tracing::span::Id, _: &tracing::span::Record<'_>) {}

    fn record_follows_from(&self, _: &tracing::span::Id, _: &tracing::span::Id) {}

    fn event(&self, _: &tracing::Event<'_>) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }

    fn enter(&self, _: &tracing::span::Id) {}

    fn exit(&self, _: &tracing::span::Id) {}
}

#[test]
fn duplicate_registration_is_rejected() {
    let (registry, _metrics) = network_metrics();
    assert!(NetworkMetrics::new(&registry).is_err());
}

#[test]
fn store_mutations_update_gauges_after_each_successful_write() {
    let (registry, metrics) = network_metrics();
    let (_pending_dir, _state_dir, pending, state) = stores(&metrics);
    let network = NetworkId::new(1);
    let certificate = Certificate::new_for_test(network, Height::ZERO);
    let certificate_id = certificate.hash();

    // Height zero is a real exported value; no header yet means no error
    // series, not a zero placeholder.
    pending
        .insert_pending_certificate(network, Height::ZERO, &certificate)
        .unwrap();
    assert_eq!(height_of(&registry, 1, "pending"), Some(0));
    assert_eq!(error_of(&registry, 1), None);

    state
        .insert_certificate_header(&certificate, CertificateStatus::Pending)
        .unwrap();
    assert_eq!(error_of(&registry, 1), Some(0));

    state
        .update_settlement_tx_hash(
            &certificate_id,
            SettlementTxHash::new(Digest::from([1_u8; 32])),
            UpdateEvenIfAlreadyPresent::No,
            UpdateStatusToCandidate::Yes,
        )
        .unwrap();
    assert_eq!(error_of(&registry, 1), Some(0));

    state
        .update_certificate_header_status(&certificate_id, &in_error_status())
        .unwrap();
    assert_eq!(error_of(&registry, 1), Some(1));

    // A status-preserving settlement-hash rewrite republishes the stored
    // status instead of clobbering the error flag.
    state
        .update_settlement_tx_hash(
            &certificate_id,
            SettlementTxHash::new(Digest::from([2_u8; 32])),
            UpdateEvenIfAlreadyPresent::Yes,
            UpdateStatusToCandidate::No,
        )
        .unwrap();
    assert_eq!(error_of(&registry, 1), Some(1));

    state.remove_settlement_tx_hash(&certificate_id).unwrap();
    assert_eq!(error_of(&registry, 1), Some(1));

    state
        .update_certificate_header_status(&certificate_id, &CertificateStatus::Proven)
        .unwrap();
    assert_eq!(error_of(&registry, 1), Some(0));

    pending
        .set_latest_proven_certificate_per_network(&network, &Height::ZERO, &certificate_id)
        .unwrap();
    assert_eq!(height_of(&registry, 1, "proven"), Some(0));

    state
        .assign_certificate_to_epoch(&certificate_id, &EpochNumber::ZERO, &CertificateIndex::ZERO)
        .unwrap();
    assert_eq!(error_of(&registry, 1), Some(0));

    state
        .set_latest_settled_certificate_for_network(
            &network,
            &Height::ZERO,
            &certificate_id,
            &EpochNumber::ZERO,
            &CertificateIndex::ZERO,
        )
        .unwrap();
    assert_eq!(height_of(&registry, 1, "settled"), Some(0));
}

#[test]
fn header_writes_for_non_current_certificates_leave_the_error_gauge() {
    let (registry, metrics) = network_metrics();
    let (_pending_dir, _state_dir, pending, state) = stores(&metrics);
    let network = NetworkId::new(1);
    let current = Certificate::new_for_test(network, Height::ZERO);
    let other = Certificate::new_for_test(network, Height::new(1));

    pending
        .insert_pending_certificate(network, Height::ZERO, &current)
        .unwrap();
    state
        .insert_certificate_header(&current, CertificateStatus::Pending)
        .unwrap();
    assert_eq!(error_of(&registry, 1), Some(0));

    // `other` is not the current pending pointer, so its errored header must
    // not flip the network's error gauge.
    state
        .insert_certificate_header(&other, in_error_status())
        .unwrap();
    assert_eq!(error_of(&registry, 1), Some(0));

    // A network without any pending pointer never gains an error series.
    let unrelated = Certificate::new_for_test(NetworkId::new(2), Height::ZERO);
    state
        .insert_certificate_header(&unrelated, in_error_status())
        .unwrap();
    assert_eq!(error_of(&registry, 2), None);
}

#[test]
fn headerless_pointer_omits_error_series_until_the_header_commits() {
    let (registry, metrics) = network_metrics();
    let (_pending_dir, _state_dir, pending, state) = stores(&metrics);
    let network = NetworkId::new(2);
    let certificate = Certificate::new_for_test(network, Height::new(4));

    pending
        .set_latest_pending_certificate_per_network(&network, &Height::new(4), &certificate.hash())
        .unwrap();
    assert_eq!(height_of(&registry, 2, "pending"), Some(4));
    assert_eq!(error_of(&registry, 2), None);

    state
        .insert_certificate_header(&certificate, CertificateStatus::Pending)
        .unwrap();
    assert_eq!(error_of(&registry, 2), Some(0));
}

#[test]
fn new_pending_pointer_clears_the_stale_error_series() {
    let (registry, metrics) = network_metrics();
    let (_pending_dir, _state_dir, pending, state) = stores(&metrics);
    let network = NetworkId::new(1);
    let errored = Certificate::new_for_test(network, Height::ZERO);

    pending
        .insert_pending_certificate(network, Height::ZERO, &errored)
        .unwrap();
    state
        .insert_certificate_header(&errored, in_error_status())
        .unwrap();
    assert_eq!(error_of(&registry, 1), Some(1));

    // The next pointer has no header yet: the previous certificate's error
    // flag no longer describes the latest certificate, so the series must
    // disappear rather than keep the stale value.
    pending
        .set_latest_pending_certificate_per_network(
            &network,
            &Height::new(1),
            &CertificateId::new([7; 32].into()),
        )
        .unwrap();
    assert_eq!(height_of(&registry, 1, "pending"), Some(1));
    assert_eq!(error_of(&registry, 1), None);
}

/// Regression test for agglayer/issues#26: bali's pending pointers carried
/// heights above `i64::MAX` and v0.6.1-rc.1 refused to start on them.
#[test]
fn heights_beyond_the_gauge_range_saturate_and_warn_at_hydration() {
    let pending_dir = TempDBDir::new();
    let state_dir = TempDBDir::new();
    // The exact pointer that took bali down.
    let network = NetworkId::new(84_803_420);
    let oversized = Height::new(13_521_720_084_024_907_505);
    let certificate_id = CertificateId::new([7; 32].into());
    let healthy = Certificate::new_for_test(NetworkId::new(1), Height::new(5));

    {
        let (registry, metrics) = network_metrics();
        let pending =
            PendingStore::new_with_path_and_metrics(&pending_dir.path, metrics.clone()).unwrap();
        let state = StateStore::new_with_path_and_metrics(
            &state_dir.path,
            BackupClient::noop(),
            metrics.clone(),
        )
        .unwrap();

        // Every write goes through and persists the real height; only the
        // exported gauge saturates, and no write logs.
        let warnings = WarningCounter::default();
        warnings.observe(|| {
            pending
                .set_latest_pending_certificate_per_network(&network, &oversized, &certificate_id)
                .unwrap();
            pending
                .set_latest_proven_certificate_per_network(&network, &oversized, &certificate_id)
                .unwrap();
            state
                .set_latest_settled_certificate_for_network(
                    &network,
                    &oversized,
                    &certificate_id,
                    &EpochNumber::ZERO,
                    &CertificateIndex::ZERO,
                )
                .unwrap();
        });
        assert_eq!(
            pending
                .get_latest_pending_certificate_for_network(&network)
                .unwrap(),
            Some((certificate_id, oversized))
        );
        for stage in ["pending", "proven", "settled"] {
            assert!(height_is_saturated(&registry, 84_803_420, stage), "{stage}");
        }
        assert_eq!(warnings.count(), 0);

        pending
            .insert_pending_certificate(NetworkId::new(1), Height::new(5), &healthy)
            .unwrap();
    }

    // Restart: hydration used to abort node startup on the stored pointer.
    let (registry, metrics) = network_metrics();
    let pending =
        PendingStore::new_with_path_and_metrics(&pending_dir.path, metrics.clone()).unwrap();
    let state = StateStore::new_with_path_and_metrics(
        &state_dir.path,
        BackupClient::noop(),
        metrics.clone(),
    )
    .unwrap();
    let warnings = WarningCounter::default();
    warnings.observe(|| metrics.hydrate(&pending, &state).unwrap());

    for stage in ["pending", "proven", "settled"] {
        assert!(height_is_saturated(&registry, 84_803_420, stage), "{stage}");
    }
    assert_eq!(height_of(&registry, 1, "pending"), Some(5));
    assert_eq!(warnings.count(), 1);

    // Only hydration reports: a later oversized write stays silent.
    warnings.observe(|| {
        pending
            .set_latest_proven_certificate_per_network(
                &network,
                &Height::new(u64::MAX),
                &certificate_id,
            )
            .unwrap();
    });
    assert!(height_is_saturated(&registry, 84_803_420, "proven"));
    assert_eq!(warnings.count(), 1);
}

#[test]
fn hydrate_seeds_every_series_from_a_storage_snapshot() {
    let pending_dir = TempDBDir::new();
    let state_dir = TempDBDir::new();

    {
        let seeding_metrics = NetworkMetrics::unregistered();
        let pending =
            PendingStore::new_with_path_and_metrics(&pending_dir.path, seeding_metrics.clone())
                .unwrap();
        let state = StateStore::new_with_path_and_metrics(
            &state_dir.path,
            BackupClient::noop(),
            seeding_metrics,
        )
        .unwrap();

        // Healthy network at height zero.
        let healthy = Certificate::new_for_test(NetworkId::new(1), Height::ZERO);
        pending
            .insert_pending_certificate(NetworkId::new(1), Height::ZERO, &healthy)
            .unwrap();
        state
            .insert_certificate_header(&healthy, CertificateStatus::Pending)
            .unwrap();

        // Errored network.
        let errored = Certificate::new_for_test(NetworkId::new(2), Height::new(5));
        pending
            .insert_pending_certificate(NetworkId::new(2), Height::new(5), &errored)
            .unwrap();
        state
            .insert_certificate_header(&errored, in_error_status())
            .unwrap();

        // Pointer without a header.
        pending
            .set_latest_pending_certificate_per_network(
                &NetworkId::new(3),
                &Height::new(7),
                &CertificateId::new([3; 32].into()),
            )
            .unwrap();

        // Proven-only and settled-only networks.
        pending
            .set_latest_proven_certificate_per_network(
                &NetworkId::new(4),
                &Height::new(3),
                &CertificateId::new([4; 32].into()),
            )
            .unwrap();
        state
            .set_latest_settled_certificate_for_network(
                &NetworkId::new(5),
                &Height::new(9),
                &CertificateId::new([5; 32].into()),
                &EpochNumber::ZERO,
                &CertificateIndex::ZERO,
            )
            .unwrap();
    }

    // Restart: fresh registry and metrics, reopened stores, one hydration.
    let (registry, metrics) = network_metrics();
    let pending =
        PendingStore::new_with_path_and_metrics(&pending_dir.path, metrics.clone()).unwrap();
    let state = StateStore::new_with_path_and_metrics(
        &state_dir.path,
        BackupClient::noop(),
        metrics.clone(),
    )
    .unwrap();
    metrics.hydrate(&pending, &state).unwrap();

    let body = gather(&registry);
    assert_eq!(
        sample_value(&body, NETWORK_HEIGHT, 1, Some("pending")),
        Some(0)
    );
    assert_eq!(
        sample_value(&body, NETWORK_LATEST_CERTIFICATE_IN_ERROR, 1, None),
        Some(0)
    );
    assert_eq!(
        sample_value(&body, NETWORK_HEIGHT, 2, Some("pending")),
        Some(5)
    );
    assert_eq!(
        sample_value(&body, NETWORK_LATEST_CERTIFICATE_IN_ERROR, 2, None),
        Some(1)
    );
    assert_eq!(
        sample_value(&body, NETWORK_HEIGHT, 3, Some("pending")),
        Some(7)
    );
    assert_eq!(
        sample_value(&body, NETWORK_LATEST_CERTIFICATE_IN_ERROR, 3, None),
        None
    );
    assert_eq!(
        sample_value(&body, NETWORK_HEIGHT, 4, Some("proven")),
        Some(3)
    );
    assert_eq!(
        sample_value(&body, NETWORK_HEIGHT, 4, Some("pending")),
        None
    );
    assert_eq!(
        sample_value(&body, NETWORK_LATEST_CERTIFICATE_IN_ERROR, 4, None),
        None
    );
    assert_eq!(
        sample_value(&body, NETWORK_HEIGHT, 5, Some("settled")),
        Some(9)
    );
    assert_eq!(
        sample_value(&body, NETWORK_LATEST_CERTIFICATE_IN_ERROR, 5, None),
        None
    );
}

#[test]
fn failed_writes_leave_gauges_unchanged() {
    let (registry, metrics) = network_metrics();
    let pending_dir = TempDBDir::new();
    let state_dir = TempDBDir::new();
    let network = NetworkId::new(1);
    let certificate = Certificate::new_for_test(network, Height::ZERO);
    let certificate_id = certificate.hash();

    {
        let pending =
            PendingStore::new_with_path_and_metrics(&pending_dir.path, metrics.clone()).unwrap();
        let state = StateStore::new_with_path_and_metrics(
            &state_dir.path,
            BackupClient::noop(),
            metrics.clone(),
        )
        .unwrap();
        pending
            .insert_pending_certificate(network, Height::ZERO, &certificate)
            .unwrap();
        state
            .insert_certificate_header(&certificate, CertificateStatus::Pending)
            .unwrap();
        state
            .set_latest_settled_certificate_for_network(
                &network,
                &Height::ZERO,
                &certificate_id,
                &EpochNumber::ZERO,
                &CertificateIndex::ZERO,
            )
            .unwrap();
    }
    let body_before = gather(&registry);

    // Reopen both databases read-only: every mutation now fails at the
    // RocksDB layer, after the shared guard is taken but before any gauge
    // update.
    let pending = PendingStore::new_with_metrics(
        Arc::new(
            DB::open_cf_readonly(
                &pending_dir.path,
                crate::stores::pending::cf_definitions::PENDING_DB,
            )
            .unwrap(),
        ),
        metrics.clone(),
    );
    let state = StateStore::new_with_metrics(
        Arc::new(
            DB::open_cf_readonly(
                &state_dir.path,
                crate::stores::state::cf_definitions::STATE_DB,
            )
            .unwrap(),
        ),
        BackupClient::noop(),
        metrics.clone(),
    );

    assert!(pending
        .set_latest_pending_certificate_per_network(
            &network,
            &Height::new(9),
            &CertificateId::new([9; 32].into()),
        )
        .is_err());
    assert!(pending
        .set_latest_proven_certificate_per_network(&network, &Height::new(9), &certificate_id)
        .is_err());
    assert!(state
        .update_certificate_header_status(&certificate_id, &in_error_status())
        .is_err());
    assert!(state
        .set_latest_settled_certificate_for_network(
            &network,
            &Height::new(9),
            &certificate_id,
            &EpochNumber::ZERO,
            &CertificateIndex::ZERO,
        )
        .is_err());

    assert_eq!(gather(&registry), body_before);
}

#[test]
fn reconcile_pending_error_publishes_only_the_current_pointer() {
    let (registry, metrics) = network_metrics();
    let (_pending_dir, _state_dir, pending, state) = stores(&metrics);
    let network = NetworkId::new(1);
    let stale = Certificate::new_for_test(network, Height::ZERO);
    let current = Certificate::new_for_test(network, Height::new(1));

    // Headers exist before any pointer, so neither insertion publishes.
    state
        .insert_certificate_header(&stale, in_error_status())
        .unwrap();
    state
        .insert_certificate_header(&current, CertificateStatus::Pending)
        .unwrap();
    pending
        .set_latest_pending_certificate_per_network(&network, &Height::ZERO, &stale.hash())
        .unwrap();
    pending
        .set_latest_pending_certificate_per_network(&network, &Height::new(1), &current.hash())
        .unwrap();

    // The pointer moved on: reconciling the stale certificate must not
    // resurrect its error status.
    metrics
        .reconcile_pending_error(&pending, &state, network, stale.hash())
        .unwrap();
    assert_eq!(error_of(&registry, 1), None);

    metrics
        .reconcile_pending_error(&pending, &state, network, current.hash())
        .unwrap();
    assert_eq!(error_of(&registry, 1), Some(0));

    // A network without any pointer reconciles to a no-op.
    metrics
        .reconcile_pending_error(&pending, &state, NetworkId::new(9), stale.hash())
        .unwrap();
    assert_eq!(error_of(&registry, 9), None);
}

#[test]
fn concurrent_same_network_writers_keep_the_error_gauge_correlated() {
    let (registry, metrics) = network_metrics();
    let (_pending_dir, _state_dir, pending, state) = stores(&metrics);
    let network = NetworkId::new(1);
    let errored = Certificate::new_for_test(network, Height::ZERO);
    let healthy = Certificate::new_for_test(network, Height::new(1));
    state
        .insert_certificate_header(&errored, in_error_status())
        .unwrap();
    state
        .insert_certificate_header(&healthy, CertificateStatus::Pending)
        .unwrap();

    // Two admin-style writers race pointer updates plus reconciliation for
    // the same network, as `admin_setLatestPendingCertificate` does.
    thread::scope(|scope| {
        for (height, certificate_id) in [
            (Height::ZERO, errored.hash()),
            (Height::new(1), healthy.hash()),
        ] {
            let (pending, state, metrics) = (&pending, &state, &metrics);
            scope.spawn(move || {
                for _ in 0..100 {
                    pending
                        .set_latest_pending_certificate_per_network(
                            &network,
                            &height,
                            &certificate_id,
                        )
                        .unwrap();
                    metrics
                        .reconcile_pending_error(pending, state, network, certificate_id)
                        .unwrap();
                }
            });
        }
    });

    // Whatever interleaving happened, the final gauge must describe the
    // final stored pointer.
    let (final_id, final_height) = pending
        .get_latest_pending_certificate_for_network(&network)
        .unwrap()
        .unwrap();
    assert_eq!(
        error_of(&registry, 1),
        Some(i64::from(final_id == errored.hash()))
    );
    assert_eq!(
        height_of(&registry, 1, "pending"),
        Some(i64::try_from(final_height.as_u64()).unwrap())
    );
}

#[test]
fn gathering_needs_no_storage_after_the_stores_drop() {
    let (registry, metrics) = network_metrics();
    let pending_dir = TempDBDir::new();
    let state_dir = TempDBDir::new();

    {
        let pending =
            PendingStore::new_with_path_and_metrics(&pending_dir.path, metrics.clone()).unwrap();
        let state = StateStore::new_with_path_and_metrics(
            &state_dir.path,
            BackupClient::noop(),
            metrics.clone(),
        )
        .unwrap();
        let certificate = Certificate::new_for_test(NetworkId::new(1), Height::ZERO);
        pending
            .insert_pending_certificate(NetworkId::new(1), Height::ZERO, &certificate)
            .unwrap();
        state
            .insert_certificate_header(&certificate, CertificateStatus::Pending)
            .unwrap();
    }

    // Reopening read-write requires the RocksDB lock, which proves the
    // metrics kept no live store handle after the stores dropped.
    let _exclusive = PendingStore::init_db(&pending_dir.path).unwrap();

    let body = gather(&registry);
    assert_eq!(
        sample_value(&body, NETWORK_HEIGHT, 1, Some("pending")),
        Some(0)
    );
    assert_eq!(
        sample_value(&body, NETWORK_LATEST_CERTIFICATE_IN_ERROR, 1, None),
        Some(0)
    );
}

#[test]
fn exposition_preserves_names_help_and_scope_label() {
    let (registry, metrics) = network_metrics();
    let (_pending_dir, _state_dir, pending, state) = stores(&metrics);
    let network = NetworkId::new(1);
    let certificate = Certificate::new_for_test(network, Height::ZERO);
    pending
        .insert_pending_certificate(network, Height::ZERO, &certificate)
        .unwrap();
    state
        .insert_certificate_header(&certificate, CertificateStatus::Pending)
        .unwrap();

    let body = gather(&registry);
    for declaration in [
        "# HELP agglayer_node_network_height Height of the latest certificate per network and \
         lifecycle stage",
        "# TYPE agglayer_node_network_height gauge",
        "# HELP agglayer_node_network_latest_certificate_in_error Whether the latest known \
         certificate of the network is in error (1) or not (0)",
        "# TYPE agglayer_node_network_latest_certificate_in_error gauge",
    ] {
        assert!(
            body.lines().any(|line| line == declaration),
            "missing metric declaration {declaration:?}, metrics body:\n{body}"
        );
    }

    let samples: Vec<&str> = body
        .lines()
        .filter(|line| line.starts_with("agglayer_node_network_") && !line.starts_with('#'))
        .collect();
    assert!(!samples.is_empty());
    for line in &samples {
        assert!(
            line.contains("otel_scope_name=\"agglayer_node_network\""),
            "network metric lost its OpenTelemetry scope label: {line}"
        );
        assert!(
            line.contains("network_id=\"1\""),
            "network metric lost its network label: {line}"
        );
    }
    assert!(samples
        .iter()
        .any(|line| line.starts_with(NETWORK_HEIGHT) && line.contains("stage=\"pending\"")));
}
