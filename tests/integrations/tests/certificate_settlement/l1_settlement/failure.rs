use std::time::Duration;

use agglayer_storage::tests::TempDBDir;
use agglayer_types::{CertificateId, CertificateStatus, CertificateStatusError};
use fail::FailScenario;
use integrations::{agglayer_setup::setup_network, wait_for_settlement_or_error};
use jsonrpsee::{core::client::ClientT as _, rpc_params};
use pessimistic_proof_test_suite::forest::Forest;
use rstest::rstest;
use tokio_util::sync::CancellationToken;

#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(180))]
#[case::type_0_ecdsa(crate::common::type_0_ecdsa_forest())]
#[ignore = "failpoints live only in the bypassed RpcSettlementClient; pending migration"]
async fn transaction_with_receipt_status_0(#[case] state: Forest) {
    // Process the transaction with the execution status 0 (it is reverted).
    // Certificate should become `InError`.
    let tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();
    let cancellation_token = CancellationToken::new();

    fail::cfg(
        "notifier::packer::settle_certificate::receipt_future_ended::status_0",
        "return",
    )
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

    assert!(matches!(result.status, CertificateStatus::InError { .. }));

    cancellation_token.cancel();
    _ = agglayer_shutdowned.await;

    scenario.teardown();
}

#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(180))]
#[case::type_0_ecdsa(crate::common::type_0_ecdsa_forest())]
#[ignore = "failpoints live only in the bypassed RpcSettlementClient; pending migration"]
async fn transaction_with_receipt_status_0_retry(#[case] state: Forest) {
    // If transaction failed (reverted) due to low gas, settlement logic should
    // retry it. Transaction should be settled eventually.
    let tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();
    let cancellation_token = CancellationToken::new();

    fail::cfg(
        "notifier::packer::settle_certificate::gas_estimate::low_gas",
        "return",
    )
    .expect("Failed to configure failpoint");

    // L1 is a RAII guard
    let (agglayer_shutdowned, _l1, client) =
        setup_network(&tmp_dir.path, None, Some(cancellation_token.clone())).await;

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

    cancellation_token.cancel();
    _ = agglayer_shutdowned.await;

    scenario.teardown();
}

#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(180))]
#[case::type_0_ecdsa(crate::common::type_0_ecdsa_forest())]
#[ignore = "failpoints live only in the bypassed RpcSettlementClient; pending migration"]
async fn transaction_without_receipt_status(#[case] state: Forest) {
    // If transaction is lost or not included,
    let tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();
    let cancellation_token = CancellationToken::new();

    fail::cfg(
        "notifier::packer::settle_certificate::receipt_future_ended::no_receipt",
        "return",
    )
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

    assert!(matches!(result.status, CertificateStatus::InError { .. }));

    cancellation_token.cancel();
    _ = agglayer_shutdowned.await;

    scenario.teardown();
}

#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(180))]
#[case::type_0_ecdsa(crate::common::type_0_ecdsa_forest())]
#[ignore = "failpoints live only in the bypassed RpcSettlementClient; pending migration"]
async fn transaction_with_receipt_timeout_many_times(#[case] state: Forest) {
    // Retry the settlement transaction limited number of times,
    // then the certificate should be in InError status with a SettlementError about
    // too many transactions
    let tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();
    let cancellation_token = CancellationToken::new();

    fail::cfg(
        "notifier::packer::settle_certificate::receipt_future_ended::timeout",
        "return",
    )
    .expect("Failed to configure failpoint");

    let mut config = agglayer_config::Config::new(&tmp_dir.path);
    config.outbound.rpc.settle_cert.confirmations = 50;

    // L1 is a RAII guard
    let (agglayer_shutdowned, _l1, client) = setup_network(
        &tmp_dir.path,
        Some(config),
        Some(cancellation_token.clone()),
    )
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
            assert!(
                matches!(
                    &*error,
                    CertificateStatusError::SettlementError(detail)
                        if detail.starts_with(
                            "Too many different settlement transactions submitted for the same certificate:"
                        )
                ),
                "Expected settlement error about too many transactions, but got: {error:?}"
            );
        }
        status => panic!("Expected InError status, but got: {status:?}"),
    }

    cancellation_token.cancel();
    _ = agglayer_shutdowned.await;

    scenario.teardown();
}

#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(180))]
#[case::type_0_ecdsa(crate::common::type_0_ecdsa_forest())]
#[ignore = "failpoints live only in the bypassed RpcSettlementClient; pending migration"]
async fn transaction_with_receipt_timeout_2_times(#[case] state: Forest) {
    // Retry the settlement transaction 2 times because of induced timeouts,
    // then the certificate should be settled
    let tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();
    let cancellation_token = CancellationToken::new();

    fail::cfg(
        "notifier::packer::settle_certificate::receipt_future_ended::timeout",
        "2*return",
    )
    .expect("Failed to configure failpoint");

    let mut config = agglayer_config::Config::new(&tmp_dir.path);
    config.outbound.rpc.settle_cert.confirmations = 2;

    // L1 is a RAII guard
    let (agglayer_shutdowned, _l1, client) = setup_network(
        &tmp_dir.path,
        Some(config),
        Some(cancellation_token.clone()),
    )
    .await;

    let withdrawals = vec![];

    let certificate = state.clone().apply_events(&[], &withdrawals);

    let certificate_id: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate.clone()])
        .await
        .unwrap();

    let result = wait_for_settlement_or_error!(client, certificate_id).await;

    assert!(matches!(result.status, CertificateStatus::Settled));

    cancellation_token.cancel();
    _ = agglayer_shutdowned.await;

    scenario.teardown();
}
