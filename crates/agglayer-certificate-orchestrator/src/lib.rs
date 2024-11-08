use std::{
    collections::{BTreeMap, HashMap},
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use agglayer_clock::Event;
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
use agglayer_types::LocalNetworkStateData;
use agglayer_types::{
    Certificate, CertificateId, CertificateIndex, CertificateStatus, CertificateStatusError,
    Height, NetworkId, ProofVerificationError,
};
use arc_swap::ArcSwap;
use futures_util::{future::BoxFuture, Stream, StreamExt};
use pessimistic_proof::ProofError;
use tokio::{
    sync::mpsc::Receiver,
    task::{JoinHandle, JoinSet},
};
use tokio_util::sync::{CancellationToken, WaitForCancellationFutureOwned};
use tracing::{debug, error, info, warn};

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
pub struct CertificateOrchestrator<
    C,
    E,
    CertifierClient,
    PendingStore,
    EpochsStore,
    PerEpochStore,
    StateStore,
> {
    /// Certificate settlement task resolver.
    certificate_settlement_tasks: JoinSet<Result<(NetworkId, SettledCertificate), Error>>,
    /// Epoch packing task resolver.
    epoch_packing_tasks: JoinSet<Result<(), Error>>,
    /// Epoch packing task builder.
    epoch_packing_task_builder: Arc<E>,
    /// Certifier task resolver.
    certifier_tasks: JoinSet<Result<CertifierOutput, Error>>,
    /// Certifier task builder.
    certifier_task_builder: Arc<CertifierClient>,
    /// Global network state.
    global_state: GlobalState,
    /// Clock stream to receive EpochEnded events.
    clock: C,
    /// Receiver for certificates coming from CDKs.
    data_receiver: Receiver<(NetworkId, Height, CertificateId)>,
    /// Cancellation token for graceful shutdown.
    cancellation_token: Pin<Box<WaitForCancellationFutureOwned>>,

    state_store: Arc<StateStore>,
    pending_store: Arc<PendingStore>,
    #[allow(unused)]
    epochs_store: Arc<EpochsStore>,
    /// The current epoch considered by the orchestrator.
    current_epoch: ArcSwap<PerEpochStore>,

    pending_state_update: HashMap<CertificateId, LocalNetworkStateData>,
}

impl<C, E, CertifierClient, PendingStore, EpochsStore, PerEpochStore, StateStore>
    CertificateOrchestrator<
        C,
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
    /// Creates a new CertificateOrchestrator instance.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn try_new(
        clock: C,
        data_receiver: Receiver<(NetworkId, Height, CertificateId)>,
        cancellation_token: CancellationToken,
        epoch_packing_task_builder: E,
        certifier_task_builder: CertifierClient,
        pending_store: Arc<PendingStore>,
        epochs_store: Arc<EpochsStore>,
        current_epoch: ArcSwap<PerEpochStore>,
        state_store: Arc<StateStore>,
    ) -> Result<Self, Error> {
        Ok(Self {
            certificate_settlement_tasks: JoinSet::new(),
            epoch_packing_tasks: JoinSet::new(),
            certifier_tasks: JoinSet::new(),
            clock,
            epoch_packing_task_builder: Arc::new(epoch_packing_task_builder),
            certifier_task_builder: Arc::new(certifier_task_builder),
            global_state: Default::default(),
            data_receiver,
            cancellation_token: Box::pin(cancellation_token.cancelled_owned()),
            pending_store,
            epochs_store,
            current_epoch,
            state_store,
            pending_state_update: Default::default(),
        })
    }
}

#[buildstructor::buildstructor]
impl<C, E, CertifierClient, PendingStore, EpochsStore, PerEpochStore, StateStore>
    CertificateOrchestrator<
        C,
        E,
        CertifierClient,
        PendingStore,
        EpochsStore,
        PerEpochStore,
        StateStore,
    >
where
    C: Stream<Item = Event> + Unpin + Send + 'static,
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
        clock: C,
        data_receiver: Receiver<(NetworkId, Height, CertificateId)>,
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
        for ProvenCertificate(_, network_id, height) in pending_store.get_current_proven_height()? {
            let local_state = orchestrator
                .global_state
                .entry(network_id)
                .or_default()
                .clone();

            let task = orchestrator.certifier_task_builder.clone();
            orchestrator
                .certifier_tasks
                .spawn(async move { task.certify(local_state, network_id, height + 1)?.await });
        }

        let handle = tokio::spawn(orchestrator);

        Ok(handle)
    }
}

impl<C, E, CertifierClient, PendingStore, EpochsStore, PerEpochStore, StateStore>
    CertificateOrchestrator<
        C,
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
    PendingStore: PendingCertificateReader + PendingCertificateWriter,
    EpochsStore: EpochStoreWriter<PerEpochStore = PerEpochStore> + EpochStoreReader + 'static,
    StateStore: StateReader + StateWriter + 'static,
    PerEpochStore: PerEpochWriter + PerEpochReader + 'static,
{
    /// Function that receives the certificates cursor pushed by the RPC module.
    /// This function is responsible for:
    /// - Updating the cursors for the proofs that have been generated so far.
    /// - Spawning the certifier task for the next height of the network.
    fn receive_certificates(
        &mut self,
        cursors: &[(NetworkId, Height, CertificateId)],
    ) -> Result<(), Error> {
        for (network_id, height, _) in cursors {
            let current_height = self
                .pending_store
                .get_current_proven_height_for_network(network_id)?;

            match current_height {
                None if *height == 0 => {
                    self.spawn_certifier_task(*network_id, *height);
                }
                Some(cursor_height) => {
                    if cursor_height == height - 1 {
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

        match self.epochs_store.open(next_epoch) {
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
            .spawn(async move { task.pack(closing_epoch)?.await });

        Ok(())
    }

    fn handle_epoch_packing_result(&mut self) {
        match self.pending_store.get_current_proven_height() {
            Err(error) => error!(
                "CRITICAL: Failed to get the current proven height for all networks: {:?}",
                error
            ),
            Ok(certificates) => {
                // Try to spawn the certifier tasks for the next height of each network
                for ProvenCertificate(_, network_id, height) in certificates {
                    self.spawn_certifier_task(network_id, height + 1);
                }
            }
        }
    }
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
impl<C, E, CertifierClient, PendingStore, EpochsStore, PerEpochStore, StateStore>
    CertificateOrchestrator<
        C,
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
    PendingStore: PendingCertificateReader + PendingCertificateWriter,
    StateStore: StateReader + StateWriter,
    PerEpochStore: PerEpochWriter + PerEpochReader + 'static,
{
    fn spawn_certifier_task(&mut self, network: NetworkId, height: Height) {
        let local_state = self.global_state.entry(network).or_default().clone();
        let task = self.certifier_task_builder.clone();
        debug!(
            "Spawning certifier task for network {} at height {}",
            network, height
        );
        self.certifier_tasks
            .spawn(async move { task.certify(local_state, network, height)?.await });
    }

    fn handle_settlement_result(
        &mut self,
        settlement_result: Result<(NetworkId, SettledCertificate), Error>,
    ) -> Result<(), ()> {
        match settlement_result {
            Ok((
                network_id,
                SettledCertificate(certificate_id, height, epoch_number, _certificate_index),
            )) => {
                info!(
                    hash = certificate_id.to_string(),
                    "Certificate {certificate_id} settled on L1 for network {network_id} at \
                     height {height}",
                );

                match self.pending_state_update.remove(&certificate_id) {
                    Some(state) => {
                        if let Some(old) = self.global_state.insert(network_id, state.clone()) {
                            debug!(
                                "Updated the state for network {} with the new state {} > {}",
                                network_id,
                                old.get_roots().display_to_hex(),
                                state.get_roots().display_to_hex()
                            );
                        } else {
                            debug!(
                                "Updated the state for network {} with the new state [] > {}",
                                network_id,
                                state.get_roots().display_to_hex()
                            );
                        }
                    }
                    None => {
                        error!(
                            "CRITICAL: Failed to find the state for the certificate {} to apply \
                             on global state",
                            certificate_id
                        );
                        return Err(());
                    }
                };
                let current_epoch = self.current_epoch.load();
                if current_epoch.get_epoch_number() == epoch_number + 1 {
                    self.spawn_certifier_task(network_id, height + 1);
                }
            }
            Err(_) => todo!(),
        }

        Ok(())
    }

    fn handle_certifier_result(
        &mut self,
        certifier_result: Result<CertifierOutput, Error>,
    ) -> Result<(), ()> {
        match certifier_result {
            Ok(CertifierOutput {
                height,
                network,
                certificate,
                new_state,
            }) => {
                self.on_proven_certificate(height, network, certificate, new_state)?;
            }

            // If we received a `CertificateNotFound` error, it means that the certificate was not
            // found in the pending store. This can happen if we try to certify a certificate that
            // has not been received yet. When received, the certificate will be stored in the
            // pending store and the certifier task will be spawned again.
            Err(Error::CertificateNotFound(network_id, height)) => {
                // TODO: Check if `CertificateHeader` if present, spawn next height
                debug!(
                    "Received a proof certification error for a certificate that is not found for \
                     network {} at {}",
                    network_id, height
                );
            }

            Err(Error::ProofAlreadyExists(network_id, height, certificate_id)) => {
                warn!(
                    hash = certificate_id.to_string(),
                    "Received a proof certification error for a proof that already exists for \
                     network {} at height {}",
                    network_id,
                    height
                );

                self.on_proof_already_exists(network_id, height, certificate_id)?;
            }
            Err(error) => {
                warn!("Error during certification process: {}", error);
                let certificate_error: Option<(CertificateId, CertificateStatusError)> = match error
                {
                    Error::TrustedSequencerNotFound(certificate_id, network) => Some((
                        certificate_id,
                        CertificateStatusError::TrustedSequencerNotFound(network),
                    )),
                    Error::ProofVerificationFailed {
                        source,
                        certificate_id,
                    } => Some((certificate_id, source.into())),

                    Error::ProverExecutionFailed {
                        source,
                        certificate_id,
                    } => Some((
                        certificate_id,
                        CertificateStatusError::ProofGenerationError {
                            generation_type: agglayer_types::GenerationType::Prover,
                            source,
                        },
                    )),

                    Error::NativeExecutionFailed {
                        source,
                        certificate_id,
                    } => Some((
                        certificate_id,
                        CertificateStatusError::ProofGenerationError {
                            generation_type: agglayer_types::GenerationType::Native,
                            source,
                        },
                    )),

                    Error::Types {
                        source,
                        certificate_id,
                    } => Some((certificate_id, source.into())),

                    Error::Storage(error) => {
                        warn!(
                            "Storage error happened in the certification process: {:?}",
                            error
                        );
                        None
                    }
                    _ => None,
                };

                if let Some((certificate_id, error)) = certificate_error {
                    if self
                        .state_store
                        .update_certificate_header_status(
                            &certificate_id,
                            &CertificateStatus::InError { error },
                        )
                        .is_err()
                    {
                        error!(
                            hash = certificate_id.to_string(),
                            "Certificate in error and failed to update the certificate header \
                             status"
                        );
                    }
                }
            }
        }
        Ok(())
    }

    /// Context:
    ///
    /// At one point in time, there is at most one certifier task per network
    /// running. The certifier task try to generate a proof based on
    /// a certificate. The certifier task doesn't know about other tasks nor if
    /// the certificate will be included in an epoch. The Orchestrator is
    /// the one that is responsible to decide if a proof is valid and should
    /// be included in an epoch.
    ///
    /// Based on the current context of the orchestrator, we can
    /// determine the following:
    ///
    /// 1. If the state doesn't know the network and the height is 0, we update
    ///    the state. This is the first certificate for this network.
    /// 2. If the state knows the network and the height is the next one, we
    ///    update the state. This is the next certificate for this network.
    /// 3. If the state doesn't know the network and the height is not 0, we
    ///    ignore the proof.
    /// 4. If the state knows the network and the height is not the next
    ///    expected one, we ignore the proof.
    ///
    /// When a generated proof is accepted:
    /// - We update the cursor for the network.
    /// - We update the latest proven certificate for the network.
    /// - We do not remove the pending certificate. (as it needs to be included
    ///   in an epoch)
    /// - We spawn the next certificate for the network.
    fn on_proven_certificate(
        &mut self,
        height: Height,
        network: NetworkId,
        certificate: Certificate,
        new_state: LocalNetworkStateData,
    ) -> Result<(), ()> {
        let current_height = self
            .pending_store
            .get_current_proven_height_for_network(&network)
            .map_err(|error| {
                error!(
                    hash = certificate.hash().to_string(),
                    "Failed to get the current proven height for network {}: {:?}", network, error
                );
            })?;

        let certificate_id = certificate.hash();

        match current_height {
            // - 1. If the state doesn't know the network and the height is 0, we update the state.
            //   This is the first certificate for this network.
            None if height == 0 => {
                // TODO: Handle error if fails to set the latest proven certificate
                if let Err(error) = self
                    .pending_store
                    .set_latest_proven_certificate_per_network(&network, &height, &certificate_id)
                {
                    error!(
                        hash = certificate_id.to_string(),
                        "Failed to set the latest proven certificate per network: {:?}", error
                    );
                }

                if let Err(error) = self
                    .state_store
                    .update_certificate_header_status(&certificate_id, &CertificateStatus::Proven)
                {
                    error!(
                        hash = certificate_id.to_string(),
                        "Failed to update the certificate header status: {:?}", error
                    );
                }

                let current_epoch = self.current_epoch.load_full();

                self.try_adding_certificate(current_epoch, network, height, certificate_id);
                self.pending_state_update.insert(certificate_id, new_state);
            }

            // - 2. If the state knows the network and the height is the next one, we update the
            //   state. This is the next certificate for this network.
            Some(cursor_height) if cursor_height + 1 == height => {
                // TODO: Handle error if fails to set the latest proven certificate
                if let Err(error) = self
                    .pending_store
                    .set_latest_proven_certificate_per_network(&network, &height, &certificate_id)
                {
                    error!(
                        hash = certificate_id.to_string(),
                        "Failed to set the latest proven certificate per network: {:?}", error
                    );
                }

                if let Err(error) = self
                    .state_store
                    .update_certificate_header_status(&certificate_id, &CertificateStatus::Proven)
                {
                    error!(
                        hash = certificate_id.to_string(),
                        "Failed to update the certificate header status: {:?}", error
                    );
                }

                let current_epoch = self.current_epoch.load_full();

                self.try_adding_certificate(current_epoch, network, height, certificate_id);
                self.pending_state_update.insert(certificate_id, new_state);
            }

            // - 3. If the state doesn't know the network and the height is not 0, we ignore the
            //   proof.
            None => {
                warn!(
                    hash = certificate_id.to_string(),
                    "Received a proof generated for a certificate at height {} for a network that \
                     is not being tracked: {}",
                    height,
                    network
                );

                if let Err(error) = self
                    .pending_store
                    .remove_pending_certificate(network, height)
                {
                    error!(
                        hash = certificate_id.to_string(),
                        "Failed to remove the pending certificate: {:?}", error
                    );
                }
                if let Err(error) = self.pending_store.remove_generated_proof(&certificate_id) {
                    error!(
                        hash = certificate_id.to_string(),
                        "Failed to remove the generated proof: {:?}", error
                    );
                }

                return Err(());
            }

            // - 4. If the state knows the network and the height is not the next expected one, we
            //   ignore the proof.
            Some(cursor_height) => {
                warn!(
                    hash = certificate_id.to_string(),
                    "Received a certificate with an unexpected height: {} for network {} which is \
                     currently at {}",
                    height,
                    network,
                    cursor_height
                );

                return Err(());
            }
        }

        Ok(())
    }

    /// If we received a `ProofAlreadyExists` error, it means that the proof has
    /// already been generated for this certificate. This should not happen
    /// unless the node crashes in the middle of the certification process.
    /// If so, we check the current state and decide if:
    ///
    /// 1. The state doesn't know the network and the height is not 0, we remove
    ///    the proof.
    /// 2. The state doesn't know the network and the height is 0. We update the
    ///    state if needed.
    /// 3. The state knows the network and the height is the current height, we
    ///    schedule the next certificate.
    /// 4. The state knows the network and the height is the next one, we update
    ///    the state and schedule the next certificate.
    /// 5. The state knows the network and the height is the previous one, we do
    ///    nothing.
    /// 6. The state knows the network and the height is not the next expected
    ///    one, we remove the proof.
    fn on_proof_already_exists(
        &mut self,
        network_id: NetworkId,
        height: Height,
        certificate_id: agglayer_types::Hash,
    ) -> Result<(), ()> {
        let current_height = self
            .pending_store
            .get_current_proven_height_for_network(&network_id)
            .map_err(|error| {
                error!(
                    hash = certificate_id.to_string(),
                    "Failed to get the current proven height for network {}: {:?}",
                    network_id,
                    error
                );
            })?;

        match current_height {
            // 1. The state doesn't know the network and the height is not 0, we remove the proof.
            None if height != 0 => {
                warn!(
                    hash = certificate_id.to_string(),
                    "Received a proof generated for a certificate at height {} for a network that \
                     is not being tracked: {}",
                    height,
                    network_id
                );

                if let Err(error) = self.pending_store.remove_generated_proof(&certificate_id) {
                    error!(
                        hash = certificate_id.to_string(),
                        "Failed to remove the proof: {:?}", error
                    );
                }

                return Ok(());
            }
            // 2. The state doesn't know the network and the height is 0. We update the state if
            //   needed.
            None => {
                // Context:
                // | CertificateHeaderStatus      | action                     |
                // |------------------------------|----------------------------|
                // | Pending                      | Update to Proven           |
                // | Proven                       | Do nothing                 |
                // | Candiate / Settled / InError | Remove certificate + proof |
                //
                // ProofAlreadyExists is an error that can happen only if we have a pending
                // certificate.

                let certificate_header = if let Ok(Some(certificate_header)) = self
                    .state_store
                    .get_certificate_header_by_cursor(network_id, height)
                {
                    certificate_header
                } else {
                    if let Err(error) = self
                        .pending_store
                        .remove_pending_certificate(network_id, height)
                    {
                        error!(
                            hash = certificate_id.to_string(),
                            "Failed to remove the pending certificate: {:?}", error
                        );
                    }
                    if let Err(error) = self.pending_store.remove_generated_proof(&certificate_id) {
                        error!(
                            hash = certificate_id.to_string(),
                            "Failed to remove the proof: {:?}", error
                        );
                    }
                    // This should not happen as ProofAlreadyExists should only
                    // happen if we have a pending certificate.
                    warn!(
                        hash = certificate_id.to_string(),
                        "Failed to find the pending certificate header for proven certificate for \
                         network {} at height {}",
                        network_id,
                        height
                    );

                    return Ok(());
                };

                match certificate_header.status {
                    CertificateStatus::Pending => {
                        if let Err(error) = self.state_store.update_certificate_header_status(
                            &certificate_id,
                            &CertificateStatus::Proven,
                        ) {
                            error!(
                                hash = certificate_id.to_string(),
                                "Failed to update the certificate header status: {:?}", error
                            );
                        }
                        if let Err(error) = self
                            .pending_store
                            .set_latest_proven_certificate_per_network(
                                &network_id,
                                &height,
                                &certificate_id,
                            )
                        {
                            error!(
                                hash = certificate_id.to_string(),
                                "Failed to set the latest proven certificate per network: {:?}",
                                error
                            );
                        }
                    }
                    CertificateStatus::Candidate
                    | CertificateStatus::Settled
                    | CertificateStatus::InError { .. } => {
                        if let Err(error) = self
                            .pending_store
                            .remove_pending_certificate(network_id, height)
                        {
                            error!(
                                hash = certificate_id.to_string(),
                                "Failed to remove the pending certificate: {:?}", error
                            );
                        }

                        if let Err(error) =
                            self.pending_store.remove_generated_proof(&certificate_id)
                        {
                            error!(
                                hash = certificate_id.to_string(),
                                "Failed to remove the proof: {:?}", error
                            );
                        }
                    }
                    CertificateStatus::Proven => {}
                }

                self.spawn_certifier_task(network_id, height + 1);
            }

            // 3. The state knows the network and the height is the current height, we schedule the
            //    next certificate.
            // 4. The state knows the network and the height is the next one, we update the state
            //    and schedule the next certificate.
            // 5. The state knows the network and the height is the previous one, we do nothing.
            // 6. The state knows the network and the height is not the next expected one, we
            //   remove the proof.
            Some(cursor_height) => {
                // Context:
                // | pending certificate  | CertificateHeader | action              |
                // |----------------------|-------------------|---------------------|
                // | Found                | Not found         | Create the header   |
                // | Found                | Found             | Delete the pending  |
                //
                // ProofAlreadyExists is an error that can happen only if we have a pending
                // certificate.
                let certificate_header = if let Ok(Some(certificate_header)) = self
                    .state_store
                    .get_certificate_header_by_cursor(network_id, height)
                {
                    certificate_header
                } else {
                    if let Err(error) = self
                        .pending_store
                        .remove_pending_certificate(network_id, height)
                    {
                        error!(
                            hash = certificate_id.to_string(),
                            "Failed to remove the pending certificate: {:?}", error
                        );
                    }
                    if let Err(error) = self.pending_store.remove_generated_proof(&certificate_id) {
                        error!(
                            hash = certificate_id.to_string(),
                            "Failed to remove the proof: {:?}", error
                        );
                    }
                    // This should not happen as ProofAlreadyExists should only
                    // happen if we have a pending certificate.
                    warn!(
                        hash = certificate_id.to_string(),
                        "Failed to find the pending certificate header for proven proof for \
                         network {} at height {}",
                        network_id,
                        height
                    );

                    return Ok(());
                };

                match certificate_header.status {
                    CertificateStatus::Pending => {
                        if let Err(error) = self.state_store.update_certificate_header_status(
                            &certificate_id,
                            &CertificateStatus::Proven,
                        ) {
                            error!(
                                hash = certificate_id.to_string(),
                                "Failed to update the certificate header status: {:?}", error
                            );
                        }
                        if cursor_height + 1 == height {
                            if let Err(error) = self
                                .pending_store
                                .set_latest_proven_certificate_per_network(
                                    &network_id,
                                    &height,
                                    &certificate_id,
                                )
                            {
                                error!(
                                    hash = certificate_id.to_string(),
                                    "Failed to set the latest proven certificate per network: {:?}",
                                    error
                                );
                            }
                        }
                    }
                    CertificateStatus::Candidate
                    | CertificateStatus::Settled
                    | CertificateStatus::InError { .. } => {
                        if let Err(error) = self
                            .pending_store
                            .remove_pending_certificate(network_id, height)
                        {
                            error!(
                                hash = certificate_id.to_string(),
                                "Failed to remove the pending certificate: {:?}", error
                            );
                        }
                        if let Err(error) =
                            self.pending_store.remove_generated_proof(&certificate_id)
                        {
                            error!(
                                hash = certificate_id.to_string(),
                                "Failed to remove the proof: {:?}", error
                            );
                        }
                    }

                    CertificateStatus::Proven => {}
                }
                self.spawn_certifier_task(network_id, height + 1);
            }
        }

        Ok(())
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
        self.certificate_settlement_tasks.spawn(async move {
            task.settle_certificate(related_epoch, certificate_index, certificate_id)?
                .await
        });

        Ok(())
    }
}

impl<C, E, A, PendingStore, EpochsStore, PerEpochStore, StateStore> Future
    for CertificateOrchestrator<C, E, A, PendingStore, EpochsStore, PerEpochStore, StateStore>
where
    C: Stream<Item = Event> + Send + Unpin + 'static,
    A: Certifier,
    E: EpochPacker<PerEpochStore = PerEpochStore>,
    PendingStore: PendingCertificateReader + PendingCertificateWriter,
    EpochsStore: EpochStoreWriter<PerEpochStore = PerEpochStore> + EpochStoreReader + 'static,
    StateStore: StateReader + StateWriter + 'static,
    PerEpochStore: PerEpochWriter + PerEpochReader + 'static,
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
                debug!("Certifier task completed successfully");
                _ = self.handle_certifier_result(proof_result);
            }

            Poll::Ready(Some(Err(error))) => {
                warn!("Critical error during p-proof generation: {:?}", error);
            }
            Poll::Ready(None) => {}
            Poll::Pending => {}
        }

        match self.certificate_settlement_tasks.poll_join_next(cx) {
            Poll::Ready(Some(Ok(settlement_result))) => {
                debug!("Certificate settlement task completed");
                _ = self.handle_settlement_result(settlement_result);
            }
            Poll::Ready(Some(Err(error))) => {
                warn!("Critical error during settlement: {:?}", error);
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

                self.handle_epoch_packing_result();
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

            return self.poll(cx);
        }

        Poll::Pending
    }
}

/// Epoch Packer used to gather all the proofs generated on-the-go
/// and to submit them in a settlement tx to the L1.
pub trait EpochPacker: Unpin + Send + Sync + 'static {
    type PerEpochStore: PerEpochWriter + PerEpochReader;
    /// Pack an epoch for settlement on the L1
    fn pack(
        &self,
        closing_epoch: Arc<Self::PerEpochStore>,
    ) -> Result<BoxFuture<Result<(), Error>>, Error>;

    fn settle_certificate(
        &self,
        related_epoch: Arc<Self::PerEpochStore>,
        certificate_index: CertificateIndex,
        certificate_id: CertificateId,
    ) -> Result<SettlementFuture, Error>;
}

pub type SettlementFuture<'a> = BoxFuture<'a, Result<(NetworkId, SettledCertificate), Error>>;

pub trait CertificateInput: Clone {
    fn network_id(&self) -> NetworkId;
}

impl CertificateInput for Certificate {
    fn network_id(&self) -> NetworkId {
        self.network_id
    }
}

#[derive(Debug, Clone)]
pub struct CertifierOutput {
    pub certificate: Certificate,
    pub height: Height,
    pub new_state: LocalNetworkStateData,
    pub network: NetworkId,
}

pub type CertifierResult = Result<BoxFuture<'static, Result<CertifierOutput, Error>>, Error>;

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
    #[error("proof already exists for network {0} at height {1} for certificate {2}")]
    ProofAlreadyExists(NetworkId, Height, CertificateId),
    #[error("proof verification failed")]
    ProofVerificationFailed {
        certificate_id: CertificateId,
        source: ProofVerificationError,
    },
    #[error("prover execution failed: {source}")]
    ProverExecutionFailed {
        certificate_id: CertificateId,
        source: ProofError,
    },
    #[error("native execution failed: {source:?}")]
    NativeExecutionFailed {
        certificate_id: CertificateId,
        source: ProofError,
    },
    #[error("Type error: {source}")]
    Types {
        certificate_id: CertificateId,
        source: agglayer_types::Error,
    },
    #[error("Storage error: {0}")]
    Storage(#[from] agglayer_storage::error::Error),
    #[error("internal error")]
    InternalError,
    #[error("Serialize error")]
    Serialize {
        certificate_id: CertificateId,
        source: bincode::Error,
    },
    #[error("Deserialize error")]
    Deserialize {
        certificate_id: CertificateId,
        source: bincode::Error,
    },
    #[error("The status of the certificate is invalid")]
    InvalidCertificateStatus,

    #[error("Failed to settle the certificate {certificate_id}: {error}")]
    SettlementError {
        certificate_id: CertificateId,
        error: String,
    },
    #[error(
        "Failed to retrieve the trusted sequencer address for network {1} during proving phase \
         for {0}"
    )]
    TrustedSequencerNotFound(CertificateId, NetworkId),
}
