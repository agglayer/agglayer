use std::{
    collections::BTreeMap,
    option::Option,
    result::Result,
    sync::{Arc, RwLock},
    task::Poll,
};

use agglayer_storage::stores::{
    PendingCertificateReader, PendingCertificateWriter, PerEpochWriter, StateReader, StateWriter,
};
use agglayer_types::{Certificate, CertificateHeader, CertificateId, Height, NetworkId, Proof};
use arc_swap::ArcSwap;
use futures_util::{future::BoxFuture, poll, Stream};
use pessimistic_proof::{local_state::LocalNetworkStateData, LocalNetworkState};
use rstest::fixture;
use tokio::sync::{broadcast, mpsc};
use tokio_stream::{wrappers::BroadcastStream, StreamExt as _};
use tokio_util::sync::CancellationToken;

use crate::{
    CertificateInput, CertificateOrchestrator, Certifier, CertifierOutput, CertifierResult,
    EpochPacker, Error,
};

mod receive_certificates;

#[derive(Default)]
pub(crate) struct DummyPendingStore {
    pub(crate) pending_certificate: RwLock<BTreeMap<(NetworkId, Height), Certificate>>,
    pub(crate) proofs: RwLock<BTreeMap<CertificateId, Proof>>,
    pub(crate) settled: RwLock<BTreeMap<NetworkId, (Height, CertificateId)>>,
    pub(crate) certificate_per_network: RwLock<BTreeMap<(NetworkId, Height), CertificateHeader>>,
}

impl PerEpochWriter for DummyPendingStore {
    fn add_certificate(
        &self,
        _network_id: NetworkId,
        _height: Height,
    ) -> std::result::Result<(), agglayer_storage::error::Error> {
        Ok(())
    }
}

impl StateReader for DummyPendingStore {
    fn get_active_networks(&self) -> Result<Vec<NetworkId>, agglayer_storage::error::Error> {
        todo!()
    }

    fn get_current_settled_height(
        &self,
    ) -> Result<Vec<(NetworkId, Height, CertificateId)>, agglayer_storage::error::Error> {
        self.settled
            .read()
            .unwrap()
            .iter()
            .map(|(network_id, (height, id))| Ok((*network_id, *height, *id)))
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
            .cloned())
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
}

impl StateWriter for DummyPendingStore {
    fn insert_certificate_header(
        &self,
        certificate: &Certificate,
    ) -> Result<(), agglayer_storage::error::Error> {
        self.certificate_per_network.write().unwrap().insert(
            (certificate.network_id, certificate.height),
            CertificateHeader {
                certificate_id: certificate.hash(),
                network_id: certificate.network_id,
                height: certificate.height,
                epoch_number: None,
                certificate_index: None,
                new_local_exit_root: certificate.new_local_exit_root,
            },
        );
        Ok(())
    }
}

impl PendingCertificateReader for DummyPendingStore {
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
        Arc::new(current_epoch),
        store.clone(),
    )
    .expect("Unable to create orchestrator");

    cancellation_token.cancel();

    assert!(matches!(poll!(&mut orchestrator), Poll::Ready(())));

    assert!(orchestrator.cursors.is_empty());
    assert!(check_receiver.try_recv().is_err());
}

// Can collect certificates and pack them at the end of an epoch
#[tokio::test]
#[ignore]
async fn test_collect_certificates() {
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
        .expected_certificates_len(1)
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
        Arc::new(current_epoch),
        store.clone(),
    )
    .expect("Unable to create orchestrator");

    _ = data_sender.send((1.into(), 1, [0; 32])).await;
    _ = clock_sender.send(agglayer_clock::Event::EpochEnded(1));

    let _poll = poll!(&mut orchestrator);

    assert!(orchestrator.cursors.is_empty());
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
        .expected_certificates_len(0)
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
        Arc::new(current_epoch),
        store.clone(),
    )
    .expect("Unable to create orchestrator");

    _ = clock_sender.send(agglayer_clock::Event::EpochEnded(1));
    let _poll = poll!(&mut orchestrator);

    _ = data_sender.send((1.into(), 1, [0; 32])).await;

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
        .expected_certificates_len(0)
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
        Arc::new(current_epoch),
        store.clone(),
    )
    .expect("Unable to create orchestrator");

    _ = clock_sender.send(agglayer_clock::Event::EpochEnded(1));
    let _poll = poll!(&mut orchestrator);

    assert!(check_receiver.recv().await.is_some());
}
//
// #[rstest]
// fn get_all_proven_certificate() {
//     let (_clock_sender, receiver) =
// broadcast::channel::<agglayer_clock::Event>(1);     let clock =
// BroadcastStream::new(receiver).filter_map(|v| v.ok());     let (_, _, mut
// orchestrator) = create_orchestrator::default();     assert!(orchestrator.
// get_cursor_for_proven_certificate().is_empty());
//
//     orchestrator
//         .cursors
//         .insert(1, (10, SettlementStatus::Proved));
//     assert_eq!(orchestrator.get_cursor_for_proven_certificate().len(), 1);
//
//     orchestrator
//         .cursors
//         .insert(2, (10, SettlementStatus::Pending));
//     assert_eq!(orchestrator.get_cursor_for_proven_certificate().len(), 1);
// }
//
// #[tokio::test]
// async fn get_certs_and_proofs() {
//     let (_clock_sender, receiver) =
// broadcast::channel::<agglayer_clock::Event>(1);     let clock =
// BroadcastStream::new(receiver).filter_map(|v| v.ok());     let (_, _, mut
// orchestrator) = create_orchestrator::default();     assert!(orchestrator.
// get_cursor_for_proven_certificate().is_empty());
//
//     let certificate = Certificate {
//         network_id: 1,
//         height: 10,
//         prev_local_exit_root: [0; 32],
//         new_local_exit_root: [1; 32],
//         bridge_exits: vec![],
//         imported_bridge_exits: vec![],
//         signature: [3; 32],
//     };
//     let proof = Proof::new_for_test();
//
//     _ = orchestrator
//         .pending_store
//         .insert_pending_certificate(1, 10, &certificate);
//
//     _ = orchestrator
//         .pending_store
//         .insert_generated_proof(&certificate.hash(), &proof);
//
//     orchestrator
//         .cursors
//         .insert(1, (10, SettlementStatus::Proved));
//
//     let certs_and_proofs = orchestrator
//         .get_certificates_and_proofs(&orchestrator.
// get_cursor_for_proven_certificate())         .expect("Unable to get certs and
// proofs");
//
//     assert_eq!(certs_and_proofs.len(), 1);
//     assert!(
//         matches!(&certs_and_proofs[0], (n_id, h, Some((inner_certificate,
// _inner_proof))) if *inner_certificate == certificate && *n_id == 1 && *h ==
// 10) );
//
//     let unproven_certificate = Certificate {
//         network_id: 2,
//         height: 10,
//         prev_local_exit_root: [0; 32],
//         new_local_exit_root: [1; 32],
//         bridge_exits: vec![],
//         imported_bridge_exits: vec![],
//         signature: [3; 32],
//     };
//
//     _ = orchestrator
//         .pending_store
//         .insert_pending_certificate(2, 10, &unproven_certificate);
//
//     orchestrator
//         .cursors
//         .insert(2, (10, SettlementStatus::Proved));
//
//     let cursor = orchestrator.get_cursor_for_proven_certificate();
//     let certs_and_proofs = orchestrator
//         .get_certificates_and_proofs(&cursor)
//         .expect("Unable to get certs and proofs");
//
//     assert_eq!(certs_and_proofs.len(), 2);
//
//     assert!(
//         matches!(&certs_and_proofs[0], (n_id, h, Some((inner_certificate,
// _))) if *inner_certificate == certificate && *n_id == 1 && *h == 10)     );
//     assert!(certs_and_proofs[1].2.is_none());
//
//     orchestrator
//         .handle_epoch_end(1)
//         .expect("Epoch packing failed");
//
//     let cursor_network_1 = orchestrator.cursors.get(&1).unwrap();
//     assert_eq!(cursor_network_1.1, SettlementStatus::Settling);
// }

// #[tokio::test]
// async fn no_more_than_one() {
//     let (clock_sender, receiver) =
// broadcast::channel::<agglayer_clock::Event>(1);     let clock =
// BroadcastStream::new(receiver).filter_map(|v| v.ok());     let (data_sender,
// mut receiver, mut orchestrator) = create_orchestrator::default();     assert!
// (orchestrator.get_cursor_for_proven_certificate().is_empty());
//
//     let certificate = Certificate {
//         network_id: 1,
//         height: 10,
//         prev_local_exit_root: [0; 32],
//         new_local_exit_root: [1; 32],
//         bridge_exits: vec![],
//         imported_bridge_exits: vec![],
//         signature: [3; 32],
//     };
//
//     let certificate2 = Certificate {
//         network_id: 1,
//         height: 11,
//         prev_local_exit_root: [0; 32],
//         new_local_exit_root: [1; 32],
//         bridge_exits: vec![],
//         imported_bridge_exits: vec![],
//         signature: [3; 32],
//     };
//
//     let proof = Proof::new_for_test();
//
//     _ = orchestrator
//         .pending_store
//         .insert_pending_certificate(1, 10, &certificate);
//
//     _ = orchestrator
//         .pending_store
//         .insert_pending_certificate(1, 11, &certificate);
//
//     _ = orchestrator
//         .pending_store
//         .insert_generated_proof(&certificate.hash(), &proof);
//
//     _ = orchestrator
//         .pending_store
//         .insert_generated_proof(&certificate2.hash(), &proof);
//
//     orchestrator
//         .cursors
//         .insert(1, (9, SettlementStatus::Settled));
//
//     data_sender.send((1, 10, certificate.hash())).await;
//     data_sender.send((1, 11, certificate2.hash())).await;
//
//     let _poll = poll!(&mut orchestrator);
//
//     assert!(orchestrator.certifier_tasks.len() == 1);
//
//     receiver.recv().await;
//     let _poll = poll!(&mut orchestrator);
//
//     assert!(orchestrator.cursors.get(&1) == Some(&(10,
// SettlementStatus::Proved)));
//
//     clock_sender
//         .send(agglayer_clock::Event::EpochEnded(1))
//         .unwrap();
//
//     let _poll = poll!(&mut orchestrator);
//
//     assert!(orchestrator.cursors.get(&1) == Some(&(10,
// SettlementStatus::Settling)));
//
//     receiver.recv().await;
//     let _poll = poll!(&mut orchestrator);
//     assert!(orchestrator.cursors.get(&1) == Some(&(10,
// SettlementStatus::Settled))); }

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
) -> (
    mpsc::Sender<(NetworkId, Height, CertificateId)>,
    mpsc::Receiver<CertifierOutput>,
    TestOrchestrator<impl Stream<Item = agglayer_clock::Event>>,
) {
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
            Arc::new(current_epoch),
            store.clone(),
        )
        .expect("Unable to create orchestrator"),
    )
}

#[derive(Clone)]
pub(crate) struct Check {
    pending_store: Arc<DummyPendingStore>,
    state_store: Arc<DummyPendingStore>,
    expected_certificate: Option<Certificate>,
    #[allow(unused)]
    expected_proof: Option<Proof>,
    executed: mpsc::Sender<CertifierOutput>,
    expected_epoch: Option<u64>,
    expected_certificates_len: Option<usize>,
}

#[buildstructor::buildstructor]
impl Check {
    #[builder]
    pub(crate) fn new(
        pending_store: Arc<DummyPendingStore>,
        executed: mpsc::Sender<CertifierOutput>,
        expected_epoch: Option<u64>,
        expected_certificates_len: Option<usize>,
    ) -> Self {
        Self {
            state_store: pending_store.clone(),
            pending_store,
            executed,
            expected_certificate: None,
            expected_proof: None,
            expected_epoch,
            expected_certificates_len,
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
    type Item = (Certificate, Proof);
    fn pack<T>(&self, epoch: u64, to_pack: T) -> Result<BoxFuture<Result<(), Error>>, Error>
    where
        T: IntoIterator<Item = Self::Item>,
    {
        if let Some(expected_epoch) = self.expected_epoch {
            assert_eq!(epoch, expected_epoch);
        }
        if let Some(expected_certificates_len) = self.expected_certificates_len {
            assert!(to_pack.into_iter().count() == expected_certificates_len);
        }

        _ = self.executed.try_send(CertifierOutput {
            certificate: self
                .expected_certificate
                .clone()
                .unwrap_or_else(|| Certificate::new_for_test(1.into(), 0)),
            height: 0,
            new_state: LocalNetworkState::default(),
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

        self.state_store
            .insert_certificate_header(&certificate)
            .expect("Storage failure: Unable to insert certificate header");

        self.pending_store
            .remove_pending_certificate(network_id, height)
            .expect("Storage failure: Unable to remove certificate");

        let result = CertifierOutput {
            certificate,
            height,
            new_state: local_state.into(),
            network: network_id,
        };
        _ = self.executed.try_send(result.clone());
        Ok(Box::pin(async move { Ok(result) }))
    }
}
