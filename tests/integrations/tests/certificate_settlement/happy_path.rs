use std::{str::FromStr, time::Duration};

use agglayer_storage::tests::TempDBDir;
use agglayer_types::{CertificateId, CertificateStatus};
use alloy::{
    network::{self, Ethereum},
    primitives::{Address, U256},
    providers::{ProviderBuilder, RootProvider, WsConnect},
    rpc::types::{Filter, FilterBlockOption},
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
#[timeout(Duration::from_secs(500))]
#[case::type_0_ecdsa(crate::common::type_0_ecdsa_forest())]
async fn send_multiple_certificates(#[case] mut state: Forest) {
    use agglayer_config::Config;
    use agglayer_contracts::contracts::PolygonRollupManager::VerifyPessimisticStateTransition;
    use agglayer_types::{aggchain_proof::AggchainData, compute_signature_info};
    use alloy::providers::Provider as _;
    use pessimistic_proof::core::commitment::SignatureCommitmentVersion;
    use tokio_util::sync::CancellationToken;

    let tmp_dir = TempDBDir::new();
    let cancellation_token = CancellationToken::new();

    // L1 is a RAII guard
    let (agglayer_shutdowned, l1, client) =
        setup_network(&tmp_dir.path, None, Some(cancellation_token.clone())).await;

    for i in 0..100 {
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

        let certificate_id: CertificateId = client
            .request("interop_sendCertificate", rpc_params![certificate.clone()])
            .await
            .unwrap();

        let result = wait_for_settlement_or_error!(client, certificate_id).await;

        // assert!(matches!(result.status, CertificateStatus::Settled));
    }

    // let config = Config::default();

    let provider = RootProvider::<Ethereum>::new_http(reqwest::Url::parse(&l1.rpc).unwrap());
    let last_block = provider.get_block_number().await.unwrap();
    assert!(last_block != 0);
    println!("last_block: {last_block}");

    let filter = alloy::rpc::types::Filter::default()
        .event_signature(VerifyPessimisticStateTransition::SIGNATURE_HASH)
        .select(FilterBlockOption::Range {
            from_block: Some(alloy::eips::BlockNumberOrTag::Earliest),
            to_block: None,
        })
        .topic1(U256::from(state.network_id))
        .address(Address::from_str("0x0b306bf915c4d645ff596e518faf3f9669b97016").unwrap());

    let events = provider.get_logs(&filter).await.unwrap();
    for log in &events {
        println!("event: {log:?}");
    }
    assert_eq!(events.len(), 4);
}
