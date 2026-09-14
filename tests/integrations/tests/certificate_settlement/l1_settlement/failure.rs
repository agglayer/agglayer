use std::time::Duration;

use agglayer_storage::tests::TempDBDir;
use agglayer_types::{CertificateId, CertificateStatus, Digest, Metadata, SettlementJobId};
use fail::FailScenario;
use integrations::{
    agglayer_setup::{setup_network, setup_network_with_config},
    wait_for_settlement_or_error,
};
use jsonrpsee::{core::client::ClientT as _, http_client::HttpClientBuilder, rpc_params};
use pessimistic_proof_test_suite::forest::Forest;
use rstest::rstest;
use tokio_util::sync::CancellationToken;

/// A settlement transaction that reverts on-chain (receipt status 0) drives the
/// certificate to `InError`. A revert is terminal in the nonce-based service.
#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(180))]
#[case::type_0_ecdsa(crate::common::type_0_ecdsa_forest())]
async fn transaction_with_receipt_status_0(#[case] state: Forest) {
    let tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();
    let cancellation_token = CancellationToken::new();

    fail::cfg("settlement::force_revert", "return").expect("Failed to configure failpoint");

    // L1 is a RAII guard
    let (agglayer_shutdowned, _l1, client) =
        setup_network(&tmp_dir.path, None, Some(cancellation_token.clone())).await;

    let withdrawals = vec![];
    let certificate = state.clone().apply_events(&[], &withdrawals);

    let certificate_id: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate])
        .await
        .unwrap();

    let result = wait_for_settlement_or_error!(client, certificate_id).await;

    assert!(matches!(result.status, CertificateStatus::InError { .. }));

    cancellation_token.cancel();
    _ = agglayer_shutdowned.await;

    scenario.teardown();
}

/// A settlement revert drives the certificate to `InError`. The fix is a
/// *corrected* certificate: a new id gets a fresh settlement job and settles.
/// Re-sending the *same* cert can't recover -- the at-most-once job guard
/// rejects a second job for the same id, and a deterministic revert would just
/// repeat anyway. An empty cert's only id-affecting free field is `metadata`,
/// so bumping it stands in for corrected content.
#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(180))]
#[case::type_0_ecdsa(crate::common::type_0_ecdsa_forest())]
async fn transaction_with_receipt_status_0_retry(#[case] state: Forest) {
    let tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();
    let cancellation_token = CancellationToken::new();

    fail::cfg("settlement::force_revert", "return").expect("Failed to configure failpoint");

    // L1 is a RAII guard
    let (agglayer_shutdowned, _l1, client) =
        setup_network(&tmp_dir.path, None, Some(cancellation_token.clone())).await;

    let withdrawals = vec![];
    let certificate = state.clone().apply_events(&[], &withdrawals);
    let certificate_id: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate])
        .await
        .unwrap();
    let result = wait_for_settlement_or_error!(client, certificate_id).await;
    assert!(matches!(result.status, CertificateStatus::InError { .. }));

    // Clear the revert and submit a corrected certificate (new id -> fresh
    // job).
    fail::cfg("settlement::force_revert", "off").expect("Failed to configure failpoint");

    let mut corrected = state.clone().apply_events(&[], &withdrawals);
    corrected.metadata = Metadata::new(Digest::from([1u8; 32]));
    let corrected_id: CertificateId = client
        .request("interop_sendCertificate", rpc_params![corrected])
        .await
        .unwrap();
    let result = wait_for_settlement_or_error!(client, corrected_id).await;
    assert!(matches!(result.status, CertificateStatus::Settled));

    cancellation_token.cancel();
    _ = agglayer_shutdowned.await;

    scenario.teardown();
}

/// The operator escape hatch for a re-sent certificate stuck behind its
/// reverted job. After a revert, the aggsender re-sends the *same* certificate
/// (the id covers the state transition, not the proof). The replacement is
/// accepted, but the certificate still points at the reverted job, so the
/// fresh job cannot be persisted and the certificate errors again. Unlinking
/// the certificate from the reverted job through the admin API lets the next
/// re-submission get a fresh job and settle.
#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(180))]
#[case::type_0_ecdsa(crate::common::type_0_ecdsa_forest())]
async fn transaction_with_receipt_status_0_admin_unlink_then_resend(#[case] state: Forest) {
    let tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();
    let cancellation_token = CancellationToken::new();

    fail::cfg("settlement::force_revert", "return").expect("Failed to configure failpoint");

    // L1 is a RAII guard
    let (agglayer_shutdowned, _l1, client, config) =
        setup_network_with_config(&tmp_dir.path, None, Some(cancellation_token.clone())).await;
    let admin_client = HttpClientBuilder::default()
        .build(format!("http://{}/", config.admin_rpc_addr()))
        .expect("Failed to build the admin client");

    let withdrawals = vec![];
    let certificate = state.clone().apply_events(&[], &withdrawals);
    let certificate_id: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate.clone()])
        .await
        .unwrap();
    let result = wait_for_settlement_or_error!(client, certificate_id).await;
    assert!(matches!(result.status, CertificateStatus::InError { .. }));

    // The cause of the revert is gone, but re-sending the same certificate
    // still fails: the certificate is linked to the reverted job.
    fail::cfg("settlement::force_revert", "off").expect("Failed to configure failpoint");
    let resent_id: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate.clone()])
        .await
        .expect("re-sending a certificate whose settlement reverted must be accepted");
    assert_eq!(resent_id, certificate_id, "same certificate, same id");
    let result = wait_for_settlement_or_error!(client, certificate_id).await;
    let CertificateStatus::InError { error } = &result.status else {
        panic!("expected InError, got {:?}", result.status);
    };
    assert!(
        format!("{error:?}").contains("Failed to persist settlement job"),
        "{error:?}"
    );

    // Unlink, then the very same certificate settles.
    let _unlinked_job_id: SettlementJobId = admin_client
        .request(
            "admin_unlinkCertificateSettlementJob",
            rpc_params![certificate_id],
        )
        .await
        .expect("unlinking a certificate from its reverted job must succeed");
    let resent_id: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate])
        .await
        .expect("re-sending after the unlink must be accepted");
    assert_eq!(resent_id, certificate_id, "same certificate, same id");
    let result = wait_for_settlement_or_error!(client, certificate_id).await;
    assert!(
        matches!(result.status, CertificateStatus::Settled),
        "{:?}",
        result.status
    );
    assert!(result.settlement_tx_hash.is_some());

    cancellation_token.cancel();
    _ = agglayer_shutdowned.await;

    scenario.teardown();
}

/// A settlement transaction whose receipt is transiently unavailable (indexing
/// lag after inclusion) is re-polled, not given up on: the certificate settles
/// once the receipt appears. (The old "no receipt -> InError" give-up is gone.)
#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(180))]
#[case::type_0_ecdsa(crate::common::type_0_ecdsa_forest())]
async fn transaction_without_receipt_settles(#[case] state: Forest) {
    let tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();
    let cancellation_token = CancellationToken::new();

    fail::cfg("settlement::receipt_transiently_unavailable", "3*return")
        .expect("Failed to configure failpoint");

    // L1 is a RAII guard
    let (agglayer_shutdowned, _l1, client) =
        setup_network(&tmp_dir.path, None, Some(cancellation_token.clone())).await;

    let withdrawals = vec![];
    let certificate = state.clone().apply_events(&[], &withdrawals);

    let certificate_id: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate])
        .await
        .unwrap();

    let result = wait_for_settlement_or_error!(client, certificate_id).await;

    assert!(matches!(result.status, CertificateStatus::Settled));

    cancellation_token.cancel();
    _ = agglayer_shutdowned.await;

    scenario.teardown();
}

/// The nonce-based service has no "too many settlement transactions" cap: it
/// keeps resubmitting for the nonce until one is included, so many
/// non-inclusion cycles still end `Settled`. (The old too-many-txs -> InError
/// give-up is gone.)
#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(180))]
#[case::type_0_ecdsa(crate::common::type_0_ecdsa_forest())]
async fn transaction_with_receipt_timeout_many_times_settles(#[case] state: Forest) {
    let tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();
    let cancellation_token = CancellationToken::new();

    fail::cfg("settlement::tx_not_included", "5*return").expect("Failed to configure failpoint");

    // L1 is a RAII guard
    let (agglayer_shutdowned, _l1, client) =
        setup_network(&tmp_dir.path, None, Some(cancellation_token.clone())).await;

    let withdrawals = vec![];
    let certificate = state.clone().apply_events(&[], &withdrawals);

    let certificate_id: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate])
        .await
        .unwrap();

    let result = wait_for_settlement_or_error!(client, certificate_id).await;

    assert!(matches!(result.status, CertificateStatus::Settled));

    cancellation_token.cancel();
    _ = agglayer_shutdowned.await;

    scenario.teardown();
}

/// A settlement transaction that is not included on L1 for a couple of cycles
/// is resubmitted by the service and eventually settles.
#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(180))]
#[case::type_0_ecdsa(crate::common::type_0_ecdsa_forest())]
async fn transaction_with_receipt_timeout_2_times(#[case] state: Forest) {
    let tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();
    let cancellation_token = CancellationToken::new();

    fail::cfg("settlement::tx_not_included", "2*return").expect("Failed to configure failpoint");

    // L1 is a RAII guard
    let (agglayer_shutdowned, _l1, client) =
        setup_network(&tmp_dir.path, None, Some(cancellation_token.clone())).await;

    let withdrawals = vec![];
    let certificate = state.clone().apply_events(&[], &withdrawals);

    let certificate_id: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate])
        .await
        .unwrap();

    let result = wait_for_settlement_or_error!(client, certificate_id).await;

    assert!(matches!(result.status, CertificateStatus::Settled));

    cancellation_token.cancel();
    _ = agglayer_shutdowned.await;

    scenario.teardown();
}
