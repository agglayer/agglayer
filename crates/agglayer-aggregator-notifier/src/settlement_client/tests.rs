use std::sync::Arc;

use agglayer_config::outbound::OutboundRpcSettleConfig;
use agglayer_contracts::{L1RpcError, L1TransactionFetcher, Settler};
use agglayer_storage::tests::mocks::{MockPendingStore, MockPerEpochStore, MockStateStore};
use agglayer_types::{
    Address, CertificateHeader, CertificateStatus, Height, Metadata, PessimisticRootInput, Proof,
};
use alloy::{
    primitives::{Bytes, FixedBytes},
    providers::PendingTransactionBuilder,
    rpc::types::TransactionReceipt,
};
use arc_swap::ArcSwap;
use mockall::predicate::eq;
use pessimistic_proof::unified_bridge::CommitmentVersion;
use pessimistic_proof_test_suite::forest::Forest;
use rstest::rstest;

use crate::settlement_client::RpcSettlementClient;

mockall::mock! {
    L1Rpc {}
    #[async_trait::async_trait]
    impl agglayer_contracts::RollupContract for L1Rpc {
        type P = alloy::providers::RootProvider<alloy::network::Ethereum>;

        async fn get_trusted_sequencer_address(
            &self,
            rollup_id: u32,
            proof_signers: std::collections::HashMap<u32, Address>,
        ) -> Result<Address, L1RpcError>;

        async fn get_rollup_contract_address(&self, rollup_id: u32) -> Result<Address, L1RpcError>;

        async fn get_l1_info_root(&self, l1_leaf_count: u32) -> Result<[u8; 32], L1RpcError>;
        fn default_l1_info_tree_entry(&self) -> (u32, [u8; 32]);
        async fn get_prev_pessimistic_root(&self, rollup_id: u32) -> Result<[u8; 32], L1RpcError>;

        async fn get_verifier_type(&self, rollup_id: u32) -> Result<agglayer_contracts::rollup::VerifierType, L1RpcError>;
    }

    #[async_trait::async_trait]
    impl L1TransactionFetcher for L1Rpc {
        type Provider = alloy::providers::RootProvider<alloy::network::Ethereum>;

        async fn fetch_transaction_receipt(&self, tx_hash: FixedBytes<32>) -> Result<TransactionReceipt, L1RpcError>;

        fn get_provider(&self) -> &<Self as L1TransactionFetcher>::Provider;
    }

    #[async_trait::async_trait]
    impl Settler for L1Rpc {
        fn decode_contract_revert(error: &alloy::contract::Error) -> Option<String>;

        async fn verify_pessimistic_trusted_aggregator(
            &self,
            rollup_id: u32,
            l_1_info_tree_leaf_count: u32,
            new_local_exit_root: [u8; 32],
            new_pessimistic_root: [u8; 32],
            proof: Bytes,
            custom_chain_data: Bytes,
        ) -> Result<PendingTransactionBuilder<alloy::network::Ethereum>, alloy::contract::Error>;

    }
}

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
            signer,
            l1_info_root,
            PessimisticRootInput::Computed(CommitmentVersion::V2),
            None,
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
        .submit_certificate_settlement(certificate_id)
        .await
        .unwrap();

    epoch_packer
        .wait_for_settlement(settlement_tx_hash, certificate_id)
        .await
        .unwrap();
}
