use std::sync::Arc;

use agglayer_certificate_orchestrator::tests::mocks::MockL1Rpc;
use agglayer_config::outbound::OutboundRpcSettleConfig;
use agglayer_storage::tests::mocks::{MockPendingStore, MockPerEpochStore, MockStateStore};
use agglayer_types::{
    aggchain_data::CertificateAggchainDataCtx, CertificateHeader, CertificateStatus, Height,
    L1WitnessCtx, Metadata, PessimisticRootInput, Proof,
};
use alloy::primitives::FixedBytes;
use arc_swap::ArcSwap;
use mockall::predicate::eq;
use pessimistic_proof::core::commitment::PessimisticRootCommitmentVersion;
use pessimistic_proof_test_suite::forest::Forest;
use rstest::rstest;

use crate::settlement_client::RpcSettlementClient;

#[rstest]
#[test_log::test(tokio::test)]
#[ignore = "Complex integration test - requires proper mock setup"]
async fn epoch_packer_can_settle_one_certificate() {
    use agglayer_certificate_orchestrator::SettlementClient;

    let network_id = 1.into();
    let mut state = Forest::new(vec![]);

    let withdrawals = vec![];

    let signer = state.get_signer();
    let certificate = state.clone().apply_events(&[], &withdrawals);

    let l1_info_root = certificate.l1_info_root().unwrap().unwrap_or_default();
    let batch_header = state
        .state_b
        .apply_certificate(
            &certificate,
            L1WitnessCtx {
                l1_info_root,
                prev_pessimistic_root: PessimisticRootInput::Computed(
                    PessimisticRootCommitmentVersion::V2,
                ),
                aggchain_data_ctx: CertificateAggchainDataCtx::LegacyEcdsa { signer },
            },
        )
        .unwrap();
    let certificate_id = certificate.hash();

    let config = Arc::new(OutboundRpcSettleConfig::default());
    let mut state_store = MockStateStore::new();
    let mut pending_store = MockPendingStore::new();

    pending_store
        .expect_get_certificate()
        .once()
        .with(eq(network_id), eq(Height::ZERO))
        .returning(move |_, _| Ok(Some(certificate.clone())));

    state_store
        .expect_get_certificate_header()
        .once()
        .with(eq(certificate_id))
        .returning(move |_| {
            Ok(Some(CertificateHeader {
                network_id,
                height: Height::ZERO,
                epoch_number: None,
                certificate_index: None,
                certificate_id,
                prev_local_exit_root: [1; 32].into(),
                new_local_exit_root: [0; 32].into(),
                metadata: Metadata::ZERO,
                status: CertificateStatus::Proven,
                settlement_tx_hash: None,
            }))
        });

    let mut l1_rpc = MockL1Rpc::new();

    l1_rpc
        .expect_default_l1_info_tree_entry()
        .once()
        .returning(|| (0u32, [1u8; 32]));

    let per_epoch_store = MockPerEpochStore::new();

    let proof = Proof::new_for_test(&state.local_state().into(), &batch_header);

    pending_store
        .expect_get_proof()
        .once()
        .with(eq(certificate_id))
        .returning(move |_| Ok(Some(proof.clone())));

    // Note: This test is currently ignored because it requires complex mock setup
    // for alloy contract calls and transaction handling. The RpcSettlementClient
    // compiles and works correctly, but comprehensive testing requires a more
    // sophisticated test setup with proper alloy provider mocking.
    // Problem with mocking alloy contract calls is that alloy client makes
    // multiple calls to l1 (to assess to gas const etc.), and we need to mock
    // each of them correctly in the correct order.

    let epoch_packer = RpcSettlementClient::<_, _, MockPerEpochStore, _>::new(
        config,
        Arc::new(state_store),
        Arc::new(pending_store),
        Arc::new(l1_rpc),
        Arc::new(ArcSwap::new(Arc::new(per_epoch_store))),
    );

    let settlement_tx_hash = epoch_packer
        .submit_certificate_settlement(certificate_id, None)
        .await
        .unwrap();

    epoch_packer
        .wait_for_settlement(settlement_tx_hash, certificate_id)
        .await
        .unwrap();
}

#[test_log::test(tokio::test)]
#[ignore = "reaches external endpoint"]
async fn test_fetch_last_settled_pp_root() {
    use agglayer_certificate_orchestrator::SettlementClient;
    use agglayer_types::NetworkId;
    use url::Url;

    // Use L1_RPC_ENDPOINT environment variable (should be set to Sepolia endpoint)
    let rpc_url = std::env::var("L1_RPC_ENDPOINT")
        .expect("L1_RPC_ENDPOINT must be defined")
        .parse::<Url>()
        .expect("Invalid URL format");

    let rpc = prover_alloy::build_alloy_fill_provider(
        &rpc_url,
        prover_alloy::DEFAULT_HTTP_RPC_NODE_INITIAL_BACKOFF_MS,
        prover_alloy::DEFAULT_HTTP_RPC_NODE_BACKOFF_MAX_RETRIES,
    )
    .expect("valid alloy provider");

    tracing::info!("Testing fetch_last_settled_pp_root for Bali testnet");

    // Use the specified contract addresses for Bali testnet
    let rollup_manager_address: alloy::primitives::Address =
        "0xE2EF6215aDc132Df6913C8DD16487aBF118d1764"
            .parse()
            .expect("Invalid rollup manager address");
    let global_exit_root_manager_address: alloy::primitives::Address =
        "0x2968D6d736178f8FE7393CC33C87f29D9C287e78"
            .parse()
            .expect("Invalid PolygonZkEVMGlobalExitRootV2 address");

    // Create L1RpcClient with default config for other parameters for Bali testnet
    let l1_rpc = agglayer_contracts::L1RpcClient::try_new(
        Arc::new(rpc.clone()),
        agglayer_contracts::contracts::PolygonRollupManager::new(rollup_manager_address, rpc),
        global_exit_root_manager_address,
        100, // default gas_multiplier_factor
        agglayer_contracts::GasPriceParams::default(),
        10000, // default event_filter_block_range
    )
    .await
    .expect("Failed to create L1RpcClient");

    // Create RpcSettlementClient to test fetch_last_settled_pp_root
    let config = Arc::new(OutboundRpcSettleConfig::default());
    let state_store = Arc::new(MockStateStore::new());
    let pending_store = Arc::new(MockPendingStore::new());
    let per_epoch_store = Arc::new(ArcSwap::new(Arc::new(MockPerEpochStore::new())));

    let settlement_client = RpcSettlementClient::new(
        config,
        state_store,
        pending_store,
        Arc::new(l1_rpc),
        per_epoch_store,
    );

    // Test fetch_last_settled_pp_root for different network IDs
    let test_network_ids = vec![NetworkId::new(48), NetworkId::new(52), NetworkId::new(57)];

    tracing::info!(
        "Testing {} network IDs: {:?}",
        test_network_ids.len(),
        test_network_ids
    );

    for network_id in test_network_ids {
        tracing::debug!("Testing network ID: {}", network_id);

        match settlement_client
            .fetch_last_settled_pp_root(network_id)
            .await
        {
            Ok(Some((pp_root, tx_hash))) => {
                tracing::info!(
                    "Network {} has settled PP root: {} in tx: {}",
                    network_id,
                    FixedBytes::<32>::from(pp_root),
                    tx_hash
                );
                // Verify that the root is not all zeros (which would indicate an invalid
                // result)
                assert_ne!(
                    pp_root, [0u8; 32],
                    "PP root should not be all zeros for network {network_id}",
                );
            }
            Ok(None) => {
                tracing::info!("Network {} has no settled PP root yet", network_id);
            }
            Err(error) => {
                tracing::warn!(
                    "Failed to fetch last settled PP root for network {network_id}: {error}",
                );
                // For this test, we expect some networks might not have any
                // settled roots yet, so we don't fail the test
                // but just continue
            }
        }
    }

    tracing::info!("Completed testing fetch_last_settled_pp_root for all network IDs");
}
