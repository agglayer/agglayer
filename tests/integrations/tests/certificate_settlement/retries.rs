use std::time::Duration;

use agglayer_storage::tests::TempDBDir;
use agglayer_types::{CertificateHeader, CertificateId, CertificateStatus};
use fail::FailScenario;
use integrations::{agglayer_setup::setup_network, wait_for_settlement_or_error};
use jsonrpsee::{core::client::ClientT as _, rpc_params};
use pessimistic_proof_test_suite::forest::Forest;
use rand::random;
use rstest::rstest;

mod recovery;

#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(180))]
#[case::type_0_ecdsa(crate::common::type_0_ecdsa_forest())]
async fn retry_on_error(#[case] state: Forest) {
    let tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();

    fail::cfg(
        "notifier::packer::settle_certificate::gas_estimate::low_gas",
        "return",
    )
    .expect("Failed to configure failpoint");

    // L1 is a RAII guard
    let (_handle, _l1, client) = setup_network(&tmp_dir.path, None, None).await;

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

/// Validate that a certificate that has been proven and sent to L1 can't be
/// replaced in the pending-pool.
#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(180))]
#[case::type_0_ecdsa(crate::common::type_0_ecdsa_forest())]
async fn regression_pushing_certificate_while_settling(#[case] state: Forest) {
    let tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();

    // L1 is a RAII guard
    let (_shutdown_shutdown, _l1, client) = setup_network(&tmp_dir.path, None, None).await;

    let withdrawals = vec![];

    let certificate = state.clone().apply_events(&[], &withdrawals);
    let network_id = certificate.network_id;

    let first_certificate_id: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate.clone()])
        .await
        .unwrap();

    loop {
        let certificate = certificate
            .clone()
            .with_new_local_exit_root(random::<[u8; 32]>().into());
        assert!(client
            .request::<CertificateId, _>("interop_sendCertificate", rpc_params![certificate])
            .await
            .is_err());

        tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(1)) => {},
            _ = client.request::<CertificateHeader, _>("interop_getLatestSettledCertificate", rpc_params![network_id, 0]) => {
                break
            }
        }
    }

    let result = wait_for_settlement_or_error!(client, first_certificate_id).await;

    assert!(matches!(result.status, CertificateStatus::Settled));

    scenario.teardown();
}

/// Validate that a certificate that has been proven and sent to L1 can't be
/// replaced in the certificate header store.
#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(180))]
#[case::type_0_ecdsa(crate::common::type_0_ecdsa_forest())]
async fn regression_pushing_certificate_after_settling(#[case] state: Forest) {
    let tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();

    // L1 is a RAII guard
    let (_shutdown_shutdown, _l1, client) = setup_network(&tmp_dir.path, None, None).await;

    let withdrawals = vec![];

    let certificate = state.clone().apply_events(&[], &withdrawals);

    let first_certificate_id: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate.clone()])
        .await
        .inspect_err(|err| eprintln!("Error sending first certificate: {err:?}"))
        .unwrap();

    // Send the first certificate. This should be settled.
    wait_for_settlement_or_error!(client, first_certificate_id).await;

    // Verify status is Settled.
    let first_header: CertificateHeader = client
        .request(
            "interop_getCertificateHeader",
            rpc_params![first_certificate_id],
        )
        .await
        .inspect_err(|err| eprintln!("Error getting certificate header: {err:?}"))
        .unwrap();
    assert!(matches!(first_header.status, CertificateStatus::Settled));

    tokio::time::sleep(Duration::from_secs(5)).await;

    // Send the second certificate, identical to the first and check the error.
    let second_submission_err = client
        .request::<CertificateId, _>("interop_sendCertificate", rpc_params![certificate.clone()])
        .await
        .inspect(|result| eprintln!("Managed to settle same certificate twice! Result: {result:?}"))
        .unwrap_err();
    match second_submission_err {
        jsonrpsee::core::ClientError::Call(error) => {
            assert_eq!(error.code(), -10_006);
            assert!(error.message().contains("Unable to replace"));
        }
        error => panic!("Unexpected error: {error:?}"),
    }

    // Optional await sufficient time for cert to be processed.
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Verify status is Settled. The two submissions have the same ID.
    // This checks the first submission still succeeds.
    let first_header_again: CertificateHeader = client
        .request(
            "interop_getCertificateHeader",
            rpc_params![certificate.hash()],
        )
        .await
        .unwrap();
    assert!(matches!(
        first_header_again.status,
        CertificateStatus::Settled
    ));

    scenario.teardown();
}

/// Test that a certificate in error that has already been settled (transaction
/// succeeded) can be automatically replaced. This tests the fix for issue #1157.
///
/// The test simulates the scenario where a certificate goes into error after
/// the settlement transaction is submitted. The certificate will have a
/// settlement_tx_hash and be in InError status. When the same certificate is
/// submitted again, it should be automatically replaced because the settlement
/// transaction was successful.
#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(180))]
#[case::type_0_ecdsa(crate::common::type_0_ecdsa_forest())]
async fn auto_replace_settled_certificate_in_error(#[case] state: Forest) {
    let tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();

    // Use a failpoint that causes the certificate to go into error after
    // the settlement transaction is submitted but before it completes.
    // This creates a certificate in InError status with a settlement_tx_hash.
    // We'll use a timeout failpoint that triggers after the transaction is
    // submitted, causing the certificate to be in error with settlement_tx_hash.
    fail::cfg(
        "notifier::packer::settle_certificate::receipt_future_ended::timeout",
        "1*return",
    )
    .expect("Failed to configure failpoint");

    // L1 is a RAII guard
    let (_shutdown_shutdown, _l1, client) = setup_network(&tmp_dir.path, None, None).await;

    let withdrawals = vec![];

    let certificate = state.clone().apply_events(&[], &withdrawals);

    // Send the certificate - it should go into error due to the timeout failpoint
    // but will have a settlement_tx_hash
    let first_certificate_id: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate.clone()])
        .await
        .inspect_err(|err| eprintln!("Error sending first certificate: {err:?}"))
        .unwrap();

    // Wait for the certificate to reach InError status
    let result = wait_for_settlement_or_error!(client, first_certificate_id).await;
    assert!(matches!(result.status, CertificateStatus::InError { .. }));

    // Verify the certificate has a settlement_tx_hash (this is key for the test)
    let header: CertificateHeader = client
        .request(
            "interop_getCertificateHeader",
            rpc_params![first_certificate_id],
        )
        .await
        .unwrap();
    
    assert!(
        header.settlement_tx_hash.is_some(),
        "Certificate in error should have settlement_tx_hash after timeout"
    );

    // Turn off the failpoint so the replacement can succeed
    fail::cfg(
        "notifier::packer::settle_certificate::receipt_future_ended::timeout",
        "off",
    )
    .expect("Failed to turn off failpoint");

    // Now send the same certificate again. The fix allows automatic replacement
    // when the certificate is in error but has a successful settlement transaction.
    // Since the transaction was submitted (has settlement_tx_hash) and we're
    // simulating it succeeded on L1, the replacement should work.
    let second_certificate_id: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate])
        .await
        .inspect_err(|err| eprintln!("Error sending replacement certificate: {err:?}"))
        .unwrap();

    // The replacement should succeed and the certificate should eventually settle
    let final_result = wait_for_settlement_or_error!(client, second_certificate_id).await;
    assert!(
        matches!(final_result.status, CertificateStatus::Settled),
        "Replacement certificate should settle successfully"
    );

    scenario.teardown();
}
