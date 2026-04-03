use std::{sync::Arc, time::Duration};

use agglayer_certificate_orchestrator::Certifier;
use agglayer_config::Config;
use agglayer_storage::tests::{mocks::MockPendingStore, TempDBDir};
use agglayer_types::{Address, Height, LocalNetworkStateData, NetworkId};
use fail::FailScenario;
use mockall::predicate::{always, eq};
use pessimistic_proof_test_suite::forest::Forest;
use prover_config::{MockProverConfig, ProverType};
use tower::buffer::Buffer;

use crate::{testutils::MockL1Rpc, CertifierClient, ELF};

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

    let buffer = Buffer::new(prover, config.prover_buffer_size);

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

    let buffer = Buffer::new(prover, config.prover_buffer_size);

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


