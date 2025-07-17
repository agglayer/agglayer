use std::{sync::Arc, thread, time::Duration};

use agglayer_certificate_orchestrator::Certifier;
use agglayer_config::Config;
use agglayer_contracts::{aggchain::AggchainVkeyHash, L1RpcError, Settler};
use agglayer_prover::fake::FakeProver;
use agglayer_storage::tests::{mocks::MockPendingStore, TempDBDir};
use agglayer_types::{Height, LocalNetworkStateData, NetworkId};
use alloy::{
    contract::Error as ContractError,
    network::Ethereum,
    primitives::{Bytes, FixedBytes},
    rpc::types::TransactionReceipt,
};
use fail::FailScenario;
use mockall::predicate::{always, eq};
use pessimistic_proof_test_suite::forest::Forest;
use prover_config::ProverType;
use tokio_util::sync::CancellationToken;

use crate::{CertifierClient, ELF};

#[rstest::rstest]
#[test_log::test(tokio::test)]
async fn happy_path() {
    let scenario = FailScenario::setup();
    let base_path = TempDBDir::new();
    let config = Config::new(&base_path.path);

    let mut pending_store = MockPendingStore::new();
    let mut l1_rpc = MockL1Rpc::new();
    let prover_config = agglayer_prover_config::ProverConfig::default();

    // spawning fake prover as we don't want to hit SP1
    let fake_prover = FakeProver::new(ELF);
    let endpoint = prover_config.grpc_endpoint;
    let cancellation = CancellationToken::new();

    FakeProver::spawn_at(fake_prover, endpoint, cancellation.clone())
        .await
        .unwrap();

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

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
        .returning(|_| Ok(alloy::primitives::Address::default().into()));

    l1_rpc
        .expect_default_l1_info_tree_entry()
        .once()
        .returning(|| (0u32, [1u8; 32]));

    l1_rpc
        .expect_get_prev_pessimistic_root()
        .once()
        .returning(|_| Ok([0u8; 32]));

    fail::cfg(
        "notifier::certifier::certify::before_verifying_proof",
        "return()",
    )
    .unwrap();

    let certifier = CertifierClient::try_new(
        config.prover_entrypoint.clone(),
        Arc::new(pending_store),
        Arc::new(l1_rpc),
        Arc::new(config),
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
    let mut config = Config::new(&base_path.path);

    let mut pending_store = MockPendingStore::new();
    let mut l1_rpc = MockL1Rpc::new();
    let prover_config = agglayer_prover_config::ProverConfig {
        grpc_endpoint: next_available_addr(),
        primary_prover: ProverType::CpuProver(prover_config::CpuProverConfig {
            proving_timeout: Duration::from_secs(1),
            ..Default::default()
        }),
        ..Default::default()
    };

    config.prover_entrypoint = format!(
        "http://{}:{}",
        prover_config.grpc_endpoint.ip(),
        prover_config.grpc_endpoint.port()
    );

    let prover_config = Arc::new(prover_config);

    let cancellation = CancellationToken::new();
    let prover_cancellation_token = cancellation.clone();

    thread::spawn(move || {
        agglayer_prover::start_prover(prover_config, prover_cancellation_token, ELF);
    });
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

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
        .returning(|_| Ok(alloy::primitives::Address::default().into()));

    l1_rpc
        .expect_default_l1_info_tree_entry()
        .once()
        .returning(|| (0u32, [1u8; 32]));

    l1_rpc
        .expect_get_prev_pessimistic_root()
        .once()
        .returning(|_| Ok([0u8; 32]));

    fail::cfg(
        "notifier::certifier::certify::before_verifying_proof",
        "return()",
    )
    .unwrap();

    let certifier = CertifierClient::try_new(
        config.prover_entrypoint.clone(),
        Arc::new(pending_store),
        Arc::new(l1_rpc),
        Arc::new(config),
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
        type P = alloy::providers::RootProvider<Ethereum>;

        async fn get_trusted_sequencer_address(
            &self,
            rollup_id: u32,
            proof_signers: std::collections::HashMap<u32, agglayer_types::Address>,
        ) -> Result<agglayer_types::Address, L1RpcError>;

        async fn get_rollup_contract_address(&self, rollup_id: u32) -> Result<agglayer_types::Address, L1RpcError>;

        async fn get_l1_info_root(&self, l1_leaf_count: u32) -> Result<[u8; 32], L1RpcError>;
        fn default_l1_info_tree_entry(&self) -> (u32, [u8; 32]);
        async fn get_prev_pessimistic_root(&self, rollup_id: u32) -> Result<[u8; 32], L1RpcError>;
        async fn get_verifier_type(&self, rollup_id: u32) -> Result<agglayer_contracts::rollup::VerifierType, L1RpcError>;
    }

    #[async_trait::async_trait]
    impl agglayer_contracts::AggchainContract for L1Rpc {
        type M = alloy::providers::RootProvider<Ethereum>;

        async fn get_aggchain_vkey_hash(
            &self,
            rollup_address: agglayer_types::Address,
            aggchain_vkey_selector: u16,
        ) -> Result<AggchainVkeyHash, L1RpcError>;

        async fn get_aggchain_hash(
            &self,
            rollup_address: agglayer_types::primitives::Address,
            aggchain_data: Bytes,
        ) -> Result<[u8; 32], L1RpcError>;
    }

    #[async_trait::async_trait]
    impl agglayer_contracts::L1TransactionFetcher for L1Rpc {
        type Provider = alloy::providers::RootProvider<Ethereum>;

        async fn fetch_transaction_receipt(&self, tx_hash: FixedBytes<32>) -> Result<TransactionReceipt, L1RpcError>;

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
        ) -> Result<alloy::providers::PendingTransactionBuilder<Ethereum>, ContractError>;
    }
}

fn next_available_addr() -> std::net::SocketAddr {
    use std::net::{TcpListener, TcpStream};

    assert!(
        std::env::var("NEXTEST").is_ok(),
        "Due to concurrency issues, the rpc tests have to be run under `cargo nextest`",
    );

    let host = "127.0.0.1";
    // Request a random available port from the OS
    let listener = TcpListener::bind((host, 0)).expect("Can't bind to an available port");
    let addr = listener.local_addr().expect("Can't find an available port");

    // Create and accept a connection (which we'll promptly drop) in order to force
    // the port into the TIME_WAIT state, ensuring that the port will be
    // reserved from some limited amount of time (roughly 60s on some Linux
    // systems)
    let _sender = TcpStream::connect(addr).expect("Can't connect to an available port");
    let _incoming = listener.accept().expect("Can't accept an available port");

    addr
}
