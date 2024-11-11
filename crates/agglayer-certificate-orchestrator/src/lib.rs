use std::{
    collections::{BTreeMap, HashMap},
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
use agglayer_types::{Certificate, CertificateId, CertificateIndex, Height, NetworkId};
use arc_swap::ArcSwap;
use futures_util::{stream::FuturesUnordered, FutureExt, Stream, StreamExt};
use network_task::NetworkTask;
use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
};
use tokio_util::sync::{CancellationToken, WaitForCancellationFutureOwned};
use tracing::{debug, error, info, warn};

mod certifier;
mod epoch_packer;
mod error;
mod network_task;

mod submitter;

#[cfg(test)]
mod tests;

pub use certifier::{CertificateInput, Certifier, CertifierOutput, CertifierResult};
pub use epoch_packer::{EpochPacker, SettlementFuture};
pub use error::{CertificationError, Error, InitialCheckError, PreCertificationError};
pub use submitter::Submitter as CertificateSubmitter;

const MAX_POLL_READS: usize = 1_000;

pub type EpochPackingTasks =
    FuturesUnordered<Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'static>>>;

pub type NetworkTasks =
    FuturesUnordered<Pin<Box<dyn Future<Output = Result<NetworkId, Error>> + Send + 'static>>>;

pub type SettlementTasks = FuturesUnordered<
    Pin<Box<dyn Future<Output = Result<(NetworkId, SettledCertificate), Error>> + Send + 'static>>,
>;

/// A response to the certificate submission.
pub type CertResponse = Result<(), InitialCheckError>;

/// A certificate response sender.
pub type CertResponseSender = oneshot::Sender<CertResponse>;

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
    data_receiver: mpsc::Receiver<(Certificate, CertResponseSender)>,
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
    current_epoch: ArcSwap<PerEpochStore>,

    /// Network tasks that are currently running, with their associated
    /// notifier.
    spawned_network_tasks: BTreeMap<NetworkId, mpsc::Sender<(Certificate, CertResponseSender)>>,

    /// Notifiers for the settlement of the certificates.
    settlement_notifier:
        HashMap<CertificateId, oneshot::Sender<Result<SettledCertificate, String>>>,

    /// Network task future resolver.
    network_tasks: NetworkTasks,
    /// Certificate settlement task future resolver.
    settlement_tasks: SettlementTasks,

    /// Channel to receive notifications of proven certificatesa in order to
    /// settle them.
    certification_notification: mpsc::Receiver<(
        oneshot::Sender<Result<SettledCertificate, String>>,
        ProvenCertificate,
    )>,
    /// Channel to pass to NetworkTask for them to send notifications of proven
    /// certificates.
    certification_notification_sender: mpsc::Sender<(
        oneshot::Sender<Result<SettledCertificate, String>>,
        ProvenCertificate,
    )>,
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
        data_receiver: mpsc::Receiver<(Certificate, CertResponseSender)>,
        cancellation_token: CancellationToken,
        epoch_packing_task_builder: E,
        certifier_task_builder: CertifierClient,
        pending_store: Arc<PendingStore>,
        epochs_store: Arc<EpochsStore>,
        current_epoch: ArcSwap<PerEpochStore>,
        state_store: Arc<StateStore>,
    ) -> Result<Self, Error> {
        let (certification_notification_sender, certification_notification) =
            mpsc::channel(Self::DEFAULT_CERTIFICATION_NOTIFICATION_CHANNEL_SIZE);

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
            settlement_tasks: FuturesUnordered::new(),
            settlement_notifier: Default::default(),
            certification_notification,
            certification_notification_sender,
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
        data_receiver: mpsc::Receiver<(Certificate, CertResponseSender)>,
        cancellation_token: CancellationToken,
        epoch_packing_task_builder: E,
        certifier_task_builder: CertifierClient,
        pending_store: Arc<PendingStore>,
        epochs_store: Arc<EpochsStore>,
        current_epoch: ArcSwap<PerEpochStore>,
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
            self.certification_notification_sender.clone(),
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
        cursors: impl IntoIterator<Item = (Certificate, CertResponseSender)>,
    ) -> Result<(), Error> {
        for (certificate, reply_sender) in cursors {
            let network_id = certificate.network_id;
            self.spawn_network_task(network_id)?;

            if let Some(sender) = self.spawned_network_tasks.get(&network_id) {
                if let Ok(sender) = sender.try_reserve() {
                    sender.send((certificate, reply_sender));
                } else {
                    let certificate_id = certificate.hash();
                    error!(
                        "Failed to send the certificate {certificate_id} to the network task for \
                         network {network_id}",
                    );
                    let _ = reply_sender.send(Err(InitialCheckError::Internal));
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
        let task = self.epoch_packing_task_builder.clone();

        let closing_epoch = self.current_epoch.load_full();
        // TODO: Check for overflow
        let next_epoch = epoch + 1;

        match self
            .epochs_store
            .open_with_start_checkpoint(next_epoch, closing_epoch.get_end_checkpoint())
        {
            Ok(new_epoch) => self.current_epoch.store(Arc::new(new_epoch)),
            Err(error) => {
                error!(
                    "CRITICAL error: Failed to open the next epoch {}: {:?}",
                    next_epoch, error
                );

                return Err(Error::InternalError);
            }
        }

        self.epoch_packing_tasks
            .push(async move { task.pack(closing_epoch)?.await }.boxed());

        Ok(())
    }

    fn handle_epoch_packing_result(&mut self) {}
}

// This block contains the logic applied to a Certificate.
// The method are executed following this flow:
//
//                      ┌─────────────────────────┐
//                      │                         │
// ┌────────────────────►  spawn certifier task   ◄───────────────────┐
// │                    │                         │                   │
// │                    └────────────┬────────────┘                   │
// │                                 │                                │
// │                                 │                                │
// │                                 │                                │
// │                                 │                                │
// │                                 │                                │
// │                                 │                                │
// │                    ┌────────────▼────────────┐              in some case
// │                    │                         │                   │
// │                    │      handle result      │                   │
// │                    │                         │                   │
// │                    └────────────┬────────────┘                   │
// │                                 │                                │
// │                 ┌───────────────┴────────────────┐               │
// │                 │                                │               │
// │       ┌─────────▼──────────┐      ┌──────────────▼────────────┐  │
// │       │                    │      │                           │  │
// └───────┼     on proven      │      │  on proof already exists  ┼──┘
//         │                    │      │                           │
//         └─────────┬──────────┘      └───────────────────────────┘
//                   │
//                   │
//                   │
//   ┌───────────────▼──────────────────┐
//   │                                  │
//   │  Try adding certificate to epoch │
//   │                                  │
//   └───────────────┬──────────────────┘
//                   │
//                   │
//        ┌──────────▼────────────┐
//        │                       │
//        │  Settle certificate   │
//        │                       │
//        └───────────────────────┘
//
// When a Certificate is proven, we try to add it to the current epoch.
// If we succeed, we try to settle it on L1.
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
    E: EpochPacker<PerEpochStore = PerEpochStore> + 'static,
    PendingStore: PendingCertificateReader + PendingCertificateWriter,
    StateStore: StateReader + StateWriter,
    PerEpochStore: PerEpochWriter + PerEpochReader + 'static,
{
    fn handle_settlement_result(
        &mut self,
        settlement_result: Result<(NetworkId, SettledCertificate), Error>,
    ) -> Result<(), ()> {
        match settlement_result {
            Ok((
                network_id,
                settled @ SettledCertificate(
                    certificate_id,
                    height,
                    _epoch_number,
                    _certificate_index,
                ),
            )) => {
                info!(
                    hash = certificate_id.to_string(),
                    "Certificate {certificate_id} settled on L1 for network {network_id} at \
                     height {height}",
                );

                if let Some(notifier) = self.settlement_notifier.remove(&certificate_id) {
                    if notifier.send(Ok(settled)).is_err() {
                        warn!(
                            "Unable to notify the settlement of the certificate {}",
                            certificate_id
                        );
                    }
                } else {
                    warn!("No notifier found for network {}", network_id);
                }
            }
            Err(Error::SettlementError {
                certificate_id,
                error,
            }) => {
                error!("Error during certificate settlement: {:?}", error);
                if let Some(notifier) = self.settlement_notifier.remove(&certificate_id) {
                    if notifier.send(Err(error)).is_err() {
                        warn!(
                            hash = certificate_id.to_string(),
                            "Unable to notify the settlement error of the certificate {}",
                            certificate_id
                        );
                    }
                } else {
                    warn!(
                        hash = certificate_id.to_string(),
                        "No notifier found for network {}", certificate_id
                    );
                }
            }
            Err(error) => {
                error!("Error during certificate settlement: {:?}", error);
            }
        }

        Ok(())
    }

    fn handle_proven_certificate(
        &mut self,
        (response, ProvenCertificate(certificate_id, network, height)): (
            oneshot::Sender<Result<SettledCertificate, String>>,
            ProvenCertificate,
        ),
    ) {
        let current_epoch = self.current_epoch.load_full();
        if self
            .settlement_notifier
            .insert(certificate_id, response)
            .is_some()
        {
            warn!(
                hash = certificate_id.to_string(),
                "Notification channel already assigned for Certificate {certificate_id} ..."
            );
        }

        self.try_adding_certificate(current_epoch, network, height, certificate_id);
    }

    /// Try to add a certificate to the current epoch and settle it on L1.
    fn try_adding_certificate(
        &mut self,
        current_epoch: Arc<PerEpochStore>,
        network: NetworkId,
        height: Height,
        certificate_id: CertificateId,
    ) {
        match current_epoch.add_certificate(network, height) {
            Err(error) => error!(
                hash = certificate_id.to_string(),
                "Failed to add the certificate to the current epoch: {}", error
            ),
            Ok((epoch_number, certificate_index)) => {
                if let Err(error) =
                    self.settle_certificate(current_epoch, certificate_index, certificate_id)
                {
                    error!(
                        hash = certificate_id.to_string(),
                        "Failed to settle the certificate {} in epoch {}: {:?}",
                        certificate_id,
                        epoch_number,
                        error
                    );
                }
            }
        }
    }

    /// Spawn a task that will settle the certificate on the L1.
    fn settle_certificate(
        &mut self,
        related_epoch: Arc<PerEpochStore>,
        certificate_index: CertificateIndex,
        certificate_id: CertificateId,
    ) -> Result<(), Error> {
        debug!(
            hash = certificate_id.to_string(),
            "Settling the certificate {certificate_id}"
        );

        let task = self.epoch_packing_task_builder.clone();
        self.settlement_tasks.push(
            async move {
                task.settle_certificate(related_epoch, certificate_index, certificate_id)?
                    .await
            }
            .boxed(),
        );

        Ok(())
    }
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

        match self.certification_notification.poll_recv(cx) {
            Poll::Ready(Some(result)) => self.handle_proven_certificate(result),
            Poll::Ready(None) => {}
            Poll::Pending => {}
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

        match self.settlement_tasks.poll_next_unpin(cx) {
            Poll::Ready(Some(settlement_result)) => {
                debug!("Certificate settlement task completed");
                _ = self.handle_settlement_result(settlement_result);
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
