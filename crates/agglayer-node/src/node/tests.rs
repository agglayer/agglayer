use std::{sync::Arc, time::Duration};

use agglayer_config::{epoch::TimeClockConfig, Config, Epoch};
use agglayer_storage::{
    backup::BackupClient,
    columns::latest_settled_certificate_per_network::{
        LatestSettledCertificatePerNetworkColumn, SettledCertificate,
    },
    stores::{
        pending::PendingStore, state::StateStore, NetworkInfoReader as _,
        PendingCertificateWriter as _, StateWriter as _,
    },
    tests::{init_state_db_before_network_info_backfill, TempDBDir},
};
use agglayer_types::{
    Certificate, CertificateId, CertificateIndex, CertificateStatus, Digest, EpochNumber, Height,
    NetworkId,
};
use tokio_util::sync::CancellationToken;

use super::Node;

fn config(tmp: &TempDBDir) -> Config {
    let mut config = Config::new(&tmp.path);
    config.epoch = Epoch::TimeClock(TimeClockConfig {
        epoch_duration: Duration::from_secs(60),
    });
    config
}

/// Start a node without an L1, which fails at some point after storage.
async fn start(config: Config) -> eyre::Report {
    let cancellation_token = CancellationToken::new();
    let start = Node::builder()
        .config(Arc::new(config))
        .cancellation_token(cancellation_token.clone())
        .registry(prometheus::Registry::new())
        .version("test".to_string())
        .start();
    let error = tokio::time::timeout(Duration::from_secs(120), start)
        .await
        .expect("startup without an L1 must fail promptly")
        .err()
        .expect("startup without an L1 must fail");
    cancellation_token.cancel();
    error
}

/// The first start of the current binary fills network info from the latest
/// certificate indexes of a database written by an older build.
#[test_log::test(tokio::test)]
async fn first_start_backfills_network_info() {
    let tmp = TempDBDir::new();
    let config = config(&tmp);
    let certificate = Certificate::new_for_test(NetworkId::new(1), Height::ZERO);
    {
        let state = StateStore::new(
            Arc::new(
                init_state_db_before_network_info_backfill(&config.storage.state_db_path).unwrap(),
            ),
            BackupClient::noop(),
        );
        state
            .insert_certificate_header(&certificate, CertificateStatus::Pending)
            .unwrap();
        PendingStore::new_with_path(&config.storage.pending_db_path)
            .unwrap()
            .insert_pending_certificate(certificate.network_id, certificate.height, &certificate)
            .unwrap();
    }

    let state_db_path = config.storage.state_db_path.clone();
    let error = start(config).await;
    assert!(
        !error
            .chain()
            .any(|cause| cause.to_string().contains("network info")),
        "the backfill must succeed: {error:?}"
    );

    let state = StateStore::new_with_path(&state_db_path, BackupClient::noop()).unwrap();
    assert_eq!(
        state
            .get_latest_pending_certificate_id(certificate.network_id)
            .unwrap(),
        Some(certificate.hash())
    );
}

/// A failed backfill stops startup while opening storage, before the clock,
/// settlement, the orchestrator, and the RPC servers.
#[test_log::test(tokio::test)]
async fn failed_backfill_stops_startup_at_storage() {
    let tmp = TempDBDir::new();
    let config = config(&tmp);
    // A settled pointer to a certificate without a header.
    init_state_db_before_network_info_backfill(&config.storage.state_db_path)
        .unwrap()
        .put::<LatestSettledCertificatePerNetworkColumn>(
            &NetworkId::new(1),
            &SettledCertificate(
                CertificateId::new(Digest([1; 32])),
                Height::ZERO,
                EpochNumber::ZERO,
                CertificateIndex::ZERO,
            ),
        )
        .unwrap();

    let error = start(config).await;

    assert!(
        error
            .chain()
            .any(|cause| cause.to_string().contains("has no header")),
        "unexpected error: {error:?}"
    );
}
