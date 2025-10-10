use std::{str::FromStr, time::Duration};

use agglayer_storage::tests::TempDBDir;
use agglayer_types::{CertificateId, CertificateStatus};
use alloy::{
    network::Ethereum,
    primitives::{Address, U256},
    providers::RootProvider,
    rpc::types::FilterBlockOption,
    sol_types::SolEvent,
};
use fail::FailScenario;
use integrations::{agglayer_setup::setup_network, wait_for_settlement_or_error};
use jsonrpsee::{core::client::ClientT as _, rpc_params};
use pessimistic_proof_test_suite::forest::Forest;
use rstest::rstest;

#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(180))]
#[case::type_0_ecdsa(crate::common::type_0_ecdsa_forest())]
async fn successfully_push_certificate(#[case] state: Forest) {
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

#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(60))]
#[case::type_0_ecdsa(crate::common::type_0_ecdsa_forest())]
async fn send_multiple_certificates(#[case] mut state: Forest) {
    let scenario = FailScenario::setup();
    use agglayer_types::{aggchain_proof::AggchainData, compute_signature_info};
    use pessimistic_proof::core::commitment::SignatureCommitmentVersion;
    use tokio_util::sync::CancellationToken;

    let tmp_dir = TempDBDir::new();
    let cancellation_token = CancellationToken::new();

    // L1 is a RAII guard
    let (_agglayer_shutdowned, _l1, client) =
        setup_network(&tmp_dir.path, None, Some(cancellation_token.clone())).await;

    for i in 0..2 {
        let withdrawals = vec![];

        let mut certificate = state.apply_events(&[], &withdrawals);
        certificate.height = i.into();
        let (_, signature, _) = compute_signature_info(
            certificate.new_local_exit_root,
            &certificate.imported_bridge_exits,
            &state.wallet,
            certificate.height,
            SignatureCommitmentVersion::V3,
        );
        certificate.aggchain_data = AggchainData::ECDSA { signature };
        if i == 1 {
            fail::cfg(
                "notifier::packer::settle_certificate::gas_estimate::zero_gas",
                "return",
            )
            .expect("Failed to configure failpoint");
        }
        let certificate_id: CertificateId = client
            .request("interop_sendCertificate", rpc_params![certificate.clone()])
            .await
            .unwrap();

        if i == 1 {
            let result = wait_for_settlement_or_error!(client, certificate_id).await;
            assert!(matches!(result.status, CertificateStatus::InError { .. }));
            fail::cfg(
                "notifier::packer::settle_certificate::gas_estimate::zero_gas",
                "off",
            )
            .expect("Failed to configure failpoint");
        }

        let result = wait_for_settlement_or_error!(client, certificate_id).await;
        assert!(matches!(result.status, CertificateStatus::Settled));
    }

    scenario.teardown();
}
