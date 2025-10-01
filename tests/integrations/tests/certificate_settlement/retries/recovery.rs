use std::time::Duration;

use agglayer_storage::tests::TempDBDir;
use agglayer_types::{
    aggchain_proof::AggchainData, compute_signature_info, CertificateId, CertificateStatus,
};
use fail::FailScenario;
use integrations::{
    agglayer_setup::{setup_network, start_agglayer},
    wait_for_settlement_or_error,
};
use jsonrpsee::{core::client::ClientT as _, rpc_params};
use pessimistic_proof_test_suite::forest::Forest;
use rstest::rstest;
use tokio_util::sync::CancellationToken;

#[rstest]
// TODO: re-enable this test, once we handle in-flight in-error certificates properly.
// #[case::settlement_type_0_ecdsa(
//     &["notifier::packer::settle_certificate::transaction_sent::kill_node"],
//     crate::common::type_0_ecdsa_forest()
// )]
#[case::cert_task_type_0_ecdsa(
    &["certificate_task::process_impl::about_to_record_candidate", "network_task::make_progress::settlement_submitted"],
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

#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(120))]
#[case::type_0_ecdsa(crate::common::type_0_ecdsa_forest())]
async fn sent_transaction_recover_after_settlement(#[case] mut state: Forest) {
    use pessimistic_proof::unified_bridge::CommitmentVersion;

    let tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();

    let cancellation_token = CancellationToken::new();

    // L1 is a RAII guard
    let (agglayer_shutdowned, l1, client) =
        setup_network(&tmp_dir.path, None, Some(cancellation_token.clone())).await;

    let withdrawals = vec![];

    let certificate = state.apply_events(&[], &withdrawals);

    let certificate_id: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate.clone()])
        .await
        .unwrap();

    let result = wait_for_settlement_or_error!(client, certificate_id).await;

    assert!(matches!(result.status, CertificateStatus::Settled));

    cancellation_token.cancel();
    _ = agglayer_shutdowned.await;
    fail::cfg(
        "notifier::packer::settle_certificate::receipt_future_ended::kill_node",
        "panic(killing node)",
    )
    .expect("Failed to configure failpoint");

    let (agglayer_shutdowned, client, _) = start_agglayer(&tmp_dir.path, &l1, None, None).await;
    let withdrawals = vec![];

    let mut certificate = state.apply_events(&[], &withdrawals);
    certificate.height = 1;
    let (_, signature, _) = compute_signature_info(
        certificate.new_local_exit_root,
        &certificate.imported_bridge_exits,
        &state.wallet,
        certificate.height,
        CommitmentVersion::V3,
    );
    certificate.aggchain_data = AggchainData::ECDSA { signature };

    let certificate_id: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate.clone()])
        .await
        .unwrap();

    _ = agglayer_shutdowned.await;

    println!("Node killed, recovering...");

    fail::cfg(
        "notifier::packer::settle_certificate::receipt_future_ended::kill_node",
        "off",
    )
    .expect("Failed to configure failpoint");

    tokio::time::sleep(Duration::from_secs(30)).await;
    let (_agglayer_shutdowned, client, _) = start_agglayer(&tmp_dir.path, &l1, None, None).await;

    println!("Node recovered, waiting for settlement...");

    let result = wait_for_settlement_or_error!(client, certificate_id).await;

    assert!(matches!(result.status, CertificateStatus::Settled));

    scenario.teardown();
}
