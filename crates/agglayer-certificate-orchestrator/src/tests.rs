use std::{
    collections::BTreeMap,
    num::NonZeroU64,
    option::Option,
    result::Result,
    sync::{atomic::AtomicU64, Arc, RwLock},
    task::Poll,
};

use agglayer_clock::ClockRef;
use agglayer_config::Config;
use agglayer_storage::{
    columns::{
        latest_proven_certificate_per_network::ProvenCertificate,
        latest_settled_certificate_per_network::SettledCertificate,
    },
    stores::{
        epochs::EpochsStore, pending::PendingStore, per_epoch::PerEpochStore, state::StateStore,
        EpochStoreReader, EpochStoreWriter, PendingCertificateReader, PendingCertificateWriter,
        PerEpochReader, PerEpochWriter, StateReader, StateWriter,
    },
    tests::{
        mocks::{MockEpochsStore, MockPendingStore, MockPerEpochStore, MockStateStore},
        TempDBDir,
    },
};
use agglayer_types::{
    Certificate, CertificateHeader, CertificateId, CertificateIndex, CertificateStatus, Digest,
    EpochNumber, ExecutionMode, Height, LocalNetworkStateData, NetworkId, Proof,
};
use arc_swap::ArcSwap;
use ethers::{
    providers::{MockProvider, PendingTransaction},
    types::H256,
};
use futures_util::poll;
use mocks::MockCertifier;
use pessimistic_proof::{
    local_exit_tree::hasher::Keccak256Hasher, multi_batch_header::MultiBatchHeader,
    LocalNetworkState,
};
use rstest::fixture;
use tokio::sync::{broadcast, mpsc};
use tokio_util::sync::CancellationToken;

use crate::{
    epoch_packer::MockEpochPacker, CertificateInput, CertificateOrchestrator, CertificationError,
    Certifier, CertifierOutput, CertifierResult, EpochPacker, Error,
};

pub(crate) mod mocks;

#[allow(dead_code)]
#[derive(Default)]
pub(crate) struct DummyPendingStore {
    pub(crate) current_epoch: u64,
    pub(crate) pending_certificate: RwLock<BTreeMap<(NetworkId, Height), Certificate>>,
    pub(crate) proofs: RwLock<BTreeMap<CertificateId, Proof>>,
    pub(crate) settled: RwLock<BTreeMap<NetworkId, (Height, CertificateId)>>,
    pub(crate) certificate_per_network: RwLock<BTreeMap<(NetworkId, Height), CertificateId>>,
    pub(crate) certificate_headers: RwLock<BTreeMap<CertificateId, CertificateHeader>>,
    pub(crate) latest_proven_certificate_per_network:
        RwLock<BTreeMap<NetworkId, ProvenCertificate>>,
    pub(crate) is_packed: bool,
}

impl PerEpochReader for DummyPendingStore {
    fn is_epoch_packed(&self) -> bool {
        self.is_packed
    }
    fn get_epoch_number(&self) -> u64 {
        self.current_epoch
    }
    fn get_certificate_at_index(
        &self,
        _index: CertificateIndex,
    ) -> Result<Option<Certificate>, agglayer_storage::error::Error> {
        todo!()
    }
    fn get_proof_at_index(
        &self,
        _index: CertificateIndex,
    ) -> Result<Option<Proof>, agglayer_storage::error::Error> {
        todo!()
    }
    fn get_end_checkpoint(&self) -> BTreeMap<NetworkId, Height> {
        todo!()
    }

    fn get_start_checkpoint(&self) -> &BTreeMap<NetworkId, Height> {
        todo!()
    }

    fn get_end_checkpoint_height_per_network(
        &self,
        _network_id: NetworkId,
    ) -> Result<Option<Height>, agglayer_storage::error::Error> {
        todo!()
    }
}

impl PerEpochWriter for DummyPendingStore {
    fn add_certificate(
        &self,
        _network_id: NetworkId,
        _height: Height,
        _mode: ExecutionMode,
    ) -> Result<(EpochNumber, CertificateIndex), agglayer_storage::error::Error> {
        Ok((0, 0))
    }

    fn start_packing(&self) -> Result<(), agglayer_storage::error::Error> {
        todo!()
    }
}

impl StateReader for DummyPendingStore {
    fn get_active_networks(&self) -> Result<Vec<NetworkId>, agglayer_storage::error::Error> {
        Ok(vec![])
    }

    fn get_latest_settled_certificate_per_network(
        &self,
        _network_id: &NetworkId,
    ) -> Result<Option<(NetworkId, SettledCertificate)>, agglayer_storage::error::Error> {
        todo!()
    }

    fn get_certificate_header(
        &self,
        certificate_id: &CertificateId,
    ) -> Result<Option<CertificateHeader>, agglayer_storage::error::Error> {
        Ok(self
            .certificate_headers
            .read()
            .unwrap()
            .get(certificate_id)
            .cloned())
    }
    fn get_current_settled_height(
        &self,
    ) -> Result<Vec<(NetworkId, SettledCertificate)>, agglayer_storage::error::Error> {
        self.settled
            .read()
            .unwrap()
            .iter()
            .map(|(network_id, (height, id))| {
                Ok((*network_id, SettledCertificate(*id, *height, 0, 0)))
            })
            .collect()
    }

    fn get_certificate_header_by_cursor(
        &self,
        network_id: NetworkId,
        height: Height,
    ) -> Result<Option<agglayer_types::CertificateHeader>, agglayer_storage::error::Error> {
        Ok(self
            .certificate_per_network
            .read()
            .unwrap()
            .get(&(network_id, height))
            .and_then(|id| self.certificate_headers.read().unwrap().get(id).cloned()))
    }

    fn read_local_network_state(
        &self,
        _network_id: NetworkId,
    ) -> Result<Option<LocalNetworkStateData>, agglayer_storage::error::Error> {
        todo!()
    }
}
impl EpochStoreReader for DummyPendingStore {}

impl EpochStoreWriter for DummyPendingStore {
    type PerEpochStore = Self;

    fn open(
        &self,
        _epoch_number: u64,
    ) -> Result<Self::PerEpochStore, agglayer_storage::error::Error> {
        Ok(DummyPendingStore::default())
    }

    fn open_with_start_checkpoint(
        &self,
        _epoch_number: u64,
        _start_checkpoint: BTreeMap<NetworkId, Height>,
    ) -> Result<Self::PerEpochStore, agglayer_storage::error::Error> {
        Ok(DummyPendingStore::default())
    }
}

impl PendingCertificateWriter for DummyPendingStore {
    fn insert_pending_certificate(
        &self,
        network_id: NetworkId,
        height: Height,
        certificate: &agglayer_types::Certificate,
    ) -> Result<(), agglayer_storage::error::Error> {
        self.pending_certificate
            .write()
            .unwrap()
            .insert((network_id, height), certificate.clone());

        Ok(())
    }

    fn insert_generated_proof(
        &self,
        certificate_id: &CertificateId,
        proof: &agglayer_types::Proof,
    ) -> Result<(), agglayer_storage::error::Error> {
        self.proofs
            .write()
            .unwrap()
            .insert(*certificate_id, proof.clone());

        Ok(())
    }

    fn remove_pending_certificate(
        &self,
        network_id: NetworkId,
        height: Height,
    ) -> Result<(), agglayer_storage::error::Error> {
        self.pending_certificate
            .write()
            .unwrap()
            .remove(&(network_id, height));

        Ok(())
    }
    fn remove_generated_proof(
        &self,
        certificate_id: &CertificateId,
    ) -> Result<(), agglayer_storage::error::Error> {
        self.proofs.write().unwrap().remove(certificate_id);

        Ok(())
    }
    fn set_latest_proven_certificate_per_network(
        &self,
        network_id: &NetworkId,
        height: &Height,
        certificate_id: &CertificateId,
    ) -> Result<(), agglayer_storage::error::Error> {
        self.latest_proven_certificate_per_network
            .write()
            .unwrap()
            .insert(
                *network_id,
                ProvenCertificate(*certificate_id, *network_id, *height),
            );

        Ok(())
    }

    fn set_latest_pending_certificate_per_network(
        &self,
        _network_id: &NetworkId,
        _height: &Height,
        _certificate_id: &CertificateId,
    ) -> Result<(), agglayer_storage::error::Error> {
        Ok(())
    }
}

impl StateWriter for DummyPendingStore {
    fn update_settlement_tx_hash(
        &self,
        _certificate_id: &CertificateId,
        _tx_hash: Digest,
    ) -> Result<(), agglayer_storage::error::Error> {
        todo!()
    }

    fn assign_certificate_to_epoch(
        &self,
        _certificate_id: &CertificateId,
        _epoch_number: &EpochNumber,
        _certificate_index: &agglayer_types::CertificateIndex,
    ) -> Result<(), agglayer_storage::error::Error> {
        todo!()
    }

    fn insert_certificate_header(
        &self,
        certificate: &Certificate,
        status: CertificateStatus,
    ) -> Result<(), agglayer_storage::error::Error> {
        self.certificate_per_network.write().unwrap().insert(
            (certificate.network_id, certificate.height),
            certificate.hash(),
        );

        self.certificate_headers.write().unwrap().insert(
            certificate.hash(),
            CertificateHeader {
                network_id: certificate.network_id,
                height: certificate.height,
                epoch_number: None,
                certificate_index: None,
                certificate_id: certificate.hash(),
                prev_local_exit_root: certificate.prev_local_exit_root,
                new_local_exit_root: certificate.new_local_exit_root,
                status,
                metadata: certificate.metadata,
                settlement_tx_hash: None,
            },
        );

        Ok(())
    }

    fn update_certificate_header_status(
        &self,
        certificate_id: &CertificateId,
        status: &CertificateStatus,
    ) -> Result<(), agglayer_storage::error::Error> {
        if let Some(entry) = self
            .certificate_headers
            .write()
            .unwrap()
            .get_mut(certificate_id)
        {
            entry.status = status.clone();
        }

        Ok(())
    }

    fn set_latest_settled_certificate_for_network(
        &self,
        _network_id: &NetworkId,
        _height: &Height,
        _certificate_id: &CertificateId,
        _epoch_number: &EpochNumber,
        _certificate_index: &CertificateIndex,
    ) -> Result<(), agglayer_storage::error::Error> {
        Ok(())
    }

    fn write_local_network_state(
        &self,
        _network_id: &NetworkId,
        _new_state: &LocalNetworkStateData,
        _new_leaves: &[Digest],
    ) -> Result<(), agglayer_storage::error::Error> {
        todo!()
    }
}

impl PendingCertificateReader for DummyPendingStore {
    fn get_latest_proven_certificate_per_network(
        &self,
        _network_id: &NetworkId,
    ) -> Result<Option<(NetworkId, Height, CertificateId)>, agglayer_storage::error::Error> {
        todo!()
    }
    fn get_latest_pending_certificate_for_network(
        &self,
        _network_id: &NetworkId,
    ) -> Result<Option<(CertificateId, Height)>, agglayer_storage::error::Error> {
        todo!()
    }

    fn get_current_proven_height(
        &self,
    ) -> Result<Vec<ProvenCertificate>, agglayer_storage::error::Error> {
        Ok(self
            .latest_proven_certificate_per_network
            .read()
            .unwrap()
            .values()
            .cloned()
            .collect())
    }
    fn get_current_proven_height_for_network(
        &self,
        network_id: &NetworkId,
    ) -> Result<Option<Height>, agglayer_storage::error::Error> {
        Ok(self
            .latest_proven_certificate_per_network
            .read()
            .unwrap()
            .get(network_id)
            .map(|x| x.2))
    }

    fn get_certificate(
        &self,
        network_id: NetworkId,
        height: agglayer_types::Height,
    ) -> Result<Option<agglayer_types::Certificate>, agglayer_storage::error::Error> {
        Ok(self
            .pending_certificate
            .read()
            .unwrap()
            .get(&(network_id, height))
            .cloned())
    }

    fn get_proof(
        &self,
        certificate_id: CertificateId,
    ) -> Result<Option<agglayer_types::Proof>, agglayer_storage::error::Error> {
        Ok(self.proofs.read().unwrap().get(&certificate_id).cloned())
    }

    fn multi_get_certificate(
        &self,
        keys: &[(NetworkId, Height)],
    ) -> Result<Vec<Option<agglayer_types::Certificate>>, agglayer_storage::error::Error> {
        let lock = self.pending_certificate.read().unwrap();

        Ok(keys.iter().map(|key| lock.get(key).cloned()).collect())
    }

    fn multi_get_proof(
        &self,
        keys: &[CertificateId],
    ) -> Result<Vec<Option<agglayer_types::Proof>>, agglayer_storage::error::Error> {
        let lock = self.proofs.read().unwrap();

        Ok(keys.iter().map(|key| lock.get(key).cloned()).collect())
    }
}

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
        StateStore::new_with_path(&config.storage.state_db_path).expect("Unable to create store"),
    );
    let epochs_store = Arc::new(
        EpochsStore::new(
            Arc::new(config),
            0,
            pending_store.clone(),
            state_store.clone(),
        )
        .expect("Unable to create store"),
    );

    let current_epoch = ArcSwap::new(Arc::new(
        epochs_store.open(0).expect("Unable to open epoch"),
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
        StateStore::new_with_path(&config.storage.state_db_path).expect("Unable to create store"),
    );

    let epochs_store = Arc::new(
        EpochsStore::new(
            Arc::new(config),
            0,
            pending_store.clone(),
            state_store.clone(),
        )
        .expect("Unable to create store"),
    );

    let current_epoch = ArcSwap::new(Arc::new(
        epochs_store.open(1).expect("Unable to open epoch"),
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
        .expected_epoch(1)
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

    _ = data_sender.send((1.into(), 1, [0; 32].into())).await;
    let current_epoch = orchestrator.current_epoch.load().clone();
    _ = clock_sender.send(agglayer_clock::Event::EpochEnded(1));

    let _poll = poll!(&mut orchestrator);

    assert!(current_epoch.is_epoch_packed());
}

// A certificate received after an EpochEnded is stored for next epoch
#[tokio::test]
#[ignore]
async fn test_collect_certificates_after_epoch() {
    let path = TempDBDir::new();
    let config = Config::new(&path.path);
    let pending_store = Arc::new(
        PendingStore::new_with_path(&config.storage.pending_db_path)
            .expect("Unable to create store"),
    );
    let state_store = Arc::new(
        StateStore::new_with_path(&config.storage.state_db_path).expect("Unable to create store"),
    );
    let epochs_store = Arc::new(
        EpochsStore::new(
            Arc::new(config),
            0,
            pending_store.clone(),
            state_store.clone(),
        )
        .expect("Unable to create store"),
    );

    let current_epoch = ArcSwap::new(Arc::new(
        epochs_store.open(0).expect("Unable to open epoch"),
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
        .expected_epoch(1)
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

    _ = clock_sender.send(agglayer_clock::Event::EpochEnded(1));
    let _poll = poll!(&mut orchestrator);

    _ = data_sender.send((1.into(), 1, [0; 32].into())).await;

    let _poll = poll!(&mut orchestrator);

    assert!(check_receiver.recv().await.is_some());
}

// If no certificate is received, the orchestrator should send an empty payload
#[tokio::test]
#[ignore]
async fn test_collect_certificates_when_empty() {
    let path = TempDBDir::new();
    let config = Config::new(&path.path);
    let pending_store = Arc::new(
        PendingStore::new_with_path(&config.storage.pending_db_path)
            .expect("Unable to create store"),
    );
    let state_store = Arc::new(
        StateStore::new_with_path(&config.storage.state_db_path).expect("Unable to create store"),
    );
    let epochs_store = Arc::new(
        EpochsStore::new(
            Arc::new(config),
            0,
            pending_store.clone(),
            state_store.clone(),
        )
        .expect("Unable to create store"),
    );

    let current_epoch = ArcSwap::new(Arc::new(
        epochs_store.open(0).expect("Unable to open epoch"),
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
        .expected_epoch(1)
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

    _ = clock_sender.send(agglayer_clock::Event::EpochEnded(1));
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
        StateStore::new_with_path(&config.storage.state_db_path).expect("Unable to create store"),
    );
    let epochs_store = Arc::new(
        EpochsStore::new(
            Arc::new(config),
            0,
            pending_store.clone(),
            state_store.clone(),
        )
        .expect("Unable to create store"),
    );

    let _current_epoch = ArcSwap::new(Arc::new(
        epochs_store.open(0).expect("Unable to open epoch"),
    ));

    let check = Check::builder()
        .pending_store(pending_store.clone())
        .state_store(state_store.clone())
        .executed(check_sender)
        .expected_epoch(1)
        .build();

    ((pending_store, state_store), check_receiver, check)
}

type IMockOrchestrator = CertificateOrchestrator<
    MockEpochPacker,
    MockCertifier,
    MockPendingStore,
    MockEpochsStore,
    MockPerEpochStore,
    MockStateStore,
>;

#[derive(Default, buildstructor::Builder)]
struct MockOrchestrator {
    certifier: Option<MockCertifier>,
    epoch_packer: Option<MockEpochPacker>,
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
            builder.epoch_packer.unwrap_or_else(|| {
                let mut epoch_packer = MockEpochPacker::default();

                epoch_packer.expect_settle_certificate().never();
                epoch_packer.expect_pack().never();

                epoch_packer
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
    expected_epoch: Option<u64>,
}

#[buildstructor::buildstructor]
impl Check {
    #[builder]
    pub(crate) fn new(
        pending_store: Arc<PendingStore>,
        state_store: Arc<StateStore>,
        executed: mpsc::Sender<CertifierOutput>,
        expected_epoch: Option<u64>,
    ) -> Self {
        Self {
            state_store,
            pending_store,
            executed,
            expected_certificate: None,
            expected_proof: None,
            expected_epoch,
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
impl EpochPacker for Check {
    type PerEpochStore = PerEpochStore<PendingStore, StateStore>;
    type Provider = MockProvider;

    async fn transaction_exists(&self, _tx_hash: H256) -> Result<bool, Error> {
        Ok(true)
    }

    async fn recover_settlement(
        &self,
        _: H256,
        certificate_id: CertificateId,
        network_id: NetworkId,
        height: Height,
    ) -> Result<(NetworkId, SettledCertificate), Error> {
        Ok((network_id, SettledCertificate(certificate_id, height, 0, 0)))
    }

    async fn settle_certificate(
        &self,
        certificate_id: CertificateId,
    ) -> Result<(NetworkId, SettledCertificate), Error> {
        Ok((0.into(), SettledCertificate(certificate_id, 0, 0, 0)))
    }

    async fn watch_and_update(
        &self,
        _pending_tx: PendingTransaction<'_, Self::Provider>,
        certificate_id: CertificateId,
        network_id: NetworkId,
        height: Height,
    ) -> Result<(NetworkId, SettledCertificate), Error> {
        Ok((network_id, SettledCertificate(certificate_id, height, 0, 0)))
    }

    async fn pack(&self, epoch: Arc<Self::PerEpochStore>) -> Result<(), Error> {
        let epoch = epoch.get_epoch_number();
        if let Some(expected_epoch) = self.expected_epoch {
            assert_eq!(epoch, expected_epoch);
        }
        // if let Some(expected_certificates_len) = self.expected_certificates_len {
        //     assert!(to_pack.into_iter().count() == expected_certificates_len);
        // }

        _ = self.executed.try_send(CertifierOutput {
            certificate: self
                .expected_certificate
                .clone()
                .unwrap_or_else(|| Certificate::new_for_test(1.into(), 0)),
            height: 0,
            new_state: LocalNetworkStateData::default(),
            network: 1.into(),
        });

        Ok(())
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
        };
        _ = self.executed.try_send(result.clone());
        Ok(result)
    }

    async fn witness_execution(
        &self,
        _certificate: &agglayer_types::Certificate,
        _state: &mut LocalNetworkStateData,
    ) -> Result<(MultiBatchHeader<Keccak256Hasher>, LocalNetworkState), CertificationError> {
        Err(CertificationError::InternalError(
            "unimplemented".to_string(),
        ))
    }
}
