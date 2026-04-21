use std::{
    num::NonZeroU64,
    sync::{atomic::AtomicU64, Arc},
    task::Poll,
};

use agglayer_clock::ClockRef;
use agglayer_config::Config;
use agglayer_storage::{
    backup::BackupClient,
    stores::{
        epochs::EpochsStore, pending::PendingStore, state::StateStore, EpochStoreWriter,
        PendingCertificateReader, PendingCertificateWriter, PerEpochReader,
    },
    tests::{
        mocks::{MockEpochsStore, MockPendingStore, MockPerEpochStore, MockStateStore},
        TempDBDir,
    },
};
use agglayer_types::{
    Certificate, CertificateId, CertificateIndex, Digest, EpochNumber, Height,
    LocalNetworkStateData, NetworkId, Proof, SettlementTxHash,
};
use arc_swap::ArcSwap;
use futures_util::poll;
use mocks::MockCertifier;
use pessimistic_proof::{
    multi_batch_header::MultiBatchHeader, LocalNetworkState, PessimisticProofOutput,
};
use rstest::fixture;
use tokio::sync::{broadcast, mpsc};
use tokio_util::sync::CancellationToken;

use crate::{
    settlement_client::{MockProvider, MockSettlementClient, SettlementClient},
    CertificateInput, CertificateOrchestrator, CertificationError, Certifier, CertifierOutput,
    CertifierResult, Error, NonceInfo,
};

pub(crate) mod mocks;

// CertificateOrchestrator can be stopped
#[tokio::test]
async fn test_certificate_orchestrator_can_stop() {
    let path = TempDBDir::new();
    let config = Config::new(&path.path);
    let pending_store = Arc::new(
        PendingStore::new_with_path(&config.storage.pending_db_path)
            .expect("Unable to create store"),
    );
    let state_store = Arc::new(
        StateStore::new_with_path(&config.storage.state_db_path, BackupClient::noop())
            .expect("Unable to create store"),
    );
    let epochs_store = Arc::new(
        EpochsStore::new(
            Arc::new(config),
            pending_store.clone(),
            state_store.clone(),
            BackupClient::noop(),
        )
        .expect("Unable to create store"),
    );

    let current_epoch = ArcSwap::new(Arc::new(
        epochs_store
            .open(EpochNumber::ZERO)
            .expect("Unable to open epoch"),
    ));

    let (clock_sender, _receiver) = broadcast::channel(1);
    let clock = ClockRef::new(
        clock_sender,
        Arc::new(AtomicU64::new(0)),
        Arc::new(NonZeroU64::new(1).unwrap()),
    );

    let (_data_sender, data_receiver) = mpsc::channel(10);
    let cancellation_token = CancellationToken::new();

    let (check_sender, mut check_receiver) = mpsc::channel(1);
    let check = Check::builder()
        .pending_store(pending_store.clone())
        .state_store(state_store.clone())
        .executed(check_sender)
        .build();

    let mut orchestrator = CertificateOrchestrator::try_new(
        clock,
        data_receiver,
        cancellation_token.clone(),
        check.clone(),
        check.clone(),
        pending_store.clone(),
        epochs_store,
        Arc::new(current_epoch),
        state_store.clone(),
    )
    .expect("Unable to create orchestrator");

    cancellation_token.cancel();

    assert!(matches!(poll!(&mut orchestrator), Poll::Ready(())));

    assert!(check_receiver.try_recv().is_err());
}

// Can collect certificates and pack them at the end of an epoch
#[test_log::test(tokio::test)]
async fn test_collect_certificates() {
    let path = TempDBDir::new();
    let config = Config::new(&path.path);
    let pending_store = Arc::new(
        PendingStore::new_with_path(&config.storage.pending_db_path)
            .expect("Unable to create store"),
    );
    let state_store = Arc::new(
        StateStore::new_with_path(&config.storage.state_db_path, BackupClient::noop())
            .expect("Unable to create store"),
    );

    let epochs_store = Arc::new(
        EpochsStore::new(
            Arc::new(config),
            pending_store.clone(),
            state_store.clone(),
            BackupClient::noop(),
        )
        .expect("Unable to create store"),
    );

    let current_epoch = ArcSwap::new(Arc::new(
        epochs_store
            .open(EpochNumber::new(1))
            .expect("Unable to open epoch"),
    ));
    let (clock_sender, _receiver) = broadcast::channel(1);
    let clock = ClockRef::new(
        clock_sender.clone(),
        Arc::new(AtomicU64::new(0)),
        Arc::new(NonZeroU64::new(1).unwrap()),
    );
    let (data_sender, data_receiver) = mpsc::channel(10);
    let cancellation_token = CancellationToken::new();

    let (check_sender, _check_receiver) = mpsc::channel(1);
    let check = Check::builder()
        .pending_store(pending_store.clone())
        .state_store(state_store.clone())
        .executed(check_sender)
        .build();

    let mut orchestrator = CertificateOrchestrator::try_new(
        clock,
        data_receiver,
        cancellation_token,
        check.clone(),
        check.clone(),
        pending_store.clone(),
        epochs_store,
        Arc::new(current_epoch),
        state_store.clone(),
    )
    .expect("Unable to create orchestrator");

    _ = data_sender
        .send((1.into(), Height::new(1), CertificateId::new([0; 32].into())))
        .await;
    let current_epoch = orchestrator.current_epoch.load().clone();
    _ = clock_sender.send(agglayer_clock::Event::EpochEnded(EpochNumber::new(1)));

    let _poll = poll!(&mut orchestrator);

    assert!(current_epoch.is_epoch_packed());
}

// A certificate received after an EpochEnded is stored for next epoch
#[tokio::test]
#[ignore = "certificate is not inserted into pending store before the notification is sent, \
            causing Check::certify to fail with CertificateNotFound"]
async fn test_collect_certificates_after_epoch() {
    let path = TempDBDir::new();
    let config = Config::new(&path.path);
    let pending_store = Arc::new(
        PendingStore::new_with_path(&config.storage.pending_db_path)
            .expect("Unable to create store"),
    );
    let state_store = Arc::new(
        StateStore::new_with_path(&config.storage.state_db_path, BackupClient::noop())
            .expect("Unable to create store"),
    );
    let epochs_store = Arc::new(
        EpochsStore::new(
            Arc::new(config),
            pending_store.clone(),
            state_store.clone(),
            BackupClient::noop(),
        )
        .expect("Unable to create store"),
    );

    let current_epoch = ArcSwap::new(Arc::new(
        epochs_store
            .open(EpochNumber::ZERO)
            .expect("Unable to open epoch"),
    ));
    let (clock_sender, _receiver) = broadcast::channel(1);
    let clock = ClockRef::new(
        clock_sender.clone(),
        Arc::new(AtomicU64::new(0)),
        Arc::new(NonZeroU64::new(1).unwrap()),
    );

    let (data_sender, data_receiver) = mpsc::channel(10);
    let cancellation_token = CancellationToken::new();

    let (check_sender, mut check_receiver) = mpsc::channel(1);
    let check = Check::builder()
        .pending_store(pending_store.clone())
        .state_store(state_store.clone())
        .executed(check_sender)
        .build();

    let mut orchestrator = CertificateOrchestrator::try_new(
        clock,
        data_receiver,
        cancellation_token,
        check.clone(),
        check.clone(),
        pending_store.clone(),
        epochs_store,
        Arc::new(current_epoch),
        state_store.clone(),
    )
    .expect("Unable to create orchestrator");

    _ = clock_sender.send(agglayer_clock::Event::EpochEnded(EpochNumber::new(1)));
    let _poll = poll!(&mut orchestrator);

    _ = data_sender
        .send((1.into(), Height::new(1), CertificateId::new([0; 32].into())))
        .await;

    let _poll = poll!(&mut orchestrator);

    assert!(check_receiver.recv().await.is_some());
}

// If no certificate is received, the orchestrator should send an empty payload
#[tokio::test]
#[ignore = "orchestrator no longer triggers certifier for empty epochs, test expectation needs to \
            be redesigned"]
async fn test_collect_certificates_when_empty() {
    let path = TempDBDir::new();
    let config = Config::new(&path.path);
    let pending_store = Arc::new(
        PendingStore::new_with_path(&config.storage.pending_db_path)
            .expect("Unable to create store"),
    );
    let state_store = Arc::new(
        StateStore::new_with_path(&config.storage.state_db_path, BackupClient::noop())
            .expect("Unable to create store"),
    );
    let epochs_store = Arc::new(
        EpochsStore::new(
            Arc::new(config),
            pending_store.clone(),
            state_store.clone(),
            BackupClient::noop(),
        )
        .expect("Unable to create store"),
    );

    let current_epoch = ArcSwap::new(Arc::new(
        epochs_store
            .open(EpochNumber::ZERO)
            .expect("Unable to open epoch"),
    ));
    let (clock_sender, _receiver) = broadcast::channel(1);

    let clock = ClockRef::new(
        clock_sender.clone(),
        Arc::new(AtomicU64::new(0)),
        Arc::new(NonZeroU64::new(1).unwrap()),
    );

    let (_data_sender, data_receiver) = mpsc::channel(10);
    let cancellation_token = CancellationToken::new();

    let (check_sender, mut check_receiver) = mpsc::channel(1);
    let check = Check::builder()
        .pending_store(pending_store.clone())
        .state_store(state_store.clone())
        .executed(check_sender)
        .build();

    let mut orchestrator = CertificateOrchestrator::try_new(
        clock,
        data_receiver,
        cancellation_token,
        check.clone(),
        check.clone(),
        pending_store.clone(),
        epochs_store,
        Arc::new(current_epoch),
        state_store.clone(),
    )
    .expect("Unable to create orchestrator");

    _ = clock_sender.send(agglayer_clock::Event::EpochEnded(EpochNumber::new(1)));
    let _poll = poll!(&mut orchestrator);

    assert!(check_receiver.recv().await.is_some());
}

#[fixture]
pub(crate) fn clock() -> ClockRef {
    let (clock_sender, _receiver) = broadcast::channel(1);

    ClockRef::new(
        clock_sender,
        Arc::new(AtomicU64::new(0)),
        Arc::new(NonZeroU64::new(1).unwrap()),
    )
}

#[fixture]
fn check() -> (
    (Arc<PendingStore>, Arc<StateStore>),
    mpsc::Receiver<CertifierOutput>,
    Check,
) {
    let (check_sender, check_receiver) = mpsc::channel(1);
    let path = TempDBDir::new();
    let config = Config::new(&path.path);
    let pending_store = Arc::new(
        PendingStore::new_with_path(&config.storage.pending_db_path)
            .expect("Unable to create store"),
    );
    let state_store = Arc::new(
        StateStore::new_with_path(&config.storage.state_db_path, BackupClient::noop())
            .expect("Unable to create store"),
    );
    let epochs_store = Arc::new(
        EpochsStore::new(
            Arc::new(config),
            pending_store.clone(),
            state_store.clone(),
            BackupClient::noop(),
        )
        .expect("Unable to create store"),
    );

    let _current_epoch = ArcSwap::new(Arc::new(
        epochs_store
            .open(EpochNumber::ZERO)
            .expect("Unable to open epoch"),
    ));

    let check = Check::builder()
        .pending_store(pending_store.clone())
        .state_store(state_store.clone())
        .executed(check_sender)
        .build();

    ((pending_store, state_store), check_receiver, check)
}

type IMockOrchestrator = CertificateOrchestrator<
    MockSettlementClient,
    MockCertifier,
    MockPendingStore,
    MockEpochsStore,
    MockPerEpochStore,
    MockStateStore,
>;

#[derive(Default, buildstructor::Builder)]
struct MockOrchestrator {
    certifier: Option<MockCertifier>,
    settlement_client: Option<MockSettlementClient>,
    pending_store: Option<MockPendingStore>,
    epochs_store: Option<MockEpochsStore>,
    state_store: Option<MockStateStore>,
    current_epoch: Option<MockPerEpochStore>,
}

type SenderAndClockRef = (mpsc::Sender<(NetworkId, Height, CertificateId)>, ClockRef);

#[fixture]
pub(crate) fn create_orchestrator_mock(
    #[default(MockOrchestrator::default())] builder: MockOrchestrator,
    clock: ClockRef,
) -> (SenderAndClockRef, IMockOrchestrator) {
    let (data_sender, data_receiver) = mpsc::channel(10);
    let cancellation_token = CancellationToken::new();
    let pending_store = Arc::new(builder.pending_store.unwrap_or_else(|| {
        let mut pending_store = MockPendingStore::default();

        pending_store
            .expect_get_current_proven_height()
            .returning(|| Ok(vec![]));

        pending_store
    }));
    let epochs_store = Arc::new(builder.epochs_store.unwrap_or_default());
    let current_epoch = ArcSwap::new(Arc::new(builder.current_epoch.unwrap_or_default()));
    let state_store = Arc::new(builder.state_store.unwrap_or_default());

    (
        (data_sender, clock.clone()),
        CertificateOrchestrator::try_new(
            clock,
            data_receiver,
            cancellation_token,
            builder.settlement_client.unwrap_or_else(|| {
                let mut settlement_client = MockSettlementClient::default();

                settlement_client
                    .expect_submit_certificate_settlement()
                    .never();

                settlement_client
            }),
            builder.certifier.unwrap_or_else(|| {
                let mut certifier = MockCertifier::default();

                certifier.expect_certify().never();

                certifier
            }),
            pending_store,
            epochs_store,
            Arc::new(current_epoch),
            state_store,
        )
        .expect("Unable to create orchestrator"),
    )
}

#[derive(Clone)]
pub(crate) struct Check {
    pending_store: Arc<PendingStore>,
    #[allow(unused)]
    state_store: Arc<StateStore>,
    expected_certificate: Option<Certificate>,
    #[allow(unused)]
    expected_proof: Option<Proof>,
    executed: mpsc::Sender<CertifierOutput>,
}

#[buildstructor::buildstructor]
impl Check {
    #[builder]
    pub(crate) fn new(
        pending_store: Arc<PendingStore>,
        state_store: Arc<StateStore>,
        executed: mpsc::Sender<CertifierOutput>,
    ) -> Self {
        Self {
            state_store,
            pending_store,
            executed,
            expected_certificate: None,
            expected_proof: None,
        }
    }

    #[allow(unused)]
    pub fn with_certificate(mut self, certificate: Certificate) -> Self {
        self.expected_certificate = Some(certificate);
        self
    }

    #[allow(unused)]
    pub fn with_proof(mut self, proof: Proof) -> Self {
        self.expected_proof = Some(proof);
        self
    }
}

#[async_trait::async_trait]
impl SettlementClient for Check {
    type Provider = MockProvider;

    async fn submit_certificate_settlement(
        &self,
        _certificate_id: CertificateId,
        _nonce_info: Option<NonceInfo>,
    ) -> Result<SettlementTxHash, Error> {
        Ok(SettlementTxHash::for_tests())
    }

    /// Watch for the transaction to be mined and update the certificate
    /// accordingly
    async fn wait_for_settlement(
        &self,
        _settlement_tx_hash: SettlementTxHash,
        _certificate_id: CertificateId,
    ) -> Result<(EpochNumber, CertificateIndex), Error> {
        Ok((EpochNumber::ZERO, CertificateIndex::ZERO))
    }

    fn get_provider(&self) -> &Self::Provider {
        unimplemented!("get_provider not needed in tests")
    }

    async fn fetch_last_settled_pp_root(
        &self,
        _network_id: NetworkId,
    ) -> Result<Option<([u8; 32], SettlementTxHash)>, Error> {
        Ok(None)
    }

    async fn fetch_settlement_nonce(
        &self,
        _settlement_tx_hash: SettlementTxHash,
    ) -> Result<Option<NonceInfo>, Error> {
        Ok(None)
    }

    async fn fetch_settlement_receipt_status(
        &self,
        _settlement_tx_hash: SettlementTxHash,
    ) -> Result<crate::TxReceiptStatus, Error> {
        Ok(crate::TxReceiptStatus::TxSuccessful)
    }
}

impl CertificateInput for () {
    fn network_id(&self) -> NetworkId {
        NetworkId::new(0)
    }
}

#[async_trait::async_trait]
impl Certifier for Check {
    async fn certify(
        &self,
        local_state: LocalNetworkStateData,
        network_id: NetworkId,
        height: Height,
    ) -> CertifierResult {
        // TODO: check whether the initial state is the expected one

        let certificate = self
            .pending_store
            .get_certificate(network_id, height)
            .expect("Storage failure: Unable to get certificate")
            .ok_or(CertificationError::CertificateNotFound(network_id, height))
            .expect("Certificate not found");

        let certificate_id = certificate.hash();

        let proof = Proof::dummy();
        self.pending_store
            .insert_generated_proof(&certificate_id, &proof)
            .expect("Storage failure: Unable to insert proof");

        let result = CertifierOutput {
            certificate,
            height,
            new_state: local_state,
            network: network_id,
            new_pp_root: Digest::ZERO,
        };
        _ = self.executed.try_send(result.clone());
        Ok(result)
    }

    async fn witness_generation(
        &self,
        _certificate: &agglayer_types::Certificate,
        _state: &mut LocalNetworkStateData,
        _certificate_tx_hash: Option<Digest>,
    ) -> Result<(MultiBatchHeader, LocalNetworkState, PessimisticProofOutput), CertificationError>
    {
        Err(CertificationError::InternalError(
            "unimplemented".to_string(),
        ))
    }
}
