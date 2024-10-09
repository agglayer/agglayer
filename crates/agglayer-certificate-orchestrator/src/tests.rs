use std::{
    collections::BTreeMap,
    option::Option,
    result::Result,
    sync::{Arc, RwLock},
    task::Poll,
};

use agglayer_storage::{
    columns::latest_proven_certificate_per_network::ProvenCertificate,
    stores::{
        EpochStoreReader, EpochStoreWriter, PendingCertificateReader, PendingCertificateWriter,
        PerEpochReader, PerEpochWriter, StateReader, StateWriter,
    },
    tests::mocks::{MockEpochsStore, MockPendingStore, MockPerEpochStore, MockStateStore},
};
use agglayer_types::{
    Certificate, CertificateHeader, CertificateId, CertificateIndex, CertificateStatus,
    EpochNumber, Height, LocalNetworkStateData, NetworkId, Proof,
};
use arc_swap::ArcSwap;
use futures_util::{future::BoxFuture, poll, Stream};
use mocks::{MockCertifier, MockEpochPacker};
use rstest::fixture;
use tokio::sync::{broadcast, mpsc};
use tokio_stream::{wrappers::BroadcastStream, StreamExt as _};
use tokio_util::sync::CancellationToken;

use crate::{
    CertificateInput, CertificateOrchestrator, Certifier, CertifierOutput, CertifierResult,
    EpochPacker, Error,
};

mod certifier_results;
mod receive_certificates;

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
}

impl PerEpochReader for DummyPendingStore {
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
    ) -> std::result::Result<(EpochNumber, CertificateIndex), agglayer_storage::error::Error> {
        Ok((0, 0))
    }

    fn start_packing(&self) -> Result<(), agglayer_storage::error::Error> {
        todo!()
    }
}

impl StateReader for DummyPendingStore {
    fn get_active_networks(&self) -> Result<Vec<NetworkId>, agglayer_storage::error::Error> {
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
    ) -> Result<Vec<(NetworkId, Height, CertificateId, EpochNumber)>, agglayer_storage::error::Error>
    {
        self.settled
            .read()
            .unwrap()
            .iter()
            .map(|(network_id, (height, id))| Ok((*network_id, *height, *id, 0)))
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
        epoch_number: u64,
        start_checkpoint: BTreeMap<NetworkId, Height>,
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
}

impl StateWriter for DummyPendingStore {
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
                new_local_exit_root: certificate.new_local_exit_root.into(),
                status,
                metadata: certificate.metadata,
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
        _certificate_id: &CertificateId,
        _epoch_number: &EpochNumber,
    ) -> Result<(), agglayer_storage::error::Error> {
        Ok(())
    }
}

impl PendingCertificateReader for DummyPendingStore {
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
    let (_clock_sender, receiver) = broadcast::channel(1);
    let clock = BroadcastStream::new(receiver).filter_map(|value| value.ok());

    let (_data_sender, data_receiver) = mpsc::channel(10);
    let cancellation_token = CancellationToken::new();
    let store = Arc::new(DummyPendingStore::default());

    let (check_sender, mut check_receiver) = mpsc::channel(1);
    let check = Check::builder()
        .pending_store(store.clone())
        .executed(check_sender)
        .build();

    let epochs_store = store.clone();
    let current_epoch = ArcSwap::new(store.clone());

    let mut orchestrator = CertificateOrchestrator::try_new(
        clock,
        data_receiver,
        cancellation_token.clone(),
        check.clone(),
        check.clone(),
        store.clone(),
        epochs_store,
        current_epoch,
        store.clone(),
    )
    .expect("Unable to create orchestrator");

    cancellation_token.cancel();

    assert!(matches!(poll!(&mut orchestrator), Poll::Ready(())));

    assert!(orchestrator.proving_cursors.is_empty());
    assert!(check_receiver.try_recv().is_err());
}

// Can collect certificates and pack them at the end of an epoch
#[tokio::test]
async fn test_collect_certificates() {
    let (clock_sender, receiver) = broadcast::channel(1);
    let clock = BroadcastStream::new(receiver).filter_map(|value| value.ok());
    let (data_sender, data_receiver) = mpsc::channel(10);
    let cancellation_token = CancellationToken::new();

    let store = DummyPendingStore {
        current_epoch: 1,
        ..Default::default()
    };

    let store = Arc::new(store);
    let (check_sender, mut check_receiver) = mpsc::channel(1);
    let check = Check::builder()
        .pending_store(store.clone())
        .executed(check_sender)
        .expected_epoch(1)
        .build();

    let epochs_store = store.clone();

    let current_epoch = ArcSwap::new(store.clone());
    let mut orchestrator = CertificateOrchestrator::try_new(
        clock,
        data_receiver,
        cancellation_token,
        check.clone(),
        check.clone(),
        store.clone(),
        epochs_store,
        current_epoch,
        store.clone(),
    )
    .expect("Unable to create orchestrator");

    _ = data_sender.send((1.into(), 1, [0; 32].into())).await;
    _ = clock_sender.send(agglayer_clock::Event::EpochEnded(1));

    let _poll = poll!(&mut orchestrator);

    assert!(orchestrator.proving_cursors.is_empty());
    assert!(check_receiver.recv().await.is_some());
}

// A certificate received after an EpochEnded is stored for next epoch
#[tokio::test]
#[ignore]
async fn test_collect_certificates_after_epoch() {
    let (clock_sender, receiver) = broadcast::channel(1);
    let clock = BroadcastStream::new(receiver).filter_map(|value| value.ok());
    let (data_sender, data_receiver) = mpsc::channel(10);
    let cancellation_token = CancellationToken::new();

    let store = Arc::new(DummyPendingStore::default());
    let (check_sender, mut check_receiver) = mpsc::channel(1);
    let check = Check::builder()
        .pending_store(store.clone())
        .executed(check_sender)
        .expected_epoch(1)
        .build();

    let epochs_store = store.clone();
    let current_epoch = ArcSwap::new(store.clone());
    let mut orchestrator = CertificateOrchestrator::try_new(
        clock,
        data_receiver,
        cancellation_token,
        check.clone(),
        check.clone(),
        store.clone(),
        epochs_store,
        current_epoch,
        store.clone(),
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
    let (clock_sender, receiver) = broadcast::channel(1);
    let clock = BroadcastStream::new(receiver).filter_map(|value| value.ok());
    let (_data_sender, data_receiver) = mpsc::channel(10);
    let cancellation_token = CancellationToken::new();

    let store = Arc::new(DummyPendingStore::default());
    let (check_sender, mut check_receiver) = mpsc::channel(1);
    let check = Check::builder()
        .pending_store(store.clone())
        .executed(check_sender)
        .expected_epoch(1)
        .build();

    let epochs_store = store.clone();
    let current_epoch = ArcSwap::new(store.clone());
    let mut orchestrator = CertificateOrchestrator::try_new(
        clock,
        data_receiver,
        cancellation_token,
        check.clone(),
        check.clone(),
        store.clone(),
        epochs_store,
        current_epoch,
        store.clone(),
    )
    .expect("Unable to create orchestrator");

    _ = clock_sender.send(agglayer_clock::Event::EpochEnded(1));
    let _poll = poll!(&mut orchestrator);

    assert!(check_receiver.recv().await.is_some());
}

#[fixture]
fn clock() -> (
    broadcast::Sender<agglayer_clock::Event>,
    impl Stream<Item = agglayer_clock::Event>,
) {
    let (clock_sender, receiver) = broadcast::channel(1);
    (
        clock_sender,
        BroadcastStream::new(receiver).filter_map(|v| v.ok()),
    )
}

#[fixture]
fn check() -> (
    Arc<DummyPendingStore>,
    mpsc::Receiver<CertifierOutput>,
    Check,
) {
    let (check_sender, check_receiver) = mpsc::channel(1);
    let store = Arc::new(DummyPendingStore::default());
    let check = Check::builder()
        .pending_store(store.clone())
        .executed(check_sender)
        .expected_epoch(1)
        .build();

    (store, check_receiver, check)
}

type TestOrchestrator<S> = CertificateOrchestrator<
    S,
    Check,
    Check,
    DummyPendingStore,
    DummyPendingStore,
    DummyPendingStore,
    DummyPendingStore,
>;

type IMockOrchestrator<S> = CertificateOrchestrator<
    S,
    MockEpochPacker,
    MockCertifier,
    MockPendingStore,
    MockEpochsStore,
    MockPerEpochStore,
    MockStateStore,
>;

mod mocks;

#[derive(Default, buildstructor::Builder)]
struct MockOrchestrator {
    certifier: Option<MockCertifier>,
    epoch_packer: Option<MockEpochPacker>,
    pending_store: Option<MockPendingStore>,
    epochs_store: Option<MockEpochsStore>,
    state_store: Option<MockStateStore>,
    current_epoch: Option<MockPerEpochStore>,
}

#[fixture]
pub(crate) fn create_orchestrator_mock(
    #[default(MockOrchestrator::default())] builder: MockOrchestrator,
    clock: (
        broadcast::Sender<agglayer_clock::Event>,
        impl Stream<Item = agglayer_clock::Event>,
    ),
) -> (
    mpsc::Sender<(NetworkId, Height, CertificateId)>,
    IMockOrchestrator<impl Stream<Item = agglayer_clock::Event>>,
) {
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
        data_sender,
        CertificateOrchestrator::try_new(
            clock.1,
            data_receiver,
            cancellation_token,
            builder.epoch_packer.unwrap_or_default(),
            builder.certifier.unwrap_or_default(),
            pending_store,
            epochs_store,
            current_epoch,
            state_store,
        )
        .expect("Unable to create orchestrator"),
    )
}

type OrchestratorResult<T> = (
    mpsc::Sender<(NetworkId, Height, CertificateId)>,
    mpsc::Receiver<CertifierOutput>,
    TestOrchestrator<T>,
);
#[fixture]
pub(crate) fn create_orchestrator(
    check: (
        Arc<DummyPendingStore>,
        mpsc::Receiver<CertifierOutput>,
        Check,
    ),
    clock: (
        broadcast::Sender<agglayer_clock::Event>,
        impl Stream<Item = agglayer_clock::Event>,
    ),
) -> OrchestratorResult<impl Stream<Item = agglayer_clock::Event>> {
    let (data_sender, data_receiver) = mpsc::channel(10);
    let cancellation_token = CancellationToken::new();
    let store = check.0;
    let epochs_store = store.clone();
    let current_epoch = ArcSwap::new(store.clone());

    (
        data_sender,
        check.1,
        CertificateOrchestrator::try_new(
            clock.1,
            data_receiver,
            cancellation_token,
            check.2.clone(),
            check.2.clone(),
            store.clone(),
            epochs_store,
            current_epoch,
            store.clone(),
        )
        .expect("Unable to create orchestrator"),
    )
}

#[derive(Clone)]
pub(crate) struct Check {
    pending_store: Arc<DummyPendingStore>,
    #[allow(unused)]
    state_store: Arc<DummyPendingStore>,
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
        pending_store: Arc<DummyPendingStore>,
        executed: mpsc::Sender<CertifierOutput>,
        expected_epoch: Option<u64>,
    ) -> Self {
        Self {
            state_store: pending_store.clone(),
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

impl EpochPacker for Check {
    type PerEpochStore = DummyPendingStore;
    fn settle_certificate(
        &self,
        _epoch: Arc<Self::PerEpochStore>,
        _certificate_index: CertificateIndex,
        _certificate_id: CertificateId,
    ) -> Result<BoxFuture<Result<(), Error>>, Error> {
        Ok(Box::pin(async { Ok(()) }))
    }

    fn pack(&self, epoch: Arc<Self::PerEpochStore>) -> Result<BoxFuture<Result<(), Error>>, Error> {
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

        Ok(Box::pin(async { Ok(()) }))
    }
}

impl CertificateInput for () {
    fn network_id(&self) -> NetworkId {
        NetworkId::new(0)
    }
}

impl Certifier for Check {
    fn certify(
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
            .ok_or(Error::CertificateNotFound(network_id, height))
            .expect("Certificate not found");

        let certificate_id = certificate.hash();

        let proof = Proof::new_for_test();
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
        Ok(Box::pin(async move { Ok(result) }))
    }
}
