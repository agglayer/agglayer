use std::{sync::Arc, time::Duration};

use agglayer_certificate_orchestrator::Certifier;
use agglayer_config::Config;
use agglayer_contracts::{L1RpcError, Settler};
use agglayer_primitives::vkey_hash::VKeyHash;
use agglayer_storage::tests::{mocks::MockPendingStore, TempDBDir};
use agglayer_types::{Address, Height, LocalNetworkStateData, NetworkId};
use alloy::{
    contract::Error as ContractError,
    network::Ethereum,
    primitives::{Bytes, FixedBytes, TxHash},
    rpc::types::TransactionReceipt,
};
use fail::FailScenario;
use mockall::predicate::{always, eq};
use pessimistic_proof_test_suite::forest::Forest;
use prover_config::{MockProverConfig, ProverType};
use tower::buffer::Buffer;

use crate::{CertifierClient, ELF};

#[rstest::rstest]
#[test_log::test(tokio::test)]
async fn happy_path() {
    let scenario = FailScenario::setup();
    let base_path = TempDBDir::new();
    let config = Config::new(&base_path.path);

    let mut pending_store = MockPendingStore::new();
    let mut l1_rpc = MockL1Rpc::new();

    let (_vkey, prover) = prover_executor::Executor::create_prover(
        ProverType::MockProver(MockProverConfig::default()),
        ELF,
    )
    .await
    .unwrap();

    let buffer = Buffer::new(prover, 1024);

    let local_state = LocalNetworkStateData::default();
    let network: NetworkId = 1.into();
    let height = Height::ZERO;

    let state = Forest::new(vec![]);

    let withdrawals = vec![];

    let certificate = state.clone().apply_events(&[], &withdrawals);
    let signer = state.get_signer();
    let certificate_id = certificate.hash();

    pending_store
        .expect_get_certificate()
        .once()
        .with(eq(network), eq(height))
        .return_once(|_, _| Ok(Some(certificate)));

    pending_store
        .expect_insert_generated_proof()
        .once()
        .with(eq(certificate_id), always())
        .return_once(|_, _| Ok(()));

    l1_rpc
        .expect_get_trusted_sequencer_address()
        .once()
        .returning(move |_, _| Ok(signer));

    l1_rpc
        .expect_get_rollup_contract_address()
        .once()
        .returning(|_| Ok(Address::ZERO));

    l1_rpc
        .expect_default_l1_info_tree_entry()
        .once()
        .returning(|| (0u32, [1u8; 32]));

    l1_rpc
        .expect_get_prev_pessimistic_root()
        .once()
        .returning(|_, _| Ok([0u8; 32]));

    fail::cfg(
        "notifier::certifier::certify::before_verifying_proof",
        "return()",
    )
    .unwrap();

    let certifier = CertifierClient::try_new(
        Arc::new(pending_store),
        Arc::new(l1_rpc),
        Arc::new(config),
        buffer,
    )
    .await
    .unwrap();

    let result = certifier
        .certify(local_state.clone(), network, height)
        .await
        .unwrap();

    assert_eq!(result.new_state.get_roots(), local_state.get_roots());

    scenario.teardown();
}

#[rstest::rstest]
#[test_log::test(tokio::test)]
#[timeout(Duration::from_secs(60))]
async fn prover_timeout() {
    let scenario = FailScenario::setup();
    let base_path = TempDBDir::new();
    let config = Config::new(&base_path.path);

    let mut pending_store = MockPendingStore::new();
    let mut l1_rpc = MockL1Rpc::new();

    let local_state = LocalNetworkStateData::default();
    let network = NetworkId::new(1);
    let height = Height::ZERO;

    let state = Forest::new(vec![]);

    let withdrawals = vec![];

    let certificate = state.clone().apply_events(&[], &withdrawals);

    let signer = state.get_signer();
    let certificate_id = certificate.hash();

    pending_store
        .expect_get_certificate()
        .once()
        .with(eq(network), eq(height))
        .return_once(|_, _| Ok(Some(certificate)));

    pending_store
        .expect_insert_generated_proof()
        .never()
        .with(eq(certificate_id), always())
        .return_once(|_, _| Ok(()));

    l1_rpc
        .expect_get_trusted_sequencer_address()
        .once()
        .returning(move |_, _| Ok(signer));

    l1_rpc
        .expect_get_rollup_contract_address()
        .once()
        .returning(|_| Ok(Address::ZERO));

    l1_rpc
        .expect_default_l1_info_tree_entry()
        .once()
        .returning(|| (0u32, [1u8; 32]));

    l1_rpc
        .expect_get_prev_pessimistic_root()
        .once()
        .returning(|_, _| Ok([0u8; 32]));

    fail::cfg(
        "notifier::certifier::certify::prover_service_timeout",
        "return",
    )
    .expect("Failed to configure failpoint");

    fail::cfg(
        "notifier::certifier::certify::before_verifying_proof",
        "return()",
    )
    .unwrap();

    let (_vkey, prover) = prover_executor::Executor::create_prover(
        ProverType::MockProver(MockProverConfig::default()),
        ELF,
    )
    .await
    .unwrap();

    let buffer = Buffer::new(prover, 1024);

    let certifier = CertifierClient::try_new(
        Arc::new(pending_store),
        Arc::new(l1_rpc),
        Arc::new(config),
        buffer,
    )
    .await
    .unwrap();

    let result = certifier
        .certify(local_state.clone(), network, height)
        .await;

    assert!(result.is_err());

    scenario.teardown();
}

mockall::mock! {
    L1Rpc {}
    #[async_trait::async_trait]
    impl agglayer_contracts::RollupContract for L1Rpc {
        async fn get_trusted_sequencer_address(
            &self,
            rollup_id: u32,
            proof_signers: std::collections::HashMap<u32, agglayer_types::Address>,
        ) -> Result<agglayer_types::Address, L1RpcError>;

        async fn get_rollup_contract_address(&self, rollup_id: u32) -> Result<agglayer_types::Address, L1RpcError>;

        async fn get_l1_info_root(&self, l1_leaf_count: u32) -> Result<[u8; 32], L1RpcError>;
        fn default_l1_info_tree_entry(&self) -> (u32, [u8; 32]);
        async fn get_prev_pessimistic_root(&self, rollup_id: u32, before_tx: Option<TxHash>) -> Result<[u8; 32], L1RpcError>;
        async fn get_verifier_type(&self, rollup_id: u32) -> Result<agglayer_contracts::rollup::VerifierType, L1RpcError>;
        fn get_rollup_manager_address(&self) -> agglayer_types::Address;
        fn get_event_filter_block_range(&self) -> u64;
    }

    #[async_trait::async_trait]
    impl agglayer_contracts::AggchainContract for L1Rpc {
        async fn get_aggchain_vkey_hash(
            &self,
            rollup_address: agglayer_types::Address,
            aggchain_vkey_selector: u16,
        ) -> Result<VKeyHash, L1RpcError>;

        async fn get_aggchain_hash(
            &self,
            rollup_address: agglayer_types::primitives::Address,
            aggchain_data: Bytes,
        ) -> Result<[u8; 32], L1RpcError>;

        async fn get_multisig_context(
            &self,
            rollup_address: agglayer_types::Address,
        ) -> Result<(Vec<agglayer_types::Address>, usize), L1RpcError>;
    }

    #[async_trait::async_trait]
    impl agglayer_contracts::L1TransactionFetcher for L1Rpc {
        type Provider = alloy::providers::RootProvider<Ethereum>;

        async fn fetch_transaction_receipt(&self, tx_hash: FixedBytes<32>) -> Result<Option<TransactionReceipt>, L1RpcError>;

        fn get_provider(&self) -> &<Self as agglayer_contracts::L1TransactionFetcher>::Provider;
    }

    #[async_trait::async_trait]
    impl Settler for L1Rpc {
        fn decode_contract_revert(error: &ContractError) -> Option<String>;

        async fn verify_pessimistic_trusted_aggregator(
            &self,
            rollup_id: u32,
            l_1_info_tree_leaf_count: u32,
            new_local_exit_root: [u8; 32],
            new_pessimistic_root: [u8; 32],
            proof: Bytes,
            custom_chain_data: Bytes,
            nonce: Option<(u64, u128, Option<u128>)>
        ) -> Result<alloy::providers::PendingTransactionBuilder<Ethereum>, ContractError>;
    }
}
