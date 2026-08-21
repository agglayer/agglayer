use std::{
    collections::BTreeMap,
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use agglayer_clock::{ClockRef, Event};
use agglayer_settlement_service::SettlementServiceTrait;
use agglayer_storage::{
    columns::{
        latest_proven_certificate_per_network::ProvenCertificate,
        latest_settled_certificate_per_network::SettledCertificate,
    },
    stores::{
        async_api::AsyncPendingCertificateReaderExt, EpochStoreReader, EpochStoreWriter,
        PendingCertificateReader, PendingCertificateWriter, PerEpochReader, PerEpochWriter,
        StateReader, StateWriter,
    },
};
use agglayer_types::{CertificateId, EpochNumber, Height, NetworkId};
use arc_swap::ArcSwap;
use futures_util::{stream::FuturesUnordered, FutureExt, Stream, StreamExt};
use network_task::{NetworkTask, NewCertificate};
use tokio::{
    sync::mpsc::{self, Receiver},
    task::JoinHandle,
};
use tokio_util::sync::{CancellationToken, WaitForCancellationFutureOwned};
use tracing::{debug, error, warn};

mod certificate_task;
mod certifier;
mod error;
mod network_task;
#[cfg(test)]
mod tests;

pub use certifier::{CertificateInput, Certifier, CertifierOutput, CertifierResult};
pub use error::{CertificationError, Error, PreCertificationError};

const MAX_POLL_READS: usize = 1_000;

pub type EpochPackingTasks =
    FuturesUnordered<Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'static>>>;

pub type NetworkTasks = FuturesUnordered<
    Pin<Box<dyn Future<Output = Result<NetworkId, (NetworkId, Error)>> + Send + 'static>>,
>;

pub type SettlementContext = (NetworkId, CertificateId);

pub type SettlementTasks = FuturesUnordered<
    Pin<
        Box<
            dyn Future<
                    Output = (
                        SettlementContext,
                        Result<(NetworkId, SettledCertificate), Error>,
                    ),
                > + Send
                + 'static,
        >,
    >,
>;

/// The Certificate orchestrator receives the certificates from CDKs.
///
/// Each certificate reception triggers the generation of a pessimistic proof.
/// The Certificate Orchestrator collects the generated proofs and settles
/// them on the L1 on the go.
pub struct CertificateOrchestrator<
    CertifierClient,
    PendingStore,
    EpochsStore,
    PerEpochStore,
    StateStore,
    SettlementService,
> {
    /// The active epoch rollover, if packing or opening is still in progress.
    epoch_rollover: Option<JoinHandle<Result<PerEpochStore, Error>>>,
    /// Certifier task builder.
    certifier_task_builder: Arc<CertifierClient>,
    /// Clock stream to receive EpochEnded events.
    clock: Pin<Box<dyn Stream<Item = Event> + Send>>,
    clock_ref: ClockRef,
    /// Receiver for certificates coming from CDKs.
    data_receiver: Receiver<(NetworkId, Height, CertificateId)>,
    /// Cancellation token future for graceful shutdown.
    cancellation_token_future: Pin<Box<WaitForCancellationFutureOwned>>,

    /// Cancellation token for graceful shutdown.
    cancellation_token: CancellationToken,

    /// The state store to access data.
    state_store: Arc<StateStore>,
    /// Pending store to access the certificates and proofs.
    pending_store: Arc<PendingStore>,
    /// Epochs store to manage epoch transitions.
    epochs_store: Arc<EpochsStore>,
    /// The current epoch considered by the orchestrator.
    current_epoch: Arc<ArcSwap<PerEpochStore>>,

    /// Network tasks that are currently running, with their associated
    /// notifier.
    spawned_network_tasks: BTreeMap<NetworkId, mpsc::Sender<NewCertificate>>,

    /// Network task future resolver.
    network_tasks: NetworkTasks,

    /// Settlement service for submitting settlement jobs
    settlement_service: Arc<SettlementService>,
}

impl<CertifierClient, PendingStore, EpochsStore, PerEpochStore, StateStore, SettlementService>
    CertificateOrchestrator<
        CertifierClient,
        PendingStore,
        EpochsStore,
        PerEpochStore,
        StateStore,
        SettlementService,
    >
where
    PendingStore: PendingCertificateReader,
    SettlementService: SettlementServiceTrait,
{
    const DEFAULT_CERTIFICATION_NOTIFICATION_CHANNEL_SIZE: usize = 1000;

    /// Creates a new CertificateOrchestrator instance.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn try_new(
        clock: ClockRef,
        data_receiver: Receiver<(NetworkId, Height, CertificateId)>,
        cancellation_token: CancellationToken,
        certifier_task_builder: CertifierClient,
        pending_store: Arc<PendingStore>,
        epochs_store: Arc<EpochsStore>,
        current_epoch: Arc<ArcSwap<PerEpochStore>>,
        state_store: Arc<StateStore>,
        settlement_service: Arc<SettlementService>,
    ) -> Result<Self, Error> {
        Ok(Self {
            epoch_rollover: None,
            clock: Box::pin(tokio_stream::StreamExt::filter_map(
                tokio_stream::wrappers::BroadcastStream::new(clock.subscribe()?),
                |v| v.ok(),
            )),
            clock_ref: clock,
            certifier_task_builder: Arc::new(certifier_task_builder),
            data_receiver,
            cancellation_token: cancellation_token.clone(),
            cancellation_token_future: Box::pin(cancellation_token.cancelled_owned()),
            pending_store,
            epochs_store,
            current_epoch,
            state_store,
            spawned_network_tasks: Default::default(),
            network_tasks: FuturesUnordered::new(),
            settlement_service,
        })
    }
}

#[buildstructor::buildstructor]
impl<CertifierClient, PendingStore, EpochsStore, PerEpochStore, StateStore, SettlementService>
    CertificateOrchestrator<
        CertifierClient,
        PendingStore,
        EpochsStore,
        PerEpochStore,
        StateStore,
        SettlementService,
    >
where
    CertifierClient: Certifier,
    PendingStore: PendingCertificateReader + PendingCertificateWriter + 'static,
    EpochsStore: EpochStoreWriter<PerEpochStore = PerEpochStore> + EpochStoreReader + 'static,
    PerEpochStore: PerEpochWriter + PerEpochReader + 'static,
    StateStore: StateReader + StateWriter + 'static,
    SettlementService: SettlementServiceTrait + 'static,
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
    /// - `start`: Starts the CertificateOrchestrator.
    ///
    /// # Errors
    ///
    /// Returns an error when orchestrator setup fails, storage cannot be read,
    /// or a startup network task cannot be initialized.
    #[allow(clippy::too_many_arguments)]
    #[builder(entry = "builder", exit = "start", visibility = "pub")]
    pub async fn start(
        clock: ClockRef,
        data_receiver: Receiver<(NetworkId, Height, CertificateId)>,
        cancellation_token: CancellationToken,
        certifier_task_builder: CertifierClient,
        pending_store: Arc<PendingStore>,
        epochs_store: Arc<EpochsStore>,
        current_epoch: Arc<ArcSwap<PerEpochStore>>,
        state_store: Arc<StateStore>,
        settlement_service: Arc<SettlementService>,
    ) -> eyre::Result<JoinHandle<()>> {
        let mut orchestrator = Self::try_new(
            clock,
            data_receiver,
            cancellation_token,
            certifier_task_builder,
            pending_store.clone(),
            epochs_store,
            current_epoch,
            state_store,
            settlement_service,
        )?;

        // Try to spawn the certifier tasks for the next height of each network.
        let proven_certificates = pending_store.get_current_proven_height_async().await?;
        for ProvenCertificate(_, network_id, _height) in proven_certificates {
            orchestrator
                .spawn_initialized_network_task(network_id)
                .await?;
        }

        let handle = tokio::spawn(orchestrator);

        Ok(handle)
    }
}

impl<CertifierClient, PendingStore, EpochsStore, PerEpochStore, StateStore, SettlementService>
    CertificateOrchestrator<
        CertifierClient,
        PendingStore,
        EpochsStore,
        PerEpochStore,
        StateStore,
        SettlementService,
    >
where
    CertifierClient: Certifier,
    PendingStore: PendingCertificateReader + PendingCertificateWriter + 'static,
    EpochsStore: EpochStoreWriter<PerEpochStore = PerEpochStore> + EpochStoreReader + 'static,
    StateStore: StateReader + StateWriter + 'static,
    PerEpochStore: PerEpochWriter + PerEpochReader + 'static,
    SettlementService: SettlementServiceTrait + 'static,
{
    async fn spawn_initialized_network_task(&mut self, network_id: NetworkId) -> Result<(), Error> {
        if self.spawned_network_tasks.contains_key(&network_id) {
            debug!("Network task already spawned for network {}", network_id);

            return Ok(());
        }

        let (sender, receiver) =
            mpsc::channel(Self::DEFAULT_CERTIFICATION_NOTIFICATION_CHANNEL_SIZE);
        let task = NetworkTask::new(
            self.pending_store.clone(),
            self.state_store.clone(),
            self.certifier_task_builder.clone(),
            self.clock_ref.clone(),
            network_id,
            receiver,
            self.settlement_service.clone(),
            self.current_epoch.clone(),
        )
        .await?;
        let cancellation_token = self.cancellation_token.clone();
        self.network_tasks.push(
            async move {
                task.run(cancellation_token)
                    .await
                    .map_err(|error| (network_id, error))
            }
            .boxed(),
        );
        self.spawned_network_tasks.insert(network_id, sender);

        Ok(())
    }

    fn spawn_network_task(&mut self, network_id: NetworkId) {
        if self.spawned_network_tasks.contains_key(&network_id) {
            debug!("Network task already spawned for network {}", network_id);

            return;
        }

        let (sender, receiver) =
            mpsc::channel(Self::DEFAULT_CERTIFICATION_NOTIFICATION_CHANNEL_SIZE);
        let task = NetworkTask::new(
            self.pending_store.clone(),
            self.state_store.clone(),
            self.certifier_task_builder.clone(),
            self.clock_ref.clone(),
            network_id,
            receiver,
            self.settlement_service.clone(),
            self.current_epoch.clone(),
        );
        let cancellation_token = self.cancellation_token.clone();
        let task_future = async move {
            let task = task.await.map_err(|error| (network_id, error))?;

            task.run(cancellation_token)
                .await
                .map_err(|error| (network_id, error))
        }
        .boxed();
        self.network_tasks.push(task_future);

        self.spawned_network_tasks.insert(network_id, sender);
    }

    /// Function that receives the certificates cursor pushed by the RPC module.
    /// This function is responsible for:
    /// - Updating the cursors for the proofs that have been generated so far.
    /// - Spawning the certifier task for the next height of the network.
    fn receive_certificates(
        &mut self,
        cursors: impl IntoIterator<Item = (NetworkId, Height, CertificateId)>,
    ) {
        for (network_id, height, certificate_id) in cursors {
            self.spawn_network_task(network_id);

            if let Some(sender) = self.spawned_network_tasks.get(&network_id) {
                if let Ok(sender) = sender.try_reserve() {
                    sender.send(NewCertificate {
                        certificate_id,
                        height,
                    })
                } else {
                    error!(
                        "Failed to send the certificate {certificate_id} to the network task for \
                         network {network_id}",
                    );
                }
            } else {
                warn!("Unable to find the network task for network {}", network_id);
                continue;
            };
        }
    }

    /// Function that handles the end of an epoch.
    /// This function is called when the orchestrator receives an EpochEnded
    /// event. The function is responsible for:
    /// - Packing the closing epoch.
    /// - Opening the next epoch.
    fn start_epoch_rollover(&mut self, epoch: EpochNumber) {
        debug!("Start the settlement of the epoch {}", epoch);

        self.epoch_rollover = Some(self.spawn_epoch_rollover_task(epoch));
    }

    fn spawn_epoch_rollover_task(
        &self,
        epoch: EpochNumber,
    ) -> JoinHandle<Result<PerEpochStore, Error>> {
        let closing_epoch = self.current_epoch.load_full();
        let epochs_store = self.epochs_store.clone();
        let cancellation_token = self.cancellation_token.clone();
        tokio::task::spawn_blocking(move || {
            if let Err(error) = closing_epoch.start_packing() {
                error!("Failed to pack the epoch {}: {:?}", epoch, error);

                match error {
                    agglayer_storage::error::Error::AlreadyPacked(_) => {}
                    agglayer_storage::error::Error::DBError(error) => {
                        let msg =
                            format!("CRITICAL error during packing of epoch {epoch}: {error}",);
                        error!(msg);
                        cancellation_token.cancel();
                        return Err(Error::InternalError(msg));
                    }

                    // Other errors shouldn't happen
                    error => {
                        let msg =
                            format!("CRITICAL error: Failed to pack the epoch {epoch}: {error:?}");
                        error!(msg);
                        return Err(Error::InternalError(msg));
                    }
                }
            }

            // TODO: Check for overflow
            let next_epoch = epoch.next();

            epochs_store
                .open_with_start_checkpoint(next_epoch, closing_epoch.get_end_checkpoint())
                .map_err(|error| {
                    let msg = format!(
                        "CRITICAL error: Failed to open the next epoch {next_epoch}: {error:?}",
                    );
                    error!(msg);
                    Error::InternalError(msg)
                })
        })
    }
}

impl<A, PendingStore, EpochsStore, PerEpochStore, StateStore, SettlementService> Future
    for CertificateOrchestrator<
        A,
        PendingStore,
        EpochsStore,
        PerEpochStore,
        StateStore,
        SettlementService,
    >
where
    A: Certifier,
    PendingStore: PendingCertificateReader + PendingCertificateWriter + 'static,
    EpochsStore: EpochStoreWriter<PerEpochStore = PerEpochStore> + EpochStoreReader + 'static,
    StateStore: StateReader + StateWriter + 'static,
    PerEpochStore: PerEpochWriter + PerEpochReader + 'static,
    SettlementService: SettlementServiceTrait + 'static,
{
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Check if the orchestrator has been cancelled and should shutdown.
        if self.cancellation_token_future.as_mut().poll(cx).is_ready() {
            debug!("Certificate orchestrator cancelled by token");

            return Poll::Ready(());
        }

        // An epoch rollover is exclusive: finish and publish it before polling
        // network tasks, certificate input, or another clock event.
        if let Some(rollover) = self.epoch_rollover.as_mut() {
            match Pin::new(rollover).poll(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(result) => {
                    self.epoch_rollover = None;
                    match result.expect("epoch rollover task panicked") {
                        Ok(new_epoch) => {
                            debug!("Successfully rolled over the epoch");
                            self.current_epoch.store(Arc::new(new_epoch));
                        }
                        Err(error) => error!("Error during epoch rollover: {error:?}"),
                    }
                }
            }
        }

        // Poll the notification tasks to check for
        match self.network_tasks.poll_next_unpin(cx) {
            Poll::Ready(Some(Ok(network_id))) => {
                warn!("Network task for {} completed successfully", network_id);
                _ = self.spawned_network_tasks.remove(&network_id);
            }

            Poll::Ready(Some(Err((network_id, error)))) => {
                warn!("Network task for rollup {network_id} failed: {error:?}");
                _ = self.spawned_network_tasks.remove(&network_id);
            }
            Poll::Ready(None) => {}
            Poll::Pending => {}
        }

        let mut received = vec![];
        if let Poll::Ready(1usize..) =
            self.data_receiver
                .poll_recv_many(cx, &mut received, MAX_POLL_READS)
        {
            self.receive_certificates(received);

            return self.poll(cx);
        }

        if let Poll::Ready(Some(Event::EpochEnded(epoch))) = self.clock.poll_next_unpin(cx) {
            debug!("Epoch change event received: {}", epoch);

            self.start_epoch_rollover(epoch);

            return self.poll(cx);
        }

        Poll::Pending
    }
}
