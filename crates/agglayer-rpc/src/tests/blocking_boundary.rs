use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Condvar, Mutex,
    },
    time::Duration,
};

use agglayer_config::Config;
use agglayer_contracts::{AggchainContract, L1RpcError, L1TransactionFetcher, RollupContract};
use agglayer_storage::tests::mocks::{
    MockDebugStore, MockEpochsStore, MockPendingStore, MockStateStore,
};
use agglayer_types::{Address, Certificate, CertificateId, Digest, Height, NetworkId};
use alloy::{
    network::Ethereum,
    providers::{mock::Asserter, ProviderBuilder, RootProvider},
    rpc::types::TransactionReceipt,
};

#[tokio::test(flavor = "current_thread")]
async fn certificate_header_query_keeps_the_runtime_responsive() {
    let certificate_id = CertificateId::new(Digest([1; 32]));
    let (storage_started, storage_started_rx) = tokio::sync::oneshot::channel();
    let release = Arc::new((Mutex::new(false), Condvar::new()));
    let release_for_storage = release.clone();
    let storage_timed_out = Arc::new(AtomicBool::new(false));
    let storage_timed_out_from_query = storage_timed_out.clone();

    let mut state_store = MockStateStore::new();
    state_store
        .expect_get_certificate_header()
        .return_once(move |_| {
            storage_started
                .send(())
                .expect("test task should wait for the storage call");

            let (released, wake) = &*release_for_storage;
            // The timeout only prevents a regressed synchronous call from hanging the
            // suite; the test task releases this immediately when the runtime
            // remains responsive.
            let (_released, timeout) = wake
                .wait_timeout_while(
                    released
                        .lock()
                        .expect("release lock should not be poisoned"),
                    Duration::from_secs(10),
                    |released| !*released,
                )
                .expect("release lock should not be poisoned");
            storage_timed_out_from_query.store(timeout.timed_out(), Ordering::SeqCst);
            Ok(None)
        });

    let asserter = Asserter::new();
    let service = Arc::new(crate::AgglayerService::new(
        tokio::sync::mpsc::channel(1).0,
        Arc::new(MockPendingStore::new()),
        Arc::new(state_store),
        Arc::new(MockDebugStore::new()),
        Arc::new(MockEpochsStore::new()),
        Arc::new(Config::default()),
        Arc::new(ProviderBuilder::new().connect_mocked_client(asserter)),
    ));

    let query = tokio::spawn(async move { service.fetch_certificate_header(certificate_id).await });

    storage_started_rx
        .await
        .expect("blocking storage task should start");
    let (released, wake) = &*release;
    *released
        .lock()
        .expect("release lock should not be poisoned") = true;
    wake.notify_one();

    assert!(matches!(
        query.await.expect("query task should not panic"),
        Err(crate::CertificateRetrievalError::NotFound { .. })
    ));
    assert!(
        !storage_timed_out.load(Ordering::SeqCst),
        "the current-thread runtime did not run while storage was blocked"
    );
}

/// L1 stub for `send_certificate` tests: the ECDSA signature check resolves
/// the trusted sequencer from the configured proof signers, and nothing else
/// may touch L1.
struct StubL1Rpc;

#[async_trait::async_trait]
impl RollupContract for StubL1Rpc {
    async fn get_trusted_sequencer_address(
        &self,
        rollup_id: u32,
        proof_signers: HashMap<u32, Address>,
    ) -> Result<Address, L1RpcError> {
        Ok(*proof_signers
            .get(&rollup_id)
            .expect("the test must configure a proof signer"))
    }

    async fn get_rollup_contract_address(&self, _rollup_id: u32) -> Result<Address, L1RpcError> {
        unreachable!("send_certificate only resolves the trusted sequencer")
    }

    async fn get_prev_pessimistic_root(
        &self,
        _rollup_id: u32,
        _before_tx: Option<alloy::primitives::TxHash>,
    ) -> Result<[u8; 32], L1RpcError> {
        unreachable!("send_certificate only resolves the trusted sequencer")
    }

    async fn get_l1_info_root(&self, _l1_leaf_count: u32) -> Result<[u8; 32], L1RpcError> {
        unreachable!("send_certificate only resolves the trusted sequencer")
    }

    async fn get_verifier_type(
        &self,
        _rollup_id: u32,
    ) -> Result<agglayer_contracts::rollup::VerifierType, L1RpcError> {
        unreachable!("send_certificate only resolves the trusted sequencer")
    }

    fn default_l1_info_tree_entry(&self) -> (u32, [u8; 32]) {
        unreachable!("send_certificate only resolves the trusted sequencer")
    }

    fn get_rollup_manager_address(&self) -> Address {
        unreachable!("send_certificate only resolves the trusted sequencer")
    }

    fn get_event_filter_block_range(&self) -> u64 {
        unreachable!("send_certificate only resolves the trusted sequencer")
    }
}

#[async_trait::async_trait]
impl AggchainContract for StubL1Rpc {
    async fn get_aggchain_vkey_hash(
        &self,
        _rollup_address: Address,
        _aggchain_vkey_selector: u16,
    ) -> Result<agglayer_contracts::aggchain::VKeyHash, L1RpcError> {
        unreachable!("send_certificate only resolves the trusted sequencer")
    }

    async fn get_aggchain_hash(
        &self,
        _rollup_address: Address,
        _aggchain_data: alloy::primitives::Bytes,
        _before_tx_hash: Option<alloy::primitives::TxHash>,
    ) -> Result<[u8; 32], L1RpcError> {
        unreachable!("send_certificate only resolves the trusted sequencer")
    }

    async fn get_multisig_context(
        &self,
        _rollup_address: Address,
    ) -> Result<(Vec<Address>, usize), L1RpcError> {
        unreachable!("send_certificate only resolves the trusted sequencer")
    }
}

#[async_trait::async_trait]
impl L1TransactionFetcher for StubL1Rpc {
    type Provider = RootProvider<Ethereum>;

    async fn fetch_transaction_receipt(
        &self,
        _tx_hash: agglayer_types::SettlementTxHash,
    ) -> Result<Option<TransactionReceipt>, L1RpcError> {
        unreachable!("send_certificate only resolves the trusted sequencer")
    }

    fn get_provider(&self) -> &Self::Provider {
        unreachable!("send_certificate only resolves the trusted sequencer")
    }
}

#[tokio::test]
async fn dropped_send_certificate_still_notifies_orchestrator() {
    let network_id = NetworkId::new(1);
    let certificate = Certificate::new_for_test(network_id, Height::ZERO);
    let certificate_id = certificate.hash();

    let mut config = Config::default();
    config
        .proof_signers
        .insert(1, Certificate::wallet_for_test(network_id).address().into());

    // The replacement check finds no pre-existing certificate.
    let mut pending_store = MockPendingStore::new();
    pending_store
        .expect_get_certificate()
        .return_once(|_, _| Ok(None));
    let mut state_store = MockStateStore::new();
    state_store
        .expect_get_certificate_header_by_cursor()
        .return_once(|_, _| Ok(None));

    // Hold the first persistence write until the submission future has been
    // cancelled mid-persistence.
    let (persistence_started, persistence_started_rx) = tokio::sync::oneshot::channel();
    let release = Arc::new((Mutex::new(false), Condvar::new()));
    let release_for_storage = release.clone();
    pending_store
        .expect_insert_pending_certificate()
        .return_once(move |_, _, _| {
            persistence_started
                .send(())
                .expect("test task should wait for the persistence call");

            let (released, wake) = &*release_for_storage;
            let (_released, timeout) = wake
                .wait_timeout_while(
                    released
                        .lock()
                        .expect("release lock should not be poisoned"),
                    Duration::from_secs(10),
                    |released| !*released,
                )
                .expect("release lock should not be poisoned");
            assert!(!timeout.timed_out(), "the test task never released storage");
            Ok(())
        });
    state_store
        .expect_insert_certificate_header()
        .return_once(|_, _| Ok(()));
    let mut debug_store = MockDebugStore::new();
    debug_store.expect_add_certificate().return_once(|_| Ok(()));

    let (certificate_sender, mut certificate_receiver) = tokio::sync::mpsc::channel(1);
    let service = Arc::new(crate::AgglayerService::new(
        certificate_sender,
        Arc::new(pending_store),
        Arc::new(state_store),
        Arc::new(debug_store),
        Arc::new(MockEpochsStore::new()),
        Arc::new(config),
        Arc::new(StubL1Rpc),
    ));

    let submission = tokio::spawn(async move { service.send_certificate(certificate).await });

    persistence_started_rx
        .await
        .expect("blocking persistence task should start");

    // Cancel the submission future while persistence is mid-write, and wait
    // for the cancellation to complete.
    submission.abort();
    assert!(submission
        .await
        .expect_err("the submission future should be cancelled")
        .is_cancelled());
    assert!(
        certificate_receiver.try_recv().is_err(),
        "no notification may be sent while persistence is incomplete"
    );

    // Release storage: the detached persistence task must finish the writes
    // and still notify the orchestrator.
    let (released, wake) = &*release;
    *released
        .lock()
        .expect("release lock should not be poisoned") = true;
    wake.notify_one();

    let notification = tokio::time::timeout(Duration::from_secs(10), certificate_receiver.recv())
        .await
        .expect("the orchestrator notification should arrive after persistence")
        .expect("the notification permit should outlive the cancelled caller");
    assert_eq!(notification, (network_id, Height::ZERO, certificate_id));
}
