use std::sync::Arc;

use agglayer_clock::ClockRef;
use agglayer_storage::{
    columns::latest_settled_certificate_per_network::SettledCertificate,
    stores::{PendingCertificateReader, PendingCertificateWriter, StateReader, StateWriter},
};
use agglayer_types::{
    primitives::{Digest, Hashable as _},
    CertificateId, CertificateIndex, CertificateStatusError, EpochNumber, Height,
    LocalNetworkStateData, NetworkId, SettlementTxHash,
};
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};

use crate::{certificate_task::CertificateTask, Certifier, Error, SettlementClient};

#[cfg(test)]
mod tests;

/// Message to notify the network task that a new certificate has been received.
#[derive(Debug)]
pub(crate) struct NewCertificate {
    pub(crate) certificate_id: CertificateId,
    pub(crate) height: Height,
}

#[allow(dead_code)] // TODO: Once we have implemented storage properly, all the fields should become used
/// Enum listing all the potential messages that can be sent to the network
/// task.
#[derive(Debug)]
pub enum NetworkTaskMessage {
    /// Get the local network state before a given height.
    GetLocalNetworkStateBeforeHeight {
        height: Height,
        response: oneshot::Sender<Result<Box<LocalNetworkStateData>, CertificateStatusError>>,
    },

    /// Notify the network task that a certificate has been successfully
    /// executed.
    ///
    /// Also encodes the new state of the network after the certificate has been
    /// executed.
    CertificateExecuted {
        height: Height,
        certificate_id: CertificateId,
        new_state: Box<LocalNetworkStateData>,
    },

    /// Notify the network task that a certificate has been successfully proven.
    CertificateProven {
        height: Height,
        certificate_id: CertificateId,
    },

    /// Notify the network task that a certificate is ready for settlement.
    ///
    /// The `settlement_submitted_notifier` is used to notify the certificate
    /// task that the settlement has been successfully submitted.
    CertificateReadyForSettlement {
        height: Height,
        certificate_id: CertificateId,
        settlement_submitted_notifier:
            oneshot::Sender<Result<SettlementTxHash, CertificateStatusError>>,
    },

    /// Notify the network task that a certificate is waiting for settlement to
    /// complete.
    ///
    /// The `settlement_complete_notifier` is used to notify the certificate
    /// task that the settlement has been successfully completed.
    CertificateWaitingForSettlement {
        height: Height,
        certificate_id: CertificateId,
        settlement_tx_hash: SettlementTxHash,
        settlement_complete_notifier:
            oneshot::Sender<Result<(EpochNumber, CertificateIndex), CertificateStatusError>>,
    },

    /// Notify the network task that a certificate has been successfully
    /// settled.
    CertificateSettled {
        height: Height,
        certificate_id: CertificateId,
        settled_certificate: SettledCertificate,
    },

    /// Notify the network task that a certificate has encountered an error.
    CertificateErrored {
        height: Height,
        certificate_id: CertificateId,
        error: CertificateStatusError,
    },
}

/// Network task that is responsible to certify the certificates for a network.
pub(crate) struct NetworkTask<CertifierClient, SettlementClient, PendingStore, StateStore> {
    /// The network id for the network task.
    network_id: NetworkId,
    /// The pending store to read and write the pending certificates.
    pending_store: Arc<PendingStore>,
    /// The state store to read and write the state of the network.
    state_store: Arc<StateStore>,
    /// The certifier client to certify the certificates.
    certifier_client: Arc<CertifierClient>,
    /// The settlement client to settle the certificates.
    settlement_client: Arc<SettlementClient>,
    /// The local network state of the network task.
    local_state: Box<LocalNetworkStateData>,

    /// The clock reference to subscribe to the epoch events and check for
    /// current epoch.
    clock_ref: ClockRef,
    /// The pending local network state that should be applied on receiving
    /// settlement response.
    pending_state: Option<Box<LocalNetworkStateData>>,
    /// The stream of new certificates to certify.
    certificate_stream: mpsc::Receiver<NewCertificate>,
    /// Flag to indicate if the network is at capacity for the current epoch.
    at_capacity_for_epoch: bool,
    /// latest certificate settled
    latest_settled: Option<SettledCertificate>,
}

impl<CertifierClient, Sc, PendingStore, StateStore>
    NetworkTask<CertifierClient, Sc, PendingStore, StateStore>
where
    CertifierClient: 'static + Certifier,
    Sc: 'static + SettlementClient,
    PendingStore: 'static + PendingCertificateReader + PendingCertificateWriter,
    StateStore: 'static + StateReader + StateWriter,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pending_store: Arc<PendingStore>,
        state_store: Arc<StateStore>,
        certifier_client: Arc<CertifierClient>,
        settlement_client: Arc<Sc>,
        clock_ref: ClockRef,
        network_id: NetworkId,
        certificate_stream: mpsc::Receiver<NewCertificate>,
    ) -> Result<Self, Error> {
        info!("Creating a new network task for network {}", network_id);

        let local_state = Box::new(
            state_store
                .read_local_network_state(network_id)?
                .unwrap_or_default(),
        );

        let latest_settled = state_store
            .get_latest_settled_certificate_per_network(&network_id)?
            .map(|(_v, settled)| settled);

        debug!(
            "Local state for network {}: {}",
            network_id,
            local_state.get_roots().display_to_hex()
        );

        Ok(Self {
            network_id,
            pending_store,
            state_store,
            certifier_client,
            local_state,
            clock_ref,
            pending_state: None,
            certificate_stream,
            at_capacity_for_epoch: false,
            latest_settled,
            settlement_client,
        })
    }

    #[tracing::instrument(
        name = "NetworkTask::run",
        skip_all,
        fields(
            network_id = %self.network_id,
        )
    )]
    pub(crate) async fn run(
        mut self,
        cancellation_token: CancellationToken,
    ) -> Result<NetworkId, Error> {
        info!("Starting the network task for network {}", self.network_id);

        let mut stream_epoch = self.clock_ref.subscribe()?;

        let current_epoch = self.clock_ref.current_epoch();

        // Start from the latest settled certificate to define the next expected height
        let latest_settled = self
            .state_store
            .get_latest_settled_certificate_per_network(&self.network_id)?
            .map(|(_network_id, settled)| settled);

        let mut next_expected_height =
            if let Some(SettledCertificate(_, current_height, epoch, _)) = latest_settled {
                debug!("Current network height is {}", current_height);
                if epoch == current_epoch {
                    debug!("Already settled for the epoch {current_epoch}");
                    self.at_capacity_for_epoch = true;
                }

                current_height.next()
            } else {
                debug!("Network never settled any certificate");
                Height::ZERO
            };

        let mut first_run = true;

        loop {
            tokio::select! {
                // TODO (IN ANOTHER PR): move cancellation token to make_progress, have make_progess return ControlFlow?
                _ = cancellation_token.cancelled() => {
                    debug!("Network task for network {} has been cancelled", self.network_id);
                    return Ok(self.network_id);
                }

                result = self.make_progress(&mut stream_epoch, &mut next_expected_height, &mut first_run, &cancellation_token) => {
                    if let Err(error)= result {
                        error!("Error during the certification process: {}", error);

                        match error {
                            Error::InternalError(_) | Error::Storage(_) | Error::PersistenceError { .. } => return Err(error),
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    async fn make_progress(
        &mut self,
        stream_epoch: &mut tokio::sync::broadcast::Receiver<agglayer_clock::Event>,
        next_expected_height: &mut Height,
        first_run: &mut bool,
        cancellation_token: &CancellationToken,
    ) -> Result<(), Error> {
        if *first_run {
            *first_run = false;
        } else {
            tokio::select! {
                event = stream_epoch.recv() => {
                    let network_id = self.network_id;
                    match event {
                        Ok(agglayer_clock::Event::EpochEnded(epoch)) => {
                            info!("Received an epoch event: {}", epoch);

                            let current_epoch = self.clock_ref.current_epoch();
                            if epoch != EpochNumber::ZERO && epoch.next() < current_epoch {
                                warn!("Received an epoch event for epoch {epoch} which is outdated, current epoch is {current_epoch}");

                                return Ok(());
                            }
                            match self.latest_settled {
                                Some(SettledCertificate(_, _, epoch, _)) if epoch == current_epoch => {
                                    warn!("Network {network_id} is at capacity for the epoch {current_epoch}");
                                    return Ok(());
                                },
                                _ => {
                                    self.at_capacity_for_epoch = false;
                                }
                            }
                        }
                        Err(broadcast::error::RecvError::Lagged(num_skipped)) => {
                            warn!("Network {network_id} skipped {num_skipped} epoch ticks");
                            return Ok(());
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            error!("Epoch channel closed for network {network_id}");
                            return Err(Error::InternalError("epoch channel closed".into()));
                        }
                    }
                }
                Some(NewCertificate { certificate_id, height, .. }) = self.certificate_stream.recv(), if !self.at_capacity_for_epoch => {
                    info!(
                        hash = certificate_id.to_string(),
                        "Received a certificate event for {certificate_id} at height {height}"
                    );

                    if matches!(
                        self.latest_settled,
                        Some(SettledCertificate(settled_id, _, _, _)) if settled_id == certificate_id)
                    {
                        return Ok(());
                    }

                    if *next_expected_height != height {
                        warn!(
                            hash = certificate_id.to_string(),
                            "Received a certificate event for the wrong height");

                        return Ok(());
                    }
                }
            }
        }

        // Get the certificate the pending certificate for the network at the height
        let certificate = if let Some(certificate) = self
            .pending_store
            .get_certificate(self.network_id, *next_expected_height)
            .inspect_err(|err| {
                error!(
                    "Cannot fetch pending certificate for {} at height {}: {}",
                    self.network_id, *next_expected_height, err
                )
            })? {
            certificate
        } else {
            debug!(
                "No certificate found for network {} at height {}",
                self.network_id, *next_expected_height
            );
            // There is no certificate to certify at this height for now
            return Ok(());
        };

        let certificate_id = certificate.hash();

        let (sender, mut receiver) = mpsc::channel(1);

        let bridge_exit_hashes = certificate
            .bridge_exits
            .iter()
            .map(|exit| exit.hash())
            .collect::<Vec<Digest>>();
        let task = tokio::spawn(
            CertificateTask::new(
                certificate,
                sender,
                self.state_store.clone(),
                self.pending_store.clone(),
                self.certifier_client.clone(),
                cancellation_token.clone(),
            )?
            .process(),
        );

        loop {
            tokio::select! {
                msg = receiver.recv() => match msg {
                    None => {
                        error!(height = next_expected_height.as_u64(), %certificate_id, "Certificate task channel closed");
                        return Err(Error::InternalError("Certificate task channel closed".into()));
                    }
                    Some(NetworkTaskMessage::GetLocalNetworkStateBeforeHeight { response, .. }) => {
                        let state = self.local_state.clone();
                        response.send(Ok(state)).map_err(|_| {
                            Error::InternalError("Certificate response channel closed".into())
                        })?;
                        continue;
                    }
                    Some(NetworkTaskMessage::CertificateExecuted { new_state, .. }) => {
                        self.pending_state = Some(new_state);
                        continue;
                    }
                    Some(NetworkTaskMessage::CertificateProven { height, .. }) => {
                        if let Err(error) = self
                            .pending_store
                            .set_latest_proven_certificate_per_network(&self.network_id, &height, &certificate_id)
                        {
                            error!(
                                hash = certificate_id.to_string(),
                                "Failed to set the latest proven certificate per network: {:?}", error
                            );
                        }
                        continue;
                    }
                    Some(NetworkTaskMessage::CertificateReadyForSettlement { settlement_submitted_notifier, .. }) => {
                        // For now, the network task directly submits the settlement.
                        // In the future, with aggregation, all this will likely move to a separate epoch packer task.
                        // This is the reason why the certificate task does not directly submit and wait for settlement.
                        let result = self
                            .settlement_client
                            .submit_certificate_settlement(certificate_id)
                            .await
                            .map_err(Into::into);
                        settlement_submitted_notifier
                            .send(result)
                            .map_err(|_| Error::InternalError("Certificate notification channel closed".into()))?;
                        #[cfg(feature = "testutils")]
                        fail::fail_point!("network_task::make_progress::settlement_submitted");
                        continue;
                    }
                    Some(NetworkTaskMessage::CertificateWaitingForSettlement { settlement_tx_hash, settlement_complete_notifier, .. }) => {
                        // See comment on CertificateReadyForSettlement.
                        let result = self
                            .settlement_client
                            .wait_for_settlement(settlement_tx_hash, certificate_id)
                            .await
                            .map_err(Into::into);
                        settlement_complete_notifier
                            .send(result)
                            .map_err(|_| Error::InternalError("Certificate notification channel closed".into()))?;
                        continue;
                    }
                    Some(NetworkTaskMessage::CertificateSettled { settled_certificate, height, .. }) => {
                        self.at_capacity_for_epoch = true;
                        let epoch_number = settled_certificate.2;
                        let certificate_index = settled_certificate.3;
                        self.latest_settled = Some(settled_certificate);
                        next_expected_height.increment();
                        debug!(%certificate_id, "Certification process completed");
                        let Some(new) = self.pending_state.take() else {
                            return Err(Error::InternalError(format!("Missing pending state needed upon settlement, current state: {}", self.local_state.get_roots().display_to_hex() )))
                        };
                        debug!(
                            old_state = self.local_state.get_roots().display_to_hex(),
                            new_state = new.get_roots().display_to_hex(),
                            "Updated the state following certificate settlement",
                        );
                        self.local_state = new;

                        // Store the current state
                        self.state_store
                            .write_local_network_state(
                                &self.network_id,
                                &self.local_state,
                                bridge_exit_hashes.as_slice(),
                            )
                            .map_err(|e| Error::PersistenceError {
                                certificate_id,
                                error: e.to_string(),
                            })?;
                        self.state_store
                            .set_latest_settled_certificate_for_network(
                                &self.network_id,
                                &height,
                                &certificate_id,
                                &epoch_number,
                                &certificate_index,
                            )
                            .map_err(|e| Error::PersistenceError { certificate_id, error: e.to_string() })?;

                        break;
                    }
                    Some(NetworkTaskMessage::CertificateErrored { .. }) => {
                        // The certificate task already logged everything that should be logged.
                        self.pending_state = None;
                        self.at_capacity_for_epoch = false;
                        break;
                    }
                }
            }
        }

        task.await
            .map_err(|e| Error::InternalError(format!("Certificate task panicked: {e}")))?;

        Ok(())
    }
}
