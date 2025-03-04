use std::time::Duration;

use agglayer_storage::tests::TempDBDir;
use agglayer_types::{CertificateHeader, CertificateId, CertificateStatus};
use fail::FailScenario;
use integrations::{agglayer_setup::setup_network, wait_for_settlement_or_error};
use jsonrpsee::core::client::ClientT as _;
use jsonrpsee::rpc_params;
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
