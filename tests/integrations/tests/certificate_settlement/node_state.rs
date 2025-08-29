use std::time::Duration;

use agglayer_storage::tests::TempDBDir;
use agglayer_types::{CertificateId, CertificateStatus, NetworkStatus};
use fail::FailScenario;
use integrations::{agglayer_setup::setup_network, wait_for_settlement_or_error};
use jsonrpsee::{core::client::ClientT as _, rpc_params};
use pessimistic_proof_test_suite::forest::Forest;
use rstest::rstest;
use tracing::info;

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(Duration::from_secs(180))]
#[case::type_0_ecdsa(crate::common::type_0_ecdsa_forest())]
async fn get_network_status(#[case] state: Forest) {
    let tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();

    // L1 is a RAII guard
    let (_handle, _l1, client) = setup_network(&tmp_dir.path, None, None).await;

    let withdrawals = vec![];

    let certificate = state.clone().apply_events(&[], &withdrawals);
    let network_id = certificate.network_id;

    let certificate_id: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate])
        .await
        .unwrap();

    let result = wait_for_settlement_or_error!(client, certificate_id).await;

    assert_eq!(result.status, CertificateStatus::Settled);

    // Now test get_network_status API
    let network_status: NetworkStatus = client
        .request("interop_getNetworkStatus", rpc_params![network_id])
        .await
        .unwrap();

    info!("Network Status: {:?}", network_status);

    // Verify the network status response
    assert_eq!(network_status.network_id, network_id);
    assert_eq!(network_status.network_status, "TBD");
    // Settled height for first certificate is 0
    assert_eq!(
        network_status.settled_height,
        agglayer_types::Height::new(0)
    );
    // The latest pending status should indicate no pending certificates ("Unknown")
    assert_eq!(network_status.latest_pending_status, "Unknown");
    // No errors should be present
    assert!(network_status.latest_pending_error.is_empty());

    scenario.teardown();
}
