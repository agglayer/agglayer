use std::sync::Arc;

use agglayer_certificate_orchestrator::EpochPacker;
use agglayer_config::{outbound::OutboundRpcSettleConfig, Config};
use agglayer_contracts::{polygon_rollup_manager::PolygonRollupManager, Settler};
use agglayer_storage::tests::mocks::{MockPerEpochStore, MockStateStore};
use agglayer_types::{Certificate, CertificateHeader, LocalNetworkStateData, Proof};
use ethers::{
    contract::{ContractCall, ContractError},
    middleware::NonceManagerMiddleware,
    providers::{Middleware as _, MockProvider, Provider},
    types::{OtherFields, Transaction, TransactionReceipt, H160, H256, U256},
    utils::Anvil,
};
use mockall::predicate::eq;
use rstest::rstest;

use crate::EpochPackerClient;

mockall::mock! {
    L1Rpc {}
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
#[tokio::test]
async fn epoch_packer_can_settle_one_certificate() {
    let network_id = 1.into();
    let certificate = Certificate::new_for_test(network_id, 0);
    let certificate_id = certificate.hash();

    let config = Arc::new(Config::default());
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
                new_local_exit_root: [0; 32].into(),
                metadata: [0; 32].into(),
                tx_hash: None,
                status: agglayer_types::CertificateStatus::Candidate,
            }))
        });

    let (mock, _) = Provider::mocked();
    let _t = NonceManagerMiddleware::new(mock, H160::zero());

    let anvil = Anvil::new().block_time(1u64).spawn();
    let anvil_provider = Provider::<ethers::providers::Http>::try_from(anvil.endpoint()).unwrap();

    let (provider, mock) = Provider::mocked();
    let block_number = anvil_provider.get_block_number().await.unwrap();

    let block = anvil_provider
        .get_block(block_number)
        .await
        .unwrap()
        .unwrap();

    let tx = Transaction {
        hash: ethers::types::H256([1u8; 32]),
        nonce: U256::zero(),
        block_number: Some(1.into()),
        block_hash: Some([1u8; 32].into()),
        transaction_index: Some(0.into()),
        from: ethers::types::H160::zero(),
        to: None,
        value: U256::zero(),
        gas_price: None,
        gas: U256::zero(),
        input: ethers::types::Bytes::new(),
        v: 0.into(),
        r: U256::zero(),
        s: U256::zero(),
        transaction_type: None,
        access_list: None,
        max_priority_fee_per_gas: None,
        max_fee_per_gas: None,
        chain_id: None,
        other: OtherFields::default(),
    };

    let receipt = TransactionReceipt::default();
    let estimate_gas = U256::from([1u8; 32]);
    let tx_hash = H256::from([2u8; 32]);

    mock.push(receipt).unwrap();
    mock.push(tx).unwrap();
    mock.push(tx_hash).unwrap();
    mock.push(estimate_gas).unwrap();
    mock.push(ethers::types::FeeHistory {
        base_fee_per_gas: Vec::new(),
        gas_used_ratio: Vec::new(),
        oldest_block: U256::zero(),
        reward: Vec::new(),
    })
    .unwrap();
    mock.push(block).unwrap();

    let inner = PolygonRollupManager::new(
        config.l1.rollup_manager_contract,
        Arc::new(provider.clone()),
    );

    let l1_rpc = agglayer_contracts::L1RpcClient::new(inner);

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
        Arc::new(config.outbound.rpc.settle.clone()),
        Arc::new(state_store),
        Arc::new(l1_rpc),
    )
    .unwrap();

    assert!(epoch_packer
        .settle_certificate(Arc::new(per_epoch_store), 0, certificate_id)
        .is_ok());
}
