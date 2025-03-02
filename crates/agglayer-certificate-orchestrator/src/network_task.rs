use std::sync::Arc;

use agglayer_clock::ClockRef;
use agglayer_storage::{
    columns::latest_settled_certificate_per_network::SettledCertificate,
    stores::{PendingCertificateReader, PendingCertificateWriter, StateReader, StateWriter},
};
use agglayer_types::{
    Certificate, CertificateId, CertificateStatus, CertificateStatusError, Height,
    LocalNetworkStateData, NetworkId,
};
use agglayer_types::{CertificateHeader, Digest};
use pessimistic_proof::utils::Hashable as _;
use tokio::sync::{broadcast, mpsc};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};

use crate::{CertificationError, Certifier, CertifierOutput, EpochPacker, Error};

#[cfg(test)]
mod tests;

/// Message to notify the network task that a new certificate has been received.
#[derive(Debug)]
pub(crate) struct NewCertificate {
    pub(crate) certificate_id: CertificateId,
    pub(crate) height: Height,
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
    local_state: LocalNetworkStateData,

    /// The clock reference to subscribe to the epoch events and check for
    /// current epoch.
    clock_ref: ClockRef,
    /// The pending local network state that should be applied on receiving
    /// settlement response.
    pending_state: Option<LocalNetworkStateData>,
    /// The stream of new certificates to certify.
    certificate_stream: mpsc::Receiver<NewCertificate>,
    /// Flag to indicate if the network is at capacity for the current epoch.
    at_capacity_for_epoch: bool,
    /// latest certificate settled
    latest_settled: Option<SettledCertificate>,
}

impl<CertifierClient, SettlementClient, PendingStore, StateStore>
    NetworkTask<CertifierClient, SettlementClient, PendingStore, StateStore>
where
    CertifierClient: Certifier,
    SettlementClient: EpochPacker,
    PendingStore: PendingCertificateReader + PendingCertificateWriter,
    StateStore: StateReader + StateWriter,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pending_store: Arc<PendingStore>,
        state_store: Arc<StateStore>,
        certifier_client: Arc<CertifierClient>,
        settlement_client: Arc<SettlementClient>,
        clock_ref: ClockRef,
        network_id: NetworkId,
        certificate_stream: mpsc::Receiver<NewCertificate>,
    ) -> Result<Self, Error> {
        info!("Creating a new network task for network {}", network_id);

        let local_state = state_store
            .read_local_network_state(network_id)?
            .unwrap_or_default();

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

                current_height + 1
            } else {
                debug!("Network never settled any certificate");
                0
            };

        let mut first_run = true;

        loop {
            tokio::select! {
                _ = cancellation_token.cancelled() => {
                    debug!("Network task for network {} has been cancelled", self.network_id);
                    return Ok(self.network_id);
                }

                result = self.make_progress(&mut stream_epoch, &mut next_expected_height, &mut first_run) => {
                    if let Err(error)= result {
                        error!("Error during the certification process: {}", error);

                        match error {
                            Error::InternalError(_) | Error::Storage(_) => return Err(error),
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
        next_expected_height: &mut u64,
        first_run: &mut bool,
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
                            if epoch != 0 && (epoch + 1) < current_epoch {
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
        };

        // Get the certificate the pending certificate for the network at the height
        let certificate = if let Some(certificate) = self
            .pending_store
            .get_certificate(self.network_id, *next_expected_height)?
        {
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
        let header =
            if let Some(header) = self.state_store.get_certificate_header(&certificate_id)? {
                header
            } else {
                error!(
                    hash = certificate_id.to_string(),
                    "Certificate header not found for {certificate_id}"
                );

                return Ok(());
            };
        match header.status {
            CertificateStatus::Pending => self.handle_pending(header, next_expected_height).await,

            // If the certificate is already proven, it means that the
            // certification process has been completed but the settlement didn't happened.
            // It also means that the proof exists and thus we should redo the native
            // execution to update the local state.
            CertificateStatus::Proven => {
                // Redo native execution to get the new_state

                warn!(
                    hash = certificate_id.to_string(),
                    "Certificate {certificate_id} is already proven but we do not have the \
                     new_state anymore...reproving",
                );

                self.state_store.update_certificate_header_status(
                    &certificate_id,
                    &CertificateStatus::Pending,
                )?;
                self.pending_store.remove_generated_proof(&certificate_id)?;

                self.handle_pending(header, next_expected_height).await
            }

            // If the certificate is candidate, it means that the certification process has
            // finished, the settlement process has been initialized but not finished.
            // It also means that the proof exists and has been sent to L1 but we do not have the
            // update to the local state. We should redo the native execution to update the local
            // state.
            CertificateStatus::Candidate => {
                self.handle_candidate(header, &certificate, next_expected_height)
                    .await
            }
            CertificateStatus::InError { error } => {
                warn!(
                    hash = certificate_id.to_string(),
                    "Certificate {certificate_id} is in error: {}", error
                );

                Ok(())
            }
            CertificateStatus::Settled => {
                warn!(
                    hash = certificate_id.to_string(),
                    "Certificate {certificate_id} is already settled while trying to certify the \
                     certificate for network {} at height {}",
                    self.network_id,
                    *next_expected_height - 1
                );

                Ok(())
            }
        }
    }

    async fn handle_candidate(
        &mut self,
        header: CertificateHeader,
        certificate: &Certificate,
        next_expected_height: &mut u64,
    ) -> Result<(), Error> {
        let certificate_id = header.certificate_id;
        let hash = certificate_id.to_string();

        if let Some(tx_hash) = header.settlement_tx_hash {
            let eth_tx_hash: ethers::types::H256 = tx_hash.0.into();

            // Check L1 transaction
            if self
                .settlement_client
                .transaction_exists(eth_tx_hash)
                .await
                .map_err(|error| {
                    error!(hash, "Error while fetching the transaction: {}", error);

                    error
                })?
            {
                let mut new_state = self.local_state.clone();
                let (_multi_batch_header, _initial_state) = self
                    .certifier_client
                    .witness_execution(certificate, &mut new_state)
                    .await
                    .map_err(|error| {
                        error!(hash, "Error while witnessing the execution: {}", error);
                        error
                    })?;

                self.pending_state = Some(new_state);

                if let Err(error) = self
                    .settlement_client
                    .recover_settlement(eth_tx_hash, certificate_id, self.network_id, header.height)
                    .await
                {
                    error!(hash, "Error while recovering the transaction: {}", error);
                } else {
                    info!(
                        hash,
                        "Recovered the transaction for certificate {}", certificate_id
                    );

                    self.handle_certificate_settlement(certificate)?;
                    *next_expected_height += 1;
                }
            } else {
                // Transaction not found, back to pending

                warn!(
                    hash,
                    "Transaction {} not found for certificate {}", eth_tx_hash, hash
                );

                self.state_store.update_certificate_header_status(
                    &certificate_id,
                    &CertificateStatus::Pending,
                )?;
                self.pending_store.remove_generated_proof(&certificate_id)?;
            }
        } else {
            // Candidate but no tx_hash, cert should be in error
            warn!(
                "Certificate {} is in candidate but has no settlement tx hash",
                hash
            );

            self.state_store.update_certificate_header_status(
                &certificate_id,
                &CertificateStatus::InError {
                    error: CertificateStatusError::SettlementError(
                        "Inconsistent transaction state".to_string(),
                    ),
                },
            )?;
        }

        Ok(())
    }

    async fn handle_pending(
        &mut self,
        header: CertificateHeader,
        next_expected_height: &mut u64,
    ) -> Result<(), Error> {
        let certificate_id = header.certificate_id;
        let height = header.height;

        info!(
            hash = certificate_id.to_string(),
            "Certifying the certificate {certificate_id} for network {} at height {}",
            self.network_id,
            height
        );

        match self
            .certifier_client
            .certify(self.local_state.clone(), self.network_id, height)
            .await
        {
            Ok(CertifierOutput {
                height,
                certificate,
                new_state,
                ..
            }) => {
                debug!(
                    hash = certificate_id.to_string(),
                    "Proof certification completed for {certificate_id} for network {}",
                    self.network_id
                );
                match self
                    .on_proven_certificate(height, certificate, new_state)
                    .await
                {
                    Err(error) => {
                        error!(
                            hash = certificate_id.to_string(),
                            "Error during the certification process of {certificate_id} for \
                             network {}: {:?}",
                            self.network_id,
                            error
                        );
                        self.pending_state = None;
                        self.at_capacity_for_epoch = false;
                    }
                    Ok(settled) => {
                        self.latest_settled = Some(settled);
                        *next_expected_height += 1;
                        self.at_capacity_for_epoch = true;

                        debug!(
                            hash = certificate_id.to_string(),
                            "Certification process completed for {certificate_id} for network {}",
                            self.network_id
                        );
                    }
                }
                Ok(())
            }

            Err(error) => {
                warn!(
                    hash = certificate_id.to_string(),
                    "Error during certification process of {certificate_id}: {}", error
                );
                let error: CertificateStatusError = match error {
                    CertificationError::CertificateNotFound(_network, _height) => {
                        CertificateStatusError::InternalError(error.to_string())
                    }
                    CertificationError::TrustedSequencerNotFound(network) => {
                        CertificateStatusError::TrustedSequencerNotFound(network)
                    }
                    CertificationError::ProofVerificationFailed { source } => source.into(),
                    CertificationError::L1InfoRootNotFound(_certificate_id, l1_leaf_count) => {
                        CertificateStatusError::L1InfoRootNotFound(l1_leaf_count)
                    }

                    CertificationError::ProverExecutionFailed { source } => {
                        CertificateStatusError::ProofGenerationError {
                            generation_type: agglayer_types::GenerationType::Prover,
                            source,
                        }
                    }
                    CertificationError::NativeExecutionFailed { source } => {
                        CertificateStatusError::ProofGenerationError {
                            generation_type: agglayer_types::GenerationType::Native,
                            source,
                        }
                    }

                    CertificationError::Types { source } => source.into(),

                    CertificationError::Storage(error) => {
                        let error = format!(
                            "Storage error happened in the certification process of \
                             {certificate_id}: {:?}",
                            error
                        );
                        warn!(hash = certificate_id.to_string(), error);

                        CertificateStatusError::InternalError(error)
                    }
                    CertificationError::Serialize { source } => {
                        let error = format!(
                            "Serialization error happened in the certification process of \
                             {certificate_id}: {:?}",
                            source
                        );
                        warn!(hash = certificate_id.to_string(), error);

                        CertificateStatusError::InternalError(error)
                    }
                    CertificationError::Deserialize { source } => {
                        let error = format!(
                            "Deserialization error happened in the certification process of \
                             {certificate_id}: {:?}",
                            source
                        );
                        warn!(hash = certificate_id.to_string(), error);
                        CertificateStatusError::InternalError(error)
                    }
                    CertificationError::InternalError(error) => {
                        let error = format!(
                            "Internal error happened in the certification process of \
                             {certificate_id}: {}",
                            error
                        );
                        warn!(hash = certificate_id.to_string(), error);

                        CertificateStatusError::InternalError(error)
                    }
                };

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
                        "Certificate {certificate_id} in error and failed to update the \
                         certificate header status"
                    );
                }
                Ok(())
            }
        }
    }
}

impl<CertifierClient, SettlementClient, PendingStore, StateStore>
    NetworkTask<CertifierClient, SettlementClient, PendingStore, StateStore>
where
    CertifierClient: Certifier,
    SettlementClient: EpochPacker,
    PendingStore: PendingCertificateReader + PendingCertificateWriter,
    StateStore: StateWriter,
{
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
    async fn on_proven_certificate(
        &mut self,
        height: Height,
        certificate: Certificate,
        new_state: LocalNetworkStateData,
    ) -> Result<SettledCertificate, Error> {
        let certificate_id = certificate.hash();
        if let Err(error) = self
            .pending_store
            .set_latest_proven_certificate_per_network(&self.network_id, &height, &certificate_id)
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

        self.pending_state = Some(new_state);

        let result = self
            .settlement_client
            .settle_certificate(certificate_id)
            .await;

        match result {
            Ok((_, settled_certificate)) => {
                info!(
                    hash = certificate_id.to_string(),
                    "Received a certificate settlement notification"
                );
                self.handle_certificate_settlement(&certificate)?;

                Ok(settled_certificate)
            }
            Err(error) => {
                let error_as_string = error.to_string();
                error!(
                    hash = certificate_id.to_string(),
                    "Failed to settle the certificate: {}", error_as_string
                );

                if self
                    .state_store
                    .update_certificate_header_status(
                        &certificate_id,
                        &CertificateStatus::InError {
                            error: error.into(),
                        },
                    )
                    .is_err()
                {
                    error!(
                        hash = certificate_id.to_string(),
                        "Certificate {certificate_id} in error and failed to update the \
                         certificate header status"
                    );
                }

                Err(Error::SettlementError {
                    certificate_id,
                    error: error_as_string,
                })
            }
        }
    }

    fn handle_certificate_settlement(&mut self, certificate: &Certificate) -> Result<(), Error> {
        let certificate_id = certificate.hash();
        if let Some(new) = self.pending_state.take() {
            debug!(
                "Updated the state for network {} with the new state {} > {}",
                self.network_id,
                self.local_state.get_roots().display_to_hex(),
                new.get_roots().display_to_hex()
            );

            self.local_state = new;

            // Store the current state
            let new_leaves = certificate
                .bridge_exits
                .iter()
                .map(|exit| exit.hash())
                .collect::<Vec<Digest>>();

            self.state_store
                .write_local_network_state(
                    &certificate.network_id,
                    &self.local_state,
                    new_leaves.as_slice(),
                )
                .map_err(|e| Error::PersistenceError {
                    certificate_id,
                    error: e.to_string(),
                })?;

            Ok(())
        } else {
            error!(
                "Missing pending state for network {} needed upon settlement, current state: {}",
                self.network_id,
                self.local_state.get_roots().display_to_hex()
            );

            Err(Error::InternalError("Missing pending state".to_string()))
        }
    }
}
