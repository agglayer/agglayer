use std::sync::Arc;

use agglayer_certificate_orchestrator::EpochPacker;
use agglayer_config::outbound::OutboundRpcSettleConfig;
use agglayer_contracts::Settler;
use agglayer_storage::tests::mocks::{MockPendingStore, MockPerEpochStore, MockStateStore};
use agglayer_types::{CertificateHeader, Proof};
use arc_swap::ArcSwap;
use ethers::{
    contract::{ContractCall, ContractError},
    middleware::NonceManagerMiddleware,
    providers::{MockProvider, Provider},
    types::H160,
};
use mockall::predicate::eq;
use pessimistic_proof_test_suite::forest::Forest;
use rstest::rstest;

use crate::EpochPackerClient;

mockall::mock! {
    L1Rpc {}
    #[async_trait::async_trait]
    impl agglayer_contracts::RollupContract for L1Rpc {
        type M = NonceManagerMiddleware<Provider<MockProvider>>;

        async fn get_trusted_sequencer_address(
            &self,
            rollup_id: u32,
            proof_signers: std::collections::HashMap<u32,ethers::types::Address> ,
        ) -> Result<ethers::types::Address, ()>;

        async fn get_l1_info_root(&self, l1_leaf_count: u32) -> Result<[u8; 32], ()>;
        fn default_l1_info_tree_entry(&self) -> (u32, [u8; 32]);
    }
    #[async_trait::async_trait]
    impl Settler for L1Rpc {
        type M = NonceManagerMiddleware<Provider<MockProvider>>;

        async fn transaction_exists(&self, tx_hash: ethers::types::H256) -> Result<bool, String>;
        fn build_pending_transaction(
            &self,
            tx_hash: ethers::types::H256,
        ) -> ethers::providers::PendingTransaction<'_, <NonceManagerMiddleware<Provider<MockProvider>> as ethers::providers::Middleware>::Provider>;

        fn decode_contract_revert(error: &ContractError<NonceManagerMiddleware<Provider<MockProvider>>>) -> Option<String>;
        fn build_verify_pessimistic_trusted_aggregator_call(
            &self,
            rollup_id: u32,
            l_1_info_tree_leaf_count: u32,
            new_local_exit_root: [u8; 32],
            new_pessimistic_root: [u8; 32],
            proof: ::ethers::core::types::Bytes,
        ) -> ContractCall<NonceManagerMiddleware<Provider<MockProvider>>, ()>;
    }
}

#[rstest]
#[test_log::test(tokio::test)]
#[ignore = "Unable to properly test contract with mock"]
async fn epoch_packer_can_settle_one_certificate() {
    let network_id = 1.into();
    let mut state = Forest::new(vec![]);

    let withdrawals = vec![];

    let signer = state.get_signer();
    let certificate = state.clone().apply_events(&[], &withdrawals);

    let l1_info_root = certificate.l1_info_root().unwrap().unwrap_or_default();
    let batch_header = state
        .state_b
        .apply_certificate(&certificate, signer, l1_info_root)
        .unwrap();
    let certificate_id = certificate.hash();

    let config = Arc::new(OutboundRpcSettleConfig::default());
    let mut state_store = MockStateStore::new();
    let mut pending_store = MockPendingStore::new();

    pending_store
        .expect_get_certificate()
        .once()
        .with(eq(network_id), eq(0))
        .returning(move |_, _| Ok(Some(certificate.clone())));

    state_store
        .expect_get_certificate_header()
        .once()
        .with(eq(certificate_id))
        .returning(move |_| {
            Ok(Some(CertificateHeader {
                network_id,
                height: 0,
                epoch_number: None,
                certificate_index: None,
                certificate_id,
                prev_local_exit_root: [1; 32].into(),
                new_local_exit_root: [0; 32].into(),
                metadata: [0; 32].into(),
                status: agglayer_types::CertificateStatus::Proven,
                settlement_tx_hash: None,
            }))
        });

    let (mock, _) = Provider::mocked();
    let _t = NonceManagerMiddleware::new(mock, H160::zero());

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

    let epoch_packer = EpochPackerClient::<_, _, MockPerEpochStore, _>::try_new(
        config,
        Arc::new(state_store),
        Arc::new(pending_store),
        Arc::new(l1_rpc),
        Arc::new(ArcSwap::new(Arc::new(per_epoch_store))),
    )
    .unwrap();

    let r = epoch_packer.settle_certificate(certificate_id).await;

    assert!(r.is_ok());
}
