use std::time::Duration;

use agglayer_storage::tests::TempDBDir;
use agglayer_types::{CertificateId, CertificateStatus};
use fail::FailScenario;
use integrations::{
    agglayer_setup::{setup_network, start_agglayer},
    wait_for_settlement_or_error,
};
use jsonrpsee::{core::client::ClientT as _, rpc_params};
use pessimistic_proof_test_suite::forest::Forest;
use rstest::rstest;

#[rstest]
// TODO: re-enable this test, once we handle in-flight in-error certificates properly.
// #[case::settlement_type_0_ecdsa(
//     &["notifier::packer::settle_certificate::transaction_sent::kill_node"],
//     crate::common::type_0_ecdsa_forest()
// )]
#[case::cert_task_type_0_ecdsa(
    &["certificate_task::about_to_record_candidate", "network_task::settlement_submitted"],
    crate::common::type_0_ecdsa_forest()
)]
#[tokio::test]
#[timeout(Duration::from_secs(90))]
async fn sent_transaction_recover(#[case] failpoints: &[&str], #[case] state: Forest) {
    let tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();

    for f in failpoints {
        fail::cfg(*f, "panic(killing node)").expect("Failed to configure failpoint");
    }

    // L1 is a RAII guard
    let (agglayer_shutdowned, l1, client) = setup_network(&tmp_dir.path, None, None).await;

    let withdrawals = vec![];

    let certificate = state.clone().apply_events(&[], &withdrawals);

    let certificate_id: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate.clone()])
        .await
        .unwrap();

    _ = agglayer_shutdowned.await;

    println!("Node killed, recovering...");

    for f in failpoints {
        fail::cfg(*f, "off").expect("Failed to configure failpoint");
    }

    let (_agglayer_shutdowned, client, _) = start_agglayer(&tmp_dir.path, &l1, None, None).await;

    println!("Node recovered, waiting for settlement...");

    let result = wait_for_settlement_or_error!(client, certificate_id).await;

    assert!(matches!(result.status, CertificateStatus::Settled));

    scenario.teardown();
}
