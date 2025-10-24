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
use pessimistic_proof::core::commitment::SignatureCommitmentVersion;
use pessimistic_proof_test_suite::forest::Forest;
use rstest::rstest;
use tokio_util::sync::CancellationToken;

#[rstest]
#[case::cert_task_type_0_ecdsa(
    &["certificate_task::process_impl::about_to_record_candidate", "network_task::make_progress::settlement_submitted"],
    crate::common::type_0_ecdsa_forest()
)]
#[tokio::test]
#[timeout(Duration::from_secs(90))]
async fn sent_transaction_recover(#[case] failpoints: &[&str], #[case] state: Forest) {
    // Shutdown node immediately after sending the settlement transaction, without
    // updating database. Try to recover by sending same certifciate after
    // startup.
    let tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();

    for f in failpoints {
        fail::cfg(*f, "panic(killing node)").expect("Failed to configure failpoint");
    }

    // L1 is a RAII guard
    let (agglayer_shutdowned, l1, client) = setup_network(&tmp_dir.path, None, None).await;
    let withdrawals = vec![];
    let imported_bridge_events = vec![];
    let certificate = state
        .clone()
        .apply_events(&imported_bridge_events, &withdrawals);
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
#[timeout(Duration::from_secs(180))]
#[case::type_0_ecdsa(crate::common::type_0_ecdsa_forest())]
async fn sent_transaction_recover_after_settlement(#[case] mut state: Forest) {
    // Settle one certificate, shutdown node.
    // Send other certificate settlement tx and on timeout (but tx has settled in
    // the background), shutdown node. Try to recover after starting up agglayer
    // for the second time.
    let tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();
    let cancellation_token = CancellationToken::new();
    // L1 is a RAII guard
    let (agglayer_shutdowned, l1, client) =
        setup_network(&tmp_dir.path, None, Some(cancellation_token.clone())).await;

    let withdrawals = vec![];
    let imported_bridge_events = vec![];
    let certificate = state.apply_events(&imported_bridge_events, &withdrawals);
    let certificate_id: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate.clone()])
        .await
        .unwrap();
    let result = wait_for_settlement_or_error!(client, certificate_id).await;

    assert!(matches!(result.status, CertificateStatus::Settled));

    cancellation_token.cancel();
    _ = agglayer_shutdowned.await;

    println!("Node killed for the first time, recovering...");

    let cancellation_token = CancellationToken::new();
    let (agglayer_shutdowned, client, _) =
        start_agglayer(&tmp_dir.path, &l1, None, Some(cancellation_token.clone())).await;

    fail::cfg_callback(
        "notifier::packer::settle_certificate::receipt_future_ended::timeout",
        move || cancellation_token.cancel(),
    )
    .expect("Failed to configure failpoint");

    let mut certificate2 = state.apply_events(&imported_bridge_events, &withdrawals);
    certificate2.height = 1.into();
    let (_, signature2, _) = compute_signature_info(
        certificate2.new_local_exit_root,
        &certificate2.imported_bridge_exits,
        &state.wallet,
        certificate2.height,
        SignatureCommitmentVersion::V3,
    );
    certificate2.aggchain_data = AggchainData::ECDSA {
        signature: signature2,
    };

    let certificate2_id: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate2.clone()])
        .await
        .unwrap();

    _ = agglayer_shutdowned.await;

    println!("Node killed for the second time, recovering...");

    fail::cfg(
        "notifier::packer::settle_certificate::receipt_future_ended::timeout",
        "off",
    )
    .expect("Failed to configure failpoint");

    tokio::time::sleep(Duration::from_secs(30)).await;
    let (_agglayer_shutdowned, client, _) = start_agglayer(&tmp_dir.path, &l1, None, None).await;

    println!("Node recovered, waiting for settlement...");

    let result = wait_for_settlement_or_error!(client, certificate2_id).await;
    assert!(matches!(result.status, CertificateStatus::Settled));

    scenario.teardown();
}

#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(120))]
#[case::type_0_ecdsa(crate::common::type_0_ecdsa_forest())]
async fn recover_after_multiple_replacement_transactions(#[case] state: Forest) {
    // Retry the settlement transaction 3 times, then shutdown node on timeout 4th
    // time. Recover on startup.
    let tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();
    let cancellation_token = CancellationToken::new();

    // Configure timeout failpoint to trigger exactly 3 times (3 timeout errors)
    fail::cfg(
        "notifier::packer::settle_certificate::receipt_future_ended::timeout",
        "3*return",
    )
    .expect("Failed to configure timeout failpoint");

    // Clone token for callback before moving it
    let cancellation_token_for_callback = cancellation_token.clone();

    // Configure timeout2 failpoint with callback to cancel token (for 4th attempt)
    fail::cfg_callback(
        "notifier::packer::settle_certificate::receipt_future_ended::timeout2",
        move || cancellation_token_for_callback.cancel(),
    )
    .expect("Failed to configure failpoint");

    let mut config = agglayer_config::Config::new(&tmp_dir.path);
    config.outbound.rpc.settle.confirmations = 10;
    config.outbound.rpc.settle.settlement_timeout = Duration::from_secs(10);

    // L1 is a RAII guard
    let (agglayer_shutdowned, l1, client) = setup_network(
        &tmp_dir.path,
        Some(config),
        Some(cancellation_token.clone()),
    )
    .await;

    let withdrawals = vec![];
    let imported_bridge_events = vec![];
    let certificate = state
        .clone()
        .apply_events(&imported_bridge_events, &withdrawals);
    let certificate_id: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate.clone()])
        .await
        .unwrap();

    _ = agglayer_shutdowned.await;

    println!("Node killed after timeouts, recovering...");

    // Turn off both failpoints
    fail::cfg(
        "notifier::packer::settle_certificate::receipt_future_ended::timeout",
        "off",
    )
    .expect("Failed to turn off timeout failpoint");

    fail::cfg(
        "notifier::packer::settle_certificate::receipt_future_ended::timeout2",
        "off",
    )
    .expect("Failed to turn off timeout2 failpoint");

    tokio::time::sleep(Duration::from_secs(30)).await;
    let (_agglayer_shutdowned, client, _) = start_agglayer(&tmp_dir.path, &l1, None, None).await;

    println!("Node recovered, waiting for settlement...");

    let result = wait_for_settlement_or_error!(client, certificate_id).await;
    assert!(matches!(result.status, CertificateStatus::Settled));

    scenario.teardown();
}
