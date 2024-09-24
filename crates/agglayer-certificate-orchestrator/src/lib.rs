use std::{
    collections::{btree_map::Entry, BTreeMap},
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use agglayer_clock::Event;
use agglayer_storage::stores::{
    EpochStoreWriter, PendingCertificateReader, PendingCertificateWriter, PerEpochWriter,
    StateReader, StateWriter,
};
use agglayer_types::{Certificate, CertificateId, Height, LocalNetworkStateData, NetworkId, Proof};
use arc_swap::ArcSwap;
use futures_util::{future::BoxFuture, Stream, StreamExt};
use pessimistic_proof::ProofError;
use tokio::{
    sync::mpsc::Receiver,
    task::{JoinHandle, JoinSet},
};
use tokio_util::sync::{CancellationToken, WaitForCancellationFutureOwned};
use tracing::{debug, error, warn};

#[cfg(test)]
mod tests;

const MAX_POLL_READS: usize = 1_000;

/// Global State composed of each network state for all networks.
/// Eventually, each state will live only in the networks themselves.
type GlobalState = BTreeMap<NetworkId, LocalNetworkStateData>;

/// The Certificate orchestrator receives the certificates from CDKs.
///
/// Each certificate reception triggers the generation of a pessimistic proof.
/// At the end of the epoch, the Certificate Orchestrator collects a set of
/// pessimistic proofs generated so far and settles them on the L1.
pub struct CertificateOrchestrator<C, E, A, PendingStore, EpochsStore, PerEpochStore, StateStore> {
    /// Epoch packing task resolver.
    epoch_packing_tasks: JoinSet<Result<(), Error>>,
    /// Epoch packing task builder.
    #[allow(unused)]
    epoch_packing_task_builder: Arc<E>,
    /// Certifier task resolver.
    certifier_tasks: JoinSet<Result<CertifierOutput, Error>>,
    /// Certifier task builder.
    certifier_task_builder: Arc<A>,
    /// Global network state.
    global_state: GlobalState,
    /// Clock stream to receive EpochEnded events.
    clock: C,
    /// Receiver for certificates coming from CDKs.
    data_receiver: Receiver<(NetworkId, Height, CertificateId)>,
    /// Cancellation token for graceful shutdown.
    cancellation_token: Pin<Box<WaitForCancellationFutureOwned>>,

    cursors: BTreeMap<NetworkId, Height>,
    state_store: Arc<StateStore>,
    pending_store: Arc<PendingStore>,
    #[allow(unused)]
    epochs_store: Arc<EpochsStore>,
    #[allow(unused)]
    current_epoch: Arc<ArcSwap<PerEpochStore>>,
}

impl<C, E, A, PendingStore, EpochsStore, PerEpochStore, StateStore>
    CertificateOrchestrator<C, E, A, PendingStore, EpochsStore, PerEpochStore, StateStore>
where
    StateStore: StateReader,
{
    /// Creates a new CertificateOrchestrator instance.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn try_new(
        clock: C,
        data_receiver: Receiver<(NetworkId, Height, CertificateId)>,
        cancellation_token: CancellationToken,
        epoch_packing_task_builder: E,
        certifier_task_builder: A,
        pending_store: Arc<PendingStore>,
        epochs_store: Arc<EpochsStore>,
        current_epoch: Arc<ArcSwap<PerEpochStore>>,
        state_store: Arc<StateStore>,
    ) -> Result<Self, Error> {
        let cursors = state_store
            .get_current_settled_height()?
            .iter()
            .map(|(network_id, height, _)| (*network_id, *height))
            .collect();

        Ok(Self {
            epoch_packing_tasks: JoinSet::new(),
            certifier_tasks: JoinSet::new(),
            clock,
            epoch_packing_task_builder: Arc::new(epoch_packing_task_builder),
            certifier_task_builder: Arc::new(certifier_task_builder),
            global_state: Default::default(),
            data_receiver,
            cancellation_token: Box::pin(cancellation_token.cancelled_owned()),
            cursors,
            pending_store,
            epochs_store,
            current_epoch,
            state_store,
        })
    }
}

#[buildstructor::buildstructor]
impl<C, E, A, PendingStore, EpochsStore, PerEpochStore, StateStore>
    CertificateOrchestrator<C, E, A, PendingStore, EpochsStore, PerEpochStore, StateStore>
where
    C: Stream<Item = Event> + Unpin + Send + 'static,
    A: Certifier,
    E: EpochPacker<Item = (Certificate, Proof)>,
    PendingStore: PendingCertificateReader + PendingCertificateWriter + 'static,
    EpochsStore: EpochStoreWriter<PerEpochStore = PerEpochStore> + 'static,
    PerEpochStore: PerEpochWriter + 'static,
    StateStore: StateReader + StateWriter + 'static,
{
    /// Function that setups and starts the CertificateOrchestrator.
    ///
    /// The available methods are:
    ///
    /// - `builder`: Creates a new builder instance.
    /// - `clock`: Sets clock stream to receive EpochEnded events.
    /// - `data_receiver`: Sets the receiver for certificates coming from CDKs.
    /// - `cancellation_token`: Sets the cancellation token for graceful
    ///   shutdown.
    /// - `epoch_packing_builder`: Sets the task builder for epoch packing.
    /// - `start`: Starts the CertificateOrchestrator.
    ///
    /// # Examples
    /// ```
    /// # use agglayer_certificate_orchestrator::Error;
    /// # use agglayer_certificate_orchestrator::EpochPacker;
    /// # use agglayer_certificate_orchestrator::Certifier;
    /// # use agglayer_certificate_orchestrator::CertifierResult;
    /// # use agglayer_certificate_orchestrator::CertifierOutput;
    /// # use agglayer_certificate_orchestrator::CertificateInput;
    /// # use agglayer_certificate_orchestrator::CertificateOrchestrator;
    /// # use tokio_stream::wrappers::BroadcastStream;
    /// # use tokio_util::sync::CancellationToken;
    /// # use futures_util::future::BoxFuture;
    /// # use tokio_stream::StreamExt;
    /// # use pessimistic_proof::bridge_exit::NetworkId;
    /// # use agglayer_types::Certificate;
    /// # use agglayer_types::Proof;
    /// # use agglayer_types::Height;
    /// # use std::sync::Arc;
    /// # use agglayer_config::Config;
    /// # use agglayer_storage::tests::TempDBDir;
    /// # use agglayer_storage::storage::DB;
    /// # use agglayer_storage::storage::state_db_cf_definitions;
    /// # use agglayer_storage::stores::pending::PendingStore;
    /// # use agglayer_storage::stores::epochs::EpochsStore;
    /// # use agglayer_storage::stores::state::StateStore;
    /// # use agglayer_storage::stores::EpochStoreReader;
    /// # use agglayer_types::LocalNetworkStateData;
    ///
    /// # #[derive(Clone)]
    /// # pub struct Empty;
    /// # impl CertificateInput for Empty {
    /// #     fn network_id(&self) -> NetworkId {
    /// #         NetworkId::new(0)
    /// #     }
    /// # }
    ///
    /// # #[derive(Clone)]
    /// # pub struct AggregatorNotifier {}
    ///
    /// # impl AggregatorNotifier {
    /// #     pub(crate) fn new() -> Self {
    /// #         Self {}
    /// #     }
    /// # }
    ///
    /// impl EpochPacker for AggregatorNotifier {
    ///     type Item = (Certificate, Proof);
    ///     fn pack<T: IntoIterator<Item = (Certificate, Proof)>>(
    ///         &self,
    ///         epoch: u64,
    ///         to_pack: T,
    ///     ) -> Result<BoxFuture<Result<(), Error>>, Error> {
    ///         Ok(Box::pin(async move { Ok(()) }))
    ///     }
    /// }
    ///
    /// impl Certifier for AggregatorNotifier {
    ///     fn certify(
    ///         &self,
    ///         local_state: LocalNetworkStateData,
    ///         network_id: NetworkId,
    ///         height: Height,
    ///     ) -> CertifierResult {
    ///         Ok(Box::pin(async move {
    ///             Ok(CertifierOutput {
    ///                 new_state: LocalNetworkStateData::default().into(),
    ///                 network: NetworkId::new(0),
    ///                 certificate: Certificate::new_for_test(network_id,
    /// height),                 height: 0,
    ///             })
    ///         }))
    ///     }
    /// }
    ///
    /// async fn start() -> anyhow::Result<()> {
    ///     let (sender, receiver) = tokio::sync::broadcast::channel(1);
    ///     let clock_stream =
    /// BroadcastStream::new(sender.subscribe()).filter_map(|value| value.ok());
    ///     let notifier = AggregatorNotifier::new();
    ///     let data_receiver = tokio::sync::mpsc::channel(1).1;
    ///
    ///     let config = Arc::new(Config::new_for_test());
    ///     let tmp = TempDBDir::new();
    ///     let db = Arc::new(DB::open_cf(tmp.path.as_path(),
    /// state_db_cf_definitions()).unwrap());
    ///
    ///     let metadata_db = Arc::new(DB::open_cf(
    ///         &config.storage.metadata_db_path,
    ///         agglayer_storage::storage::metadata_db_cf_definitions(),
    ///     )?);
    ///     let pending_db = Arc::new(DB::open_cf(
    ///         &config.storage.pending_db_path,
    ///         agglayer_storage::storage::pending_db_cf_definitions(),
    ///     )?);
    ///     let state_db = Arc::new(DB::open_cf(
    ///         &config.storage.state_db_path,
    ///         agglayer_storage::storage::state_db_cf_definitions(),
    ///     )?);
    ///
    ///     let epochs_store = Arc::new(EpochsStore::new(config, 0,
    /// pending_db.clone())?);
    ///
    ///     let state_store = Arc::new(StateStore::new(state_db.clone()));
    ///     let pending_store = Arc::new(PendingStore::new(pending_db.clone()));

    ///     CertificateOrchestrator::builder()
    ///         .clock(clock_stream)
    ///         .data_receiver(data_receiver)
    ///         .cancellation_token(CancellationToken::new())
    ///         .epoch_packing_task_builder(notifier.clone())
    ///         .certifier_task_builder(notifier)
    ///         .pending_store(pending_store)
    ///         .current_epoch(epochs_store.get_current_epoch())
    ///         .epochs_store(epochs_store)
    ///         .state_store(state_store)
    ///         .start()
    ///         .await
    ///         .unwrap();
    ///
    ///     Ok(())
    /// }
    /// ```
    /// 
    /// # Errors
    ///
    /// This function can't fail but returns a Result for convenience and future
    ///
    /// evolution.
    #[builder(entry = "builder", exit = "start", visibility = "pub")]
    pub async fn start(
        clock: C,
        data_receiver: Receiver<(NetworkId, Height, CertificateId)>,
        cancellation_token: CancellationToken,
        epoch_packing_task_builder: E,
        certifier_task_builder: A,
        pending_store: Arc<PendingStore>,
        epochs_store: Arc<EpochsStore>,
        current_epoch: Arc<ArcSwap<PerEpochStore>>,
        state_store: Arc<StateStore>,
    ) -> anyhow::Result<JoinHandle<()>> {
        let mut orchestrator = Self::try_new(
            clock,
            data_receiver,
            cancellation_token,
            epoch_packing_task_builder,
            certifier_task_builder,
            pending_store,
            epochs_store,
            current_epoch,
            state_store,
        )?;

        let check_cursor = orchestrator
            .cursors
            .iter()
            .map(|(network_id, height)| (*network_id, height + 1))
            .collect::<Vec<_>>();

        // Try to spawn the certifier tasks for the next height of each network
        for (network_id, height) in check_cursor {
            let local_state = orchestrator
                .global_state
                .entry(network_id)
                .or_default()
                .clone();

            let task = orchestrator.certifier_task_builder.clone();
            orchestrator
                .certifier_tasks
                .spawn(async move { task.certify(local_state, network_id, height)?.await });
        }

        let handle = tokio::spawn(orchestrator);

        Ok(handle)
    }
}

impl<C, E, A, PendingStore, EpochsStore, PerEpochStore, StateStore>
    CertificateOrchestrator<C, E, A, PendingStore, EpochsStore, PerEpochStore, StateStore>
where
    A: Certifier,
    E: EpochPacker<Item = (Certificate, Proof)>,
    PendingStore: PendingCertificateReader + PendingCertificateWriter,
    EpochsStore: Send + Sync + 'static,
    StateStore: StateReader + StateWriter + 'static,
    PerEpochStore: PerEpochWriter + 'static,
{
    fn handle_epoch_end(&mut self, _epoch: u64) -> Result<(), Error> {
        Ok(())
    }

    fn receive_certificates(
        &mut self,
        cursors: &[(NetworkId, Height, CertificateId)],
    ) -> Result<(), Error> {
        for (network_id, height, _) in cursors {
            let entry = self.cursors.entry(*network_id);
            match entry {
                Entry::Vacant(entry) if *height == 0 => {
                    entry.insert(*height);
                    self.spawn_certifier_task(*network_id, *height);
                }
                Entry::Occupied(mut entry) => {
                    let cursor_height = entry.get_mut();

                    if *cursor_height == height - 1 {
                        *cursor_height = *height;

                        // TODO: Check already present in `CertificateHeader`
                        self.spawn_certifier_task(*network_id, *height);
                    } else {
                        warn!(
                            "Received a certificate with an unexpected height: {} for network {}",
                            height, network_id
                        );
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn spawn_certifier_task(&mut self, network: NetworkId, height: Height) {
        let local_state = self.global_state.entry(network).or_default().clone();
        let task = self.certifier_task_builder.clone();
        self.certifier_tasks
            .spawn(async move { task.certify(local_state, network, height)?.await });
    }

    fn handle_certifier_result(
        &mut self,
        certifier_result: Result<CertifierOutput, Error>,
    ) -> Result<(), ()> {
        match certifier_result {
            Ok(CertifierOutput {
                certificate,
                height,
                network,
                ..
            }) => {
                let entry = self.cursors.entry(network);

                match entry {
                    Entry::Vacant(entry) if certificate.height == 0 => {
                        entry.insert(0);
                    }
                    Entry::Vacant(_) => {
                        warn!(
                            "Received a proof generated for a certificate at height {} for a \
                             network that is not being tracked: {}",
                            height, network
                        );

                        return Err(());
                    }
                    Entry::Occupied(entry) if *entry.get() + 1 == certificate.height => todo!(),
                    Entry::Occupied(_) => todo!(),
                }
                // TODO: Deal with errors
                self.state_store
                    .insert_certificate_header(&certificate)
                    .expect("Failed to insert certificate header");

                self.pending_store
                    .remove_pending_certificate(network, height)
                    .expect("Failed to remove certificate");

                self.spawn_certifier_task(network, height + 1);

                return Ok(());
            }
            Err(Error::CertificateNotFound(network_id, height)) => {
                // TODO: Check if `CertificateHeader` if present, spawn next height
                warn!(
                    "Received a proof certification error for a certificate that is not found for \
                     network {} at {}",
                    network_id, height
                );
            }
            Err(Error::ProofAlreadyExists(network_id, height)) => {
                warn!(
                    "Received a proof certification error for a proof that already exists for \
                     network {} at height {}",
                    network_id, height
                );

                let entry = self.cursors.entry(network_id);

                match entry {
                    Entry::Vacant(_) if height != 0 => {
                        warn!(
                            "Received a proof generated for a certificate at height {} for a \
                             network that is not being tracked: {}",
                            height, network_id
                        );

                        // TODO: remove the proof?
                        return Err(());
                    }
                    Entry::Vacant(entry) => {
                        // Context:
                        // | pending certificate  | CertificateHeader | action              |
                        // |----------------------|-------------------|---------------------|
                        // | Found                | Not found         | Create the header   |
                        // | Found                | Found             | Delete the pending  |
                        //
                        // ProofAlreadyExists is an error that can happen only if we have a pending
                        // certificate.

                        // TODO: Handle errors
                        match self
                            .state_store
                            .get_certificate_header_by_cursor(network_id, height)
                            .expect("Failed to get certificate header by cursor")
                        {
                            Some(_header) => {
                                self.pending_store
                                    .remove_pending_certificate(network_id, height)
                                    .expect("Failed to remove certificate");
                            }
                            None => {
                                if let Some(certificate) = self
                                    .pending_store
                                    .get_certificate(network_id, height)
                                    .expect("Failed to get certificate")
                                {
                                    self.state_store
                                        .insert_certificate_header(&certificate)
                                        .expect("Failed to insert certificate header");
                                } else {
                                    // This should not happen as ProofAlreadyExists should only
                                    // happen if we have a pending certificate.
                                    warn!(
                                        "Failed to find the pending certificate for already \
                                         proved proof for network {} at height {}",
                                        network_id, height
                                    );

                                    return Err(());
                                }
                            }
                        }
                        entry.insert(height);
                        self.spawn_certifier_task(network_id, height + 1);
                    }
                    Entry::Occupied(entry) => {
                        let cursor_height = entry.into_mut();

                        // Context:
                        // | cursor | received_height  | action                      |
                        // |--------|------------------|-----------------------------|
                        // | N      | N                | Spawn N+1                   |
                        // | N      | N+1              | Update cursor and spawn N+1 |
                        // | N      | N-1              | Do nothing                  |
                        // | N      | N+X              | Remove the proof            |
                        if *cursor_height == height || *cursor_height == height - 1 {
                            if *cursor_height < height {
                                *cursor_height = height;
                            }
                            // Spawn the next certificate
                            self.spawn_certifier_task(network_id, height + 1);
                        } else {
                            warn!(
                                "Received a proof for a certificate with an unexpected height: {} \
                                 for network {}",
                                height, network_id
                            );

                            // TODO: remove the proof?
                            return Err(());
                        }

                        // Context:
                        // | pending certificate  | CertificateHeader | action              |
                        // |----------------------|-------------------|---------------------|
                        // | Found                | Not found         | Create the header   |
                        // | Found                | Found             | Delete the pending  |
                        //
                        // ProofAlreadyExists is an error that can happen only if we have a pending
                        // certificate.

                        // TODO: Handle errors
                        match self
                            .state_store
                            .get_certificate_header_by_cursor(network_id, height)
                            .expect("Failed to get certificate header by cursor")
                        {
                            Some(_header) => {
                                self.pending_store
                                    .remove_pending_certificate(network_id, height)
                                    .expect("Failed to remove certificate");
                            }
                            None => {
                                if let Some(certificate) = self
                                    .pending_store
                                    .get_certificate(network_id, height)
                                    .expect("Failed to get certificate")
                                {
                                    self.state_store
                                        .insert_certificate_header(&certificate)
                                        .expect("Failed to insert certificate header");
                                } else {
                                    // This should not happen as ProofAlreadyExists should only
                                    // happen if we have a pending certificate.
                                    warn!(
                                        "Failed to find the pending certificate for already \
                                         proved proof for network {} at height {}",
                                        network_id, height
                                    );

                                    return Err(());
                                }
                            }
                        }

                        // TODO: How to handle the fact that we do not have a state to update?
                        //  if let Some(_) =
                        //      self.global_state.insert(network_id, new_state) {
                        //          warn!(
                        //              "A proof has been generated for a
                        //              certificate at height 0 but the         // \
                        //              global state was already initialized"
                        //          );
                        // }

                        return Ok(());
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}

impl<C, E, A, PendingStore, EpochsStore, PerEpochStore, StateStore> Future
    for CertificateOrchestrator<C, E, A, PendingStore, EpochsStore, PerEpochStore, StateStore>
where
    C: Stream<Item = Event> + Send + Unpin + 'static,
    A: Certifier,
    E: EpochPacker<Item = (Certificate, Proof)>,
    PendingStore: PendingCertificateReader + PendingCertificateWriter,
    EpochsStore: Send + Sync + 'static,
    StateStore: StateReader + StateWriter + 'static,
    PerEpochStore: PerEpochWriter + 'static,
{
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Check if the orchestrator has been cancelled and should shutdown.
        if self.cancellation_token.as_mut().poll(cx).is_ready() {
            debug!("Certificate orchestrator cancelled by token");

            return Poll::Ready(());
        }

        // Poll the notification tasks to check for
        match self.certifier_tasks.poll_join_next(cx) {
            Poll::Ready(Some(Ok(proof_result))) => {
                _ = self.handle_certifier_result(proof_result);
            }

            Poll::Ready(Some(Err(error))) => {
                debug!("Critical error during p-proof generation: {:?}", error);
            }
            Poll::Ready(None) => {}
            Poll::Pending => {}
        }

        // Poll the notification tasks to check if any have errored.
        match self.epoch_packing_tasks.poll_join_next(cx) {
            Poll::Ready(Some(Ok(Err(error)))) => {
                error!("Error during epoch packing: {:?}", error)
            }
            Poll::Ready(Some(Err(error))) => {
                error!("Critical error during epoch packing: {:?}", error);
            }
            Poll::Ready(Some(Ok(Ok(())))) => {
                debug!("Successfully settled the epoch");
                // for (network_id, height) in self.cursors.iter_mut() {
                //     if *status == SettlementStatus::Settling {
                //         *status = SettlementStatus::Settled;
                //         debug!(
                //             "Settled the proofs for network {} at height {}",
                //             network_id, height
                //         );
                //     }
                // }
            }
            _ => {}
        }

        let mut received = vec![];
        if let Poll::Ready(1usize..) =
            self.data_receiver
                .poll_recv_many(cx, &mut received, MAX_POLL_READS)
        {
            if let Err(e) = self.receive_certificates(&received) {
                error!("Failed to handle a group of certificates: {e:?}");
            }

            return self.poll(cx);
        }

        if let Poll::Ready(Some(Event::EpochEnded(epoch))) = self.clock.poll_next_unpin(cx) {
            debug!("Epoch change event received: {}", epoch);

            if let Err(error) = self.handle_epoch_end(epoch) {
                error!("Failed to handle the EpochEnded event: {:?}", error);
            }
        }

        Poll::Pending
    }
}

/// Epoch Packer used to gather all the proofs generated on-the-go
/// and to submit them in a settlement tx to the L1.
pub trait EpochPacker: Unpin + Send + Sync + 'static {
    type Item;

    /// Pack a set of proofs for settlement on the L1
    fn pack<T: IntoIterator<Item = Self::Item>>(
        &self,
        epoch: u64,
        to_pack: T,
    ) -> Result<BoxFuture<Result<(), Error>>, Error>;
}

pub trait CertificateInput: Clone {
    fn network_id(&self) -> NetworkId;
}

impl CertificateInput for Certificate {
    fn network_id(&self) -> NetworkId {
        self.network_id
    }
}

#[derive(Clone)]
pub struct CertifierOutput {
    pub certificate: Certificate,
    pub height: Height,
    pub new_state: LocalNetworkStateData,
    pub network: NetworkId,
}

pub type CertifierResult<'a> = Result<BoxFuture<'a, Result<CertifierOutput, Error>>, Error>;

/// Apply one Certificate on top of a local state and computes one proof.
pub trait Certifier: Unpin + Send + Sync + 'static {
    fn certify(
        &self,
        full_state: LocalNetworkStateData,
        network_id: NetworkId,
        height: Height,
    ) -> CertifierResult;
}

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("certificate not found for network {0} at height {1}")]
    CertificateNotFound(NetworkId, Height),
    #[error("proof already exists for network {0} at height {1}")]
    ProofAlreadyExists(NetworkId, Height),
    #[error("proof verification failed")]
    ProofVerificationFailed,
    #[error("prover execution failed: {0}")]
    ProverExecutionFailed(#[from] anyhow::Error),
    #[error("native execution failed: {0:?}")]
    NativeExecutionFailed(#[from] ProofError),
    #[error("type error: {0}")]
    Types(#[from] agglayer_types::Error),
    #[error("Storage error: {0}")]
    Storage(#[from] agglayer_storage::error::Error),
    #[error("internal error")]
    InternalError,
}
