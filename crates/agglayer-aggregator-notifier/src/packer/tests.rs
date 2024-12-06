use std::sync::Arc;

use agglayer_certificate_orchestrator::EpochPacker;
use agglayer_config::outbound::OutboundRpcSettleConfig;
use agglayer_contracts::Settler;
use agglayer_storage::tests::mocks::{MockPerEpochStore, MockStateStore};
use agglayer_types::{Certificate, CertificateHeader, LocalNetworkStateData, Proof};
use ethers::{
    contract::{ContractCall, ContractError},
    middleware::NonceManagerMiddleware,
    providers::{MockProvider, Provider},
    types::H160,
};
use mockall::predicate::eq;
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
    impl Settler for L1Rpc {
        type M = NonceManagerMiddleware<Provider<MockProvider>>;

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
fn epoch_packer_can_settle_one_certificate() {
    let network_id = 1.into();
    let certificate = Certificate::new_for_test(network_id, 0);
    let certificate_id = certificate.hash();

    let config = Arc::new(OutboundRpcSettleConfig::default());
    let mut state_store = MockStateStore::new();

    state_store
        .expect_get_certificate_header()
        .once()
        .with(eq(certificate_id))
        .returning(move |_| {
            Ok(Some(CertificateHeader {
                network_id,
                height: 0,
                epoch_number: Some(0),
                certificate_index: Some(0),
                certificate_id,
                prev_local_exit_root: [1; 32].into(),
                new_local_exit_root: [0; 32].into(),
                metadata: [0; 32].into(),
                status: agglayer_types::CertificateStatus::Candidate,
            }))
        });

    let (mock, _) = Provider::mocked();
    let _t = NonceManagerMiddleware::new(mock, H160::zero());

    let mut l1_rpc = MockL1Rpc::new();

    l1_rpc
        .expect_default_l1_info_tree_entry()
        .once()
        .returning(|| (0u32, [1u8; 32]));

    let mut per_epoch_store = MockPerEpochStore::new();
    per_epoch_store
        .expect_get_epoch_number()
        .once()
        .return_const(0u64);

    per_epoch_store
        .expect_get_certificate_at_index()
        .once()
        .with(eq(0))
        .returning(move |_| Ok(Some(certificate.clone())));

    per_epoch_store
        .expect_get_proof_at_index()
        .once()
        .with(eq(0))
        .returning(move |_| {
            let proof = Proof::new_for_test();

            let _state = LocalNetworkStateData::default();
            // TODO: generation PP

            Ok(Some(proof))
        });

    let epoch_packer = EpochPackerClient::<_, MockPerEpochStore, _>::try_new(
        config,
        Arc::new(state_store),
        Arc::new(l1_rpc),
    )
    .unwrap();

    assert!(epoch_packer
        .settle_certificate(Arc::new(per_epoch_store), 0, certificate_id)
        .is_err());
}
