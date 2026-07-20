use std::time::Duration;

use agglayer_storage::tests::TempDBDir;
use agglayer_types::{CertificateId, CertificateStatus, Digest, Metadata};
use fail::FailScenario;
use integrations::{agglayer_setup::setup_network, wait_for_settlement_or_error};
use jsonrpsee::{core::client::ClientT as _, rpc_params};
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

    // Clear the revert and submit a corrected certificate (new id -> fresh job).
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
