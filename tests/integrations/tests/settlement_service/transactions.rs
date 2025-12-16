use std::time::Duration;

use agglayer_storage::tests::TempDBDir;
use agglayer_types::{CertificateId, CertificateStatus};
use alloy::{
    network::Ethereum,
    providers::{Provider, RootProvider},
};
use fail::FailScenario;
use integrations::{
    agglayer_setup::{setup_network, start_l1},
    wait_for_settlement_or_error,
};
use jsonrpsee::{core::client::ClientT as _, rpc_params};
use pessimistic_proof_test_suite::forest::Forest;
use rstest::rstest;
use tracing::info;

#[test_log::test(tokio::test)]
async fn start_l1_network() {
    let _tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();

    info!("Starting L1 network...");
    let l1 = start_l1().await;

    info!("L1 network started, waiting for blocks to be mined...");

    // Create a provider to check the L1 chain
    let provider = RootProvider::<Ethereum>::new_http(reqwest::Url::parse(&l1.rpc).unwrap());

    // Get current block number
    let current_block = provider.get_block_number().await.unwrap();
    info!("Current block number: {}", current_block);

    info!("Test completed successfully");
    scenario.teardown();
}

#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(180))]
#[case::type_0_ecdsa(crate::common::type_0_ecdsa_forest())]
async fn deconstruct_reconstruct_transaction(#[case] state: Forest) {
    let tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();

    // L1 is a RAII guard
    let (_handle, _l1, client) = setup_network(&tmp_dir.path, None, None).await;

    let withdrawals = vec![];

    let certificate = state.clone().apply_events(&[], &withdrawals);

    let certificate_id: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate])
        .await
        .unwrap();

    let result = wait_for_settlement_or_error!(client, certificate_id).await;

    assert_eq!(result.status, CertificateStatus::Settled);

    scenario.teardown();
}
