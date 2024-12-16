use std::time::Duration;

use agglayer_storage::tests::TempDBDir;
use agglayer_types::{CertificateId, CertificateStatus};
use fail::FailScenario;
use integrations::{
    agglayer_setup::{get_signer, setup_network, start_agglayer},
    wait_for_settlement_or_error,
};
use jsonrpsee::core::client::ClientT as _;
use jsonrpsee::rpc_params;
use pessimistic_proof_test_suite::forest::Forest;
use rstest::rstest;

#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(60))]
async fn sent_transaction_recover() {
    let tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();

    fail::cfg(
        "notifier::packer::settle_certificate::transaction_sent::kill_node",
        "panic(killing node)",
    )
    .expect("Failed to configure failpoint");

    // L1 is a RAII guard
    let (agglayer_shutdowned, l1, client) = setup_network(&tmp_dir.path).await;
    let signer = get_signer(0);

    let state = Forest::default().with_signer(signer);

    let withdrawals = vec![];

    let certificate = state.clone().apply_events(&[], &withdrawals);

    let certificate_id: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate.clone()])
        .await
        .unwrap();

    _ = agglayer_shutdowned.await;

    println!("Node killed, recovering...");

    fail::cfg(
        "notifier::packer::settle_certificate::transaction_sent::kill_node",
        "off",
    )
    .expect("Failed to configure failpoint");

    let (_agglayer_shutdowned, client) = start_agglayer(&tmp_dir.path, &l1).await;

    println!("Node recovered, waiting for settlement...");

    let result = wait_for_settlement_or_error!(client, certificate_id).await;

    assert!(matches!(result.status, CertificateStatus::Settled));

    scenario.teardown();
}
