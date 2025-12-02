use std::time::Duration;

use agglayer_storage::tests::TempDBDir;
use agglayer_types::{CertificateId, CertificateStatus};
use fail::FailScenario;
use integrations::{agglayer_setup::AgglayerSetup, wait_for_settlement_or_error};
use jsonrpsee::{core::client::ClientT as _, rpc_params};
use pessimistic_proof_test_suite::forest::Forest;
use rstest::rstest;

#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(180))]
#[case::type_0_ecdsa(crate::common::type_0_ecdsa_forest())]
async fn transaction_with_receipt_status_0(#[case] state: Forest) {
    // Process the transaction with the execution status 0 (it is reverted).
    // Certificate should become `InError`.
    let tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();

    fail::cfg(
        "notifier::packer::settle_certificate::receipt_future_ended::status_0",
        "return",
    )
    .expect("Failed to configure failpoint");

    // L1 is a RAII guard
    let (_handle, _l1, client) = AgglayerSetup::default().setup_network(&tmp_dir.path).await;

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
#[case::type_0_ecdsa(crate::common::type_0_ecdsa_forest())]
async fn transaction_with_receipt_status_0_retry(#[case] state: Forest) {
    // If transaction failed (reverted) due to low gas, settlement logic should
    // retry it. Transaction should be settled eventually.
    let tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();

    fail::cfg(
        "notifier::packer::settle_certificate::gas_estimate::low_gas",
        "return",
    )
    .expect("Failed to configure failpoint");

    // L1 is a RAII guard
    let (_handle, _l1, client) = AgglayerSetup::default().setup_network(&tmp_dir.path).await;

    let withdrawals = vec![];

    let certificate = state.clone().apply_events(&[], &withdrawals);

    let certificate_id: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate.clone()])
        .await
        .unwrap();

    let result = wait_for_settlement_or_error!(client, certificate_id).await;

    println!("{result:?}",);
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

#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(180))]
#[case::type_0_ecdsa(crate::common::type_0_ecdsa_forest())]
async fn transaction_without_receipt_status(#[case] state: Forest) {
    // If transaction is lost or not included,
    let tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();

    fail::cfg(
        "notifier::packer::settle_certificate::receipt_future_ended::no_receipt",
        "return",
    )
    .expect("Failed to configure failpoint");

    // L1 is a RAII guard
    let (_handle, _l1, client) = AgglayerSetup::default().setup_network(&tmp_dir.path).await;

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
#[case::type_0_ecdsa(crate::common::type_0_ecdsa_forest())]
async fn transaction_with_receipt_timeout_many_times(#[case] state: Forest) {
    // Retry the settlement transaction limited number of times,
    // then the certificate should be in InError status with a SettlementError about
    // too many transactions
    let tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();

    fail::cfg(
        "notifier::packer::settle_certificate::receipt_future_ended::timeout",
        "return",
    )
    .expect("Failed to configure failpoint");

    let mut config = agglayer_config::Config::new(&tmp_dir.path);
    config.outbound.rpc.settle.confirmations = 50;
    config.outbound.rpc.settle.settlement_timeout = Duration::from_secs(10);

    // L1 is a RAII guard
    let (_handle, _l1, client) = AgglayerSetup::default()
        .with_config(config)
        .setup_network(&tmp_dir.path)
        .await;

    let withdrawals = vec![];

    let certificate = state.clone().apply_events(&[], &withdrawals);

    let certificate_id: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate.clone()])
        .await
        .unwrap();

    let result = wait_for_settlement_or_error!(client, certificate_id).await;

    // Check that we got an InError status with a SettlementError about too many
    // transactions
    match result.status {
        CertificateStatus::InError { error } => {
            let error_message = error.to_string();
            assert!(
                error_message.contains(
                    "Too many different settlement transactions submitted for the same certificate"
                ),
                "Expected error message about too many settlement transactions, but got: \
                 {error_message}"
            );
        }
        status => panic!("Expected InError status, but got: {status:?}"),
    }

    scenario.teardown();
}

#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(180))]
#[case::type_0_ecdsa(crate::common::type_0_ecdsa_forest())]
async fn transaction_with_receipt_timeout_2_times(#[case] state: Forest) {
    // Retry the settlement transaction 2 times because of induced timeouts,
    // then the certificate should be settled
    let tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();

    fail::cfg(
        "notifier::packer::settle_certificate::receipt_future_ended::timeout",
        "2*return",
    )
    .expect("Failed to configure failpoint");

    let mut config = agglayer_config::Config::new(&tmp_dir.path);
    config.outbound.rpc.settle.confirmations = 2;
    config.outbound.rpc.settle.settlement_timeout = Duration::from_secs(10);

    // L1 is a RAII guard
    let (_handle, _l1, client) = AgglayerSetup::default()
        .with_config(config)
        .setup_network(&tmp_dir.path)
        .await;

    let withdrawals = vec![];

    let certificate = state.clone().apply_events(&[], &withdrawals);

    let certificate_id: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate.clone()])
        .await
        .unwrap();

    let result = wait_for_settlement_or_error!(client, certificate_id).await;

    assert!(matches!(result.status, CertificateStatus::Settled));

    scenario.teardown();
}
