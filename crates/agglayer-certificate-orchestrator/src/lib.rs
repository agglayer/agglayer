use std::{
    collections::BTreeMap,
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use agglayer_clock::{ClockRef, Event};
use agglayer_storage::{
    columns::{
        latest_proven_certificate_per_network::ProvenCertificate,
        latest_settled_certificate_per_network::SettledCertificate,
    },
    stores::{
        EpochStoreReader, EpochStoreWriter, PendingCertificateReader, PendingCertificateWriter,
        PerEpochReader, PerEpochWriter, StateReader, StateWriter,
    },
};
use agglayer_types::{CertificateId, Height, NetworkId};
use arc_swap::ArcSwap;
use futures_util::{stream::FuturesUnordered, FutureExt, Stream, StreamExt};
use network_task::{NetworkTask, NewCertificate};
use tokio::{
    sync::mpsc::{self, Receiver},
    task::JoinHandle,
};
use tokio_util::sync::{CancellationToken, WaitForCancellationFutureOwned};
use tracing::{debug, error, warn};

mod certifier;
mod epoch_packer;
mod error;
mod network_task;

#[cfg(test)]
mod tests;

pub use certifier::{CertificateInput, Certifier, CertifierOutput, CertifierResult};
pub use epoch_packer::EpochPacker;
pub use error::{CertificationError, Error, PreCertificationError};

const MAX_POLL_READS: usize = 1_000;

pub type EpochPackingTasks =
    FuturesUnordered<Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'static>>>;

pub type NetworkTasks =
    FuturesUnordered<Pin<Box<dyn Future<Output = Result<NetworkId, Error>> + Send + 'static>>>;

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
    E,
    CertifierClient,
    PendingStore,
    EpochsStore,
    PerEpochStore,
    StateStore,
> {
    /// Epoch packing task resolver.
    epoch_packing_tasks: EpochPackingTasks,
    /// Epoch packing task builder.
    epoch_packing_task_builder: Arc<E>,
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
}

impl<E, CertifierClient, PendingStore, EpochsStore, PerEpochStore, StateStore>
    CertificateOrchestrator<
        E,
        CertifierClient,
        PendingStore,
        EpochsStore,
        PerEpochStore,
        StateStore,
    >
where
    PendingStore: PendingCertificateReader,
{
    const DEFAULT_CERTIFICATION_NOTIFICATION_CHANNEL_SIZE: usize = 1000;

    /// Creates a new CertificateOrchestrator instance.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn try_new(
        clock: ClockRef,
        data_receiver: Receiver<(NetworkId, Height, CertificateId)>,
        cancellation_token: CancellationToken,
        epoch_packing_task_builder: E,
        certifier_task_builder: CertifierClient,
        pending_store: Arc<PendingStore>,
        epochs_store: Arc<EpochsStore>,
        current_epoch: Arc<ArcSwap<PerEpochStore>>,
        state_store: Arc<StateStore>,
    ) -> Result<Self, Error> {
        Ok(Self {
            epoch_packing_tasks: FuturesUnordered::new(),
            clock: Box::pin(tokio_stream::StreamExt::filter_map(
                tokio_stream::wrappers::BroadcastStream::new(clock.subscribe()?),
                |v| v.ok(),
            )),
            clock_ref: clock,
            epoch_packing_task_builder: Arc::new(epoch_packing_task_builder),
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
        })
    }
}

#[buildstructor::buildstructor]
impl<E, CertifierClient, PendingStore, EpochsStore, PerEpochStore, StateStore>
    CertificateOrchestrator<
        E,
        CertifierClient,
        PendingStore,
        EpochsStore,
        PerEpochStore,
        StateStore,
    >
where
    CertifierClient: Certifier,
    E: EpochPacker<PerEpochStore = PerEpochStore>,
    PendingStore: PendingCertificateReader + PendingCertificateWriter + 'static,
    EpochsStore: EpochStoreWriter<PerEpochStore = PerEpochStore> + EpochStoreReader + 'static,
    PerEpochStore: PerEpochWriter + PerEpochReader + 'static,
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
    /// # Errors
    ///
    /// This function can't fail but returns a Result for convenience and future
    ///
    /// evolution.
    #[allow(clippy::too_many_arguments)]
    #[builder(entry = "builder", exit = "start", visibility = "pub")]
    pub async fn start(
        clock: ClockRef,
        data_receiver: Receiver<(NetworkId, Height, CertificateId)>,
        cancellation_token: CancellationToken,
        epoch_packing_task_builder: E,
        certifier_task_builder: CertifierClient,
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
            pending_store.clone(),
            epochs_store,
            current_epoch,
            state_store,
        )?;

        // Try to spawn the certifier tasks for the next height of each network
        for ProvenCertificate(_, network_id, _height) in
            pending_store.get_current_proven_height()?
        {
            orchestrator.spawn_network_task(network_id)?;
        }

        let handle = tokio::spawn(orchestrator);

        Ok(handle)
    }
}

impl<E, CertifierClient, PendingStore, EpochsStore, PerEpochStore, StateStore>
    CertificateOrchestrator<
        E,
        CertifierClient,
        PendingStore,
        EpochsStore,
        PerEpochStore,
        StateStore,
    >
where
    CertifierClient: Certifier,
    E: EpochPacker<PerEpochStore = PerEpochStore>,
    PendingStore: PendingCertificateReader + PendingCertificateWriter + 'static,
    EpochsStore: EpochStoreWriter<PerEpochStore = PerEpochStore> + EpochStoreReader + 'static,
    StateStore: StateReader + StateWriter + 'static,
    PerEpochStore: PerEpochWriter + PerEpochReader + 'static,
{
    fn spawn_network_task(&mut self, network_id: NetworkId) -> Result<(), Error> {
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
            self.epoch_packing_task_builder.clone(),
            self.clock_ref.clone(),
            network_id,
            receiver,
        )?;

        self.network_tasks
            .push(task.run(self.cancellation_token.clone()).boxed());

        self.spawned_network_tasks.insert(network_id, sender);

        Ok(())
    }

    /// Function that receives the certificates cursor pushed by the RPC module.
    /// This function is responsible for:
    /// - Updating the cursors for the proofs that have been generated so far.
    /// - Spawning the certifier task for the next height of the network.
    fn receive_certificates(
        &mut self,
        cursors: impl IntoIterator<Item = (NetworkId, Height, CertificateId)>,
    ) -> Result<(), Error> {
        for (network_id, height, certificate_id) in cursors {
            self.spawn_network_task(network_id)?;

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

        Ok(())
    }

    /// Function that handles the end of an epoch.
    /// This function is called when the orchestrator receives an EpochEnded
    /// event. The function is responsible for:
    /// - Opening the next epoch.
    /// - Spawning the epoch packing task.
    fn handle_epoch_end(&mut self, epoch: u64) -> Result<(), Error> {
        debug!("Start the settlement of the epoch {}", epoch);

        let closing_epoch = self.current_epoch.load_full();
        if let Err(error) = closing_epoch.start_packing() {
            error!("Failed to pack the epoch {}: {:?}", epoch, error);

            match error {
                agglayer_storage::error::Error::AlreadyPacked(_) => {}
                agglayer_storage::error::Error::DBError(error) => {
                    let msg = format!(
                        "CRITICAL error during packing of epoch {}: {}",
                        epoch, error
                    );
                    error!(msg);
                    self.cancellation_token.cancel();
                    return Err(Error::InternalError(msg));
                }

                // Other errors shouldn't happen
                error => {
                    let msg = format!(
                        "CRITICAL error: Failed to pack the epoch {}: {:?}",
                        epoch, error
                    );
                    error!(msg);
                    return Err(Error::InternalError(msg));
                }
            }
        }

        // TODO: Check for overflow
        let next_epoch = epoch + 1;

        match self
            .epochs_store
            .open_with_start_checkpoint(next_epoch, closing_epoch.get_end_checkpoint())
        {
            Ok(new_epoch) => self.current_epoch.store(Arc::new(new_epoch)),
            Err(error) => {
                let msg = format!(
                    "CRITICAL error: Failed to open the next epoch {}: {:?}",
                    next_epoch, error
                );

                error!(msg);

                return Err(Error::InternalError(msg));
            }
        }

        Ok(())
    }

    fn handle_epoch_packing_result(&mut self) {}
}

impl<E, A, PendingStore, EpochsStore, PerEpochStore, StateStore> Future
    for CertificateOrchestrator<E, A, PendingStore, EpochsStore, PerEpochStore, StateStore>
where
    A: Certifier,
    E: EpochPacker<PerEpochStore = PerEpochStore>,
    PendingStore: PendingCertificateReader + PendingCertificateWriter + 'static,
    EpochsStore: EpochStoreWriter<PerEpochStore = PerEpochStore> + EpochStoreReader + 'static,
    StateStore: StateReader + StateWriter + 'static,
    PerEpochStore: PerEpochWriter + PerEpochReader + 'static,
{
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Check if the orchestrator has been cancelled and should shutdown.
        if self.cancellation_token_future.as_mut().poll(cx).is_ready() {
            debug!("Certificate orchestrator cancelled by token");

            return Poll::Ready(());
        }

        // Poll the notification tasks to check for
        match self.network_tasks.poll_next_unpin(cx) {
            Poll::Ready(Some(Ok(network_id))) => {
                warn!("Network task for {} completed successfully", network_id);
                _ = self.spawned_network_tasks.remove(&network_id);
            }

            Poll::Ready(Some(Err(error))) => {
                warn!(
                    "Network task Critical error during p-proof generation: {:?}",
                    error
                );
                // TODO: Need to find a way to remove the task
            }
            Poll::Ready(None) => {}
            Poll::Pending => {}
        }

        // Poll the notification tasks to check if any have errored.
        match self.epoch_packing_tasks.poll_next_unpin(cx) {
            Poll::Ready(Some(Err(error))) => {
                error!("Error during epoch packing: {:?}", error)
            }
            Poll::Ready(Some(Ok(()))) => {
                debug!("Successfully settled the epoch");

                self.handle_epoch_packing_result();
            }
            _ => {}
        }

        let mut received = vec![];
        if let Poll::Ready(1usize..) =
            self.data_receiver
                .poll_recv_many(cx, &mut received, MAX_POLL_READS)
        {
            if let Err(e) = self.receive_certificates(received) {
                error!("Failed to handle a group of certificates: {e:?}");
            }

            return self.poll(cx);
        }

        if let Poll::Ready(Some(Event::EpochEnded(epoch))) = self.clock.poll_next_unpin(cx) {
            debug!("Epoch change event received: {}", epoch);

            if let Err(error) = self.handle_epoch_end(epoch) {
                error!("Failed to handle the EpochEnded event: {:?}", error);
            }

            return self.poll(cx);
        }

        Poll::Pending
    }
}
