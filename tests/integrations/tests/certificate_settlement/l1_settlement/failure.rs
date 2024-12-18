use std::time::Duration;

use agglayer_storage::tests::TempDBDir;
use agglayer_types::{CertificateId, CertificateStatus};
use fail::FailScenario;
use integrations::{
    agglayer_setup::{get_signer, setup_network},
    wait_for_settlement_or_error,
};
use jsonrpsee::core::client::ClientT as _;
use jsonrpsee::rpc_params;
use pessimistic_proof_test_suite::forest::Forest;
use rstest::rstest;

#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(180))]
async fn transaction_with_receipt_status_0() {
    let tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();

    fail::cfg(
        "notifier::packer::settle_certificate::receipt_future_ended::status_0",
        "return",
    )
    .expect("Failed to configure failpoint");

    // L1 is a RAII guard
    let (_handle, _l1, client) = setup_network(&tmp_dir.path).await;
    let signer = get_signer(0);

    let state = Forest::default().with_signer(signer);

    let withdrawals = vec![];

    let certificate = state.clone().apply_events(&[], &withdrawals);

    let certificate_id: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate])
        .await
        .unwrap();

    let result = wait_for_settlement_or_error!(client, certificate_id).await;

    assert!(matches!(result.status, CertificateStatus::InError { .. }));

    scenario.teardown();
}

#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(180))]
async fn transaction_with_receipt_status_0_retry() {
    let tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();

    fail::cfg(
        "notifier::packer::settle_certificate::receipt_future_ended::status_0",
        "return",
    )
    .expect("Failed to configure failpoint");

    // L1 is a RAII guard
    let (_handle, _l1, client) = setup_network(&tmp_dir.path).await;
    let signer = get_signer(0);

    let state = Forest::default().with_signer(signer);

    let withdrawals = vec![];

    let certificate = state.clone().apply_events(&[], &withdrawals);

    let certificate_id: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate.clone()])
        .await
        .unwrap();

    let result = wait_for_settlement_or_error!(client, certificate_id).await;

    assert!(matches!(result.status, CertificateStatus::InError { .. }));

    fail::cfg(
        "notifier::packer::settle_certificate::receipt_future_ended::status_0",
        "off",
    )
    .expect("Failed to configure failpoint");

    let certificate_id: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate])
        .await
        .unwrap();

    let result = wait_for_settlement_or_error!(client, certificate_id).await;

    assert!(matches!(result.status, CertificateStatus::Settled));

    scenario.teardown();
}
#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(180))]
async fn transaction_without_receipt_status() {
    let tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();

    fail::cfg(
        "notifier::packer::settle_certificate::receipt_future_ended::no_receipt",
        "return",
    )
    .expect("Failed to configure failpoint");

    // L1 is a RAII guard
    let (_handle, _l1, client) = setup_network(&tmp_dir.path).await;
    let signer = get_signer(0);

    let state = Forest::default().with_signer(signer);

    let withdrawals = vec![];

    let certificate = state.clone().apply_events(&[], &withdrawals);

    let certificate_id: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate])
        .await
        .unwrap();

    let result = wait_for_settlement_or_error!(client, certificate_id).await;

    assert!(matches!(result.status, CertificateStatus::InError { .. }));

    scenario.teardown();
}

#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(180))]
async fn transaction_fails_due_to_out_of_gas() {
    let tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();

    fail::cfg(
        "notifier::packer::settle_certificate::gas_estimate::low_gas",
        "return",
    )
    .expect("Failed to configure failpoint");

    // L1 is a RAII guard
    let (_handle, _l1, client) = setup_network(&tmp_dir.path).await;
    let signer = get_signer(0);

    let state = Forest::default().with_signer(signer);

    let withdrawals = vec![];

    let certificate = state.clone().apply_events(&[], &withdrawals);

    let certificate_id: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate.clone()])
        .await
        .unwrap();

    let result = wait_for_settlement_or_error!(client, certificate_id).await;

    assert!(matches!(result.status, CertificateStatus::InError { .. }));

    fail::cfg(
        "notifier::packer::settle_certificate::gas_estimate::low_gas",
        "off",
    )
    .expect("Failed to configure failpoint");

    let certificate_id: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate])
        .await
        .unwrap();

    let result = wait_for_settlement_or_error!(client, certificate_id).await;

    assert!(matches!(result.status, CertificateStatus::Settled));

    scenario.teardown();
}
