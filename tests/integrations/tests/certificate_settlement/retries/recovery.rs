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
#[test_log::test(tokio::test)]
#[timeout(Duration::from_secs(240))]
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

    tracing::info!("Node killed for the first time, recovering...");

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

    tracing::info!("Node killed for the second time, recovering...");

    fail::cfg(
        "notifier::packer::settle_certificate::receipt_future_ended::timeout",
        "off",
    )
    .expect("Failed to configure failpoint");

    tokio::time::sleep(Duration::from_secs(30)).await;
    let (_agglayer_shutdowned, client, _) = start_agglayer(&tmp_dir.path, &l1, None, None).await;

    tracing::info!("Node recovered, waiting for settlement...");

    let result = wait_for_settlement_or_error!(client, certificate2_id).await;
    tracing::info!("Second settlement result: {result:?}");
    assert!(matches!(result.status, CertificateStatus::Settled));

    scenario.teardown();
}

#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(120))]
#[case::type_0_ecdsa(crate::common::type_0_ecdsa_forest())]
async fn recover_after_invalid_transaction_in_header(#[case] state: Forest) {
    // Submit a certificate, inject an invalid tx hash in the header, then shutdown
    // node. Recover on startup and verify the node can detect and recover from
    // the invalid hash.
    let tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();
    let cancellation_token = CancellationToken::new();

    // Clone token for callback before moving it
    let cancellation_token_for_callback = cancellation_token.clone();

    // Configure invalid_settlement_tx_hash to inject invalid hash on first call
    fail::cfg(
        "certificate_task::process_impl::invalid_settlement_tx_hash",
        "return",
    )
    .expect("Failed to configure invalid_settlement_tx_hash failpoint");

    // Configure candidate_recorded to shutdown the node after invalid hash is
    // written
    fail::cfg_callback(
        "certificate_task::process_impl::candidate_recorded",
        move || {
            cancellation_token_for_callback.cancel();
            panic!("killing node after invalid hash injection");
        },
    )
    .expect("Failed to configure candidate_recorded failpoint");

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

    println!("Node killed after invalid hash injection, recovering...");

    // Turn off all failpoints
    fail::cfg(
        "certificate_task::process_impl::invalid_settlement_tx_hash",
        "off",
    )
    .expect("Failed to turn off invalid_settlement_tx_hash failpoint");

    fail::cfg("certificate_task::process_impl::candidate_recorded", "off")
        .expect("Failed to turn off candidate_recorded failpoint");

    tokio::time::sleep(Duration::from_secs(30)).await;
    let (_agglayer_shutdowned, client, _) = start_agglayer(&tmp_dir.path, &l1, None, None).await;

    println!("Node recovered, waiting for settlement...");

    let result = wait_for_settlement_or_error!(client, certificate_id).await;
    assert!(matches!(result.status, CertificateStatus::Settled));

    scenario.teardown();
}
