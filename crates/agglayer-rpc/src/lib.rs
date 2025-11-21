use std::sync::Arc;

use agglayer_config::{epoch::BlockClockConfig, Config, Epoch};
use agglayer_contracts::{AggchainContract, L1TransactionFetcher, RollupContract};
use agglayer_primitives::Hashable;
use agglayer_rate_limiting as rate_limiting;
use agglayer_storage::{
    columns::latest_settled_certificate_per_network::SettledCertificate,
    stores::{
        DebugReader, DebugWriter, EpochStoreReader, NetworkInfoReader, PendingCertificateReader,
        PendingCertificateWriter, StateReader, StateWriter,
    },
};
use agglayer_types::{
    aggchain_data::MultisigCtx, aggchain_proof::AggchainData, Address, Certificate,
    CertificateHeader, CertificateId, CertificateStatus, EpochConfiguration, Height, NetworkId,
    NetworkInfo, NetworkStatus, NetworkType, SettledClaim, Signature, U256,
};
use error::SignatureVerificationError;
use tokio::sync::mpsc;
use tracing::{debug, error, info, instrument, warn};

pub use self::error::{CertificateRetrievalError, CertificateSubmissionError, GetNetworkInfoError};
use crate::error::{GetLatestCertificateError, GetLatestSettledClaimError, ProofRetrievalError};

pub mod error;
#[cfg(test)]
mod tests;

/// The RPC agglayer service implementation.
pub struct AgglayerService<L1Rpc, PendingStore, StateStore, DebugStore, EpochsStore> {
    certificate_sender: mpsc::Sender<(NetworkId, Height, CertificateId)>,
    pub(crate) pending_store: Arc<PendingStore>,
    pub(crate) state: Arc<StateStore>,
    debug_store: Arc<DebugStore>,
    epochs_store: Arc<EpochsStore>,
    config: Arc<Config>,
    l1_rpc_provider: Arc<L1Rpc>,
}

impl<L1Rpc, PendingStore, StateStore, DebugStore, EpochsStore>
    AgglayerService<L1Rpc, PendingStore, StateStore, DebugStore, EpochsStore>
{
    /// Create an instance of the RPC agglayer service.
    pub fn new(
        certificate_sender: mpsc::Sender<(NetworkId, Height, CertificateId)>,
        pending_store: Arc<PendingStore>,
        state: Arc<StateStore>,
        debug_store: Arc<DebugStore>,
        epochs_store: Arc<EpochsStore>,
        config: Arc<Config>,
        l1_rpc_provider: Arc<L1Rpc>,
    ) -> Self {
        Self {
            certificate_sender,
            pending_store,
            state,
            debug_store,
            epochs_store,
            config,
            l1_rpc_provider,
        }
    }

    /// Get access to the configuration.
    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn get_epoch_configuration(&self) -> Option<EpochConfiguration> {
        info!("Received request to get epoch configuration");

        if let Epoch::BlockClock(BlockClockConfig {
            epoch_duration,
            genesis_block,
        }) = self.config.epoch
        {
            Some(EpochConfiguration {
                epoch_duration: epoch_duration.into(),
                genesis_block,
            })
        } else {
            None
        }
    }
}

impl<L1Rpc, PendingStore, StateStore, DebugStore, EpochsStore> Drop
    for AgglayerService<L1Rpc, PendingStore, StateStore, DebugStore, EpochsStore>
{
    fn drop(&mut self) {
        info!("Shutting down the agglayer RPC service");
    }
}

impl<L1Rpc, PendingStore, StateStore, DebugStore, EpochsStore>
    AgglayerService<L1Rpc, PendingStore, StateStore, DebugStore, EpochsStore>
where
    PendingStore: PendingCertificateReader + 'static,
    StateStore: NetworkInfoReader + StateReader + 'static,
    DebugStore: DebugReader + 'static,
    L1Rpc: Send + Sync + 'static,
    EpochsStore: EpochStoreReader + 'static,
{
    pub fn get_latest_known_certificate_header(
        &self,
        network_id: NetworkId,
    ) -> Result<Option<CertificateHeader>, CertificateRetrievalError> {
        debug!(
            "Received request to get the latest known certificate header for rollup {network_id}",
        );

        let settled_certificate_id_and_height = self
            .state
            .get_latest_settled_certificate_per_network(&network_id)
            .inspect_err(|e| error!("Failed to get latest settled certificate: {e}"))?
            .map(|(_, SettledCertificate(id, height, _, _))| (id, height));

        let proven_certificate_id_and_height = self
            .pending_store
            .get_latest_proven_certificate_per_network(&network_id)
            .inspect_err(|e| error!("Failed to get latest proven certificate: {e}"))?
            .map(|(_, height, id)| (id, height));

        let pending_certificate_id_and_height = self
            .pending_store
            .get_latest_pending_certificate_for_network(&network_id)
            .inspect_err(|e| error!("Failed to get latest pending certificate: {e}"))?;

        let certificate_id = [
            pending_certificate_id_and_height,
            proven_certificate_id_and_height,
            settled_certificate_id_and_height,
        ]
        .into_iter()
        .flatten()
        .max_by(|x, y| x.1.cmp(&y.1))
        .map(|v| v.0);

        match certificate_id {
            None => Ok(None),
            Some(certificate_id) => self.fetch_certificate_header(certificate_id).map(Some),
        }
    }

    /// Get latest available certificate data for a network.
    /// Note: This includes proven certificates
    /// and settled certificates. If no certificate is found, return None.
    pub fn get_latest_available_certificate_for_network(
        &self,
        network_id: NetworkId,
    ) -> Result<Option<Certificate>, GetLatestCertificateError> {
        debug!("Received request to get the latest available certificate for rollup {network_id}");

        let proven_certificate_id_and_height = self
            .pending_store
            .get_latest_proven_certificate_per_network(&network_id)
            .inspect_err(|e| error!("Failed to get latest proven certificate: {e}"))?
            .map(|(_, height, id)| (id, height));

        let settled_certificate_id_and_height = self
            .state
            .get_latest_settled_certificate_per_network(&network_id)
            .inspect_err(|e| error!("Failed to get latest settled certificate: {e}"))?
            .map(|(_, SettledCertificate(id, height, _, _))| (id, height));

        let certificate_id = std::cmp::max_by_key(
            proven_certificate_id_and_height,
            settled_certificate_id_and_height,
            |v| v.map(|(_, ht)| ht),
        )
        .map(|v| v.0);

        let latest_certificate_header = match certificate_id {
            None => Ok(None),
            Some(certificate_id) => self
                .fetch_certificate_header(certificate_id)
                .map(Some)
                .map_err(|error| {
                    error!(?error, "Failed to get latest known certificate header");
                    GetLatestCertificateError::UnknownLatestCertificateHeader {
                        network_id,
                        source: Box::new(error),
                    }
                }),
        }?;

        match latest_certificate_header {
            None => Ok(None),
            Some(CertificateHeader {
                certificate_id,
                height,
                epoch_number,
                certificate_index,
                ..
            }) => {
                // First try to get the full certificate from pending store
                if let Ok(Some(certificate)) =
                    self.pending_store.get_certificate(network_id, height)
                {
                    // Verify that this is indeed the certificate we're looking for
                    if certificate.hash() == certificate_id {
                        return Ok(Some(certificate));
                    } else {
                        error!(
                            "Pending certificate hash mismatch: expected {}, got {}",
                            certificate_id,
                            certificate.hash()
                        );
                        return Err(GetLatestCertificateError::CertificateIdHashMismatch {
                            expected: certificate_id,
                            got: certificate.hash(),
                        });
                    }
                }

                // If not found in pending store, try to get from epoch store.
                if let (Some(epoch_number), Some(certificate_index)) =
                    (epoch_number, certificate_index)
                {
                    match self
                        .epochs_store
                        .get_certificate(epoch_number, certificate_index)
                    {
                        Ok(Some(certificate)) => {
                            debug!("Found certificate {certificate_id} in epoch store");
                            return Ok(Some(certificate));
                        }
                        _ => {
                            debug!("Certificate {certificate_id} not found in debug store");
                        }
                    }
                }

                warn!(
                    "Certificate {} at height {} not found in any store",
                    certificate_id, height
                );
                Err(GetLatestCertificateError::NotFound { certificate_id })
            }
        }
    }

    pub fn get_latest_settled_certificate_header(
        &self,
        network_id: NetworkId,
    ) -> Result<Option<CertificateHeader>, CertificateRetrievalError> {
        let id = match self
            .state
            .get_latest_settled_certificate_per_network(&network_id)
            .inspect_err(|e| error!("Failed to get latest settled certificate id: {e}"))?
        {
            Some((_, SettledCertificate(id, _, _, _))) => id,
            None => return Ok(None),
        };

        self.fetch_certificate_header(id).map(Some)
    }

    pub fn get_latest_pending_certificate_header(
        &self,
        network_id: NetworkId,
    ) -> Result<Option<CertificateHeader>, CertificateRetrievalError> {
        let id = match self
            .pending_store
            .get_latest_pending_certificate_for_network(&network_id)
            .inspect_err(|e| error!("Failed to get latest pending certificate id: {e}"))?
        {
            Some((id, _height)) => id,
            None => return Ok(None),
        };

        self.fetch_certificate_header(id)
            .map(|header| match header.status {
                CertificateStatus::Pending
                | CertificateStatus::Proven
                | CertificateStatus::Candidate
                | CertificateStatus::InError { .. } => Some(header),
                CertificateStatus::Settled => None,
            })
    }

    /// Get the certificate header, raising an error if not found.
    pub fn fetch_certificate_header(
        &self,
        certificate_id: CertificateId,
    ) -> Result<CertificateHeader, CertificateRetrievalError> {
        self.state
            .get_certificate_header(&certificate_id)
            .inspect_err(|err| error!("Failed to get certificate header: {err}"))?
            .ok_or(CertificateRetrievalError::NotFound { certificate_id })
    }

    /// Get the proof for a certificate by certificate ID
    pub fn get_proof(
        &self,
        certificate_id: CertificateId,
    ) -> Result<Option<agglayer_types::Proof>, ProofRetrievalError> {
        // First try to get the proof from the pending store
        match self
            .pending_store
            .get_proof(certificate_id)
            .map_err(|error| {
                error!(
                    ?error,
                    "Failed to get proof for certificate {certificate_id} from pending store",
                );
                ProofRetrievalError::Storage(error)
            })? {
            Some(proof) => Ok(Some(proof)),
            None => {
                // If not found in pending store, check the epoch store
                // First get the certificate header to obtain epoch_number and certificate_index
                match self.fetch_certificate_header(certificate_id) {
                    Ok(header) => {
                        if let (Some(epoch_number), Some(certificate_index)) =
                            (header.epoch_number, header.certificate_index)
                        {
                            // Call the epoch store's get_proof method with epoch_number and
                            // certificate_index
                            self.epochs_store
                                .get_proof(epoch_number, certificate_index)
                                .map_err(|error| {
                                    error!(
                                        ?error,
                                        "Failed to get proof for certificate {certificate_id} \
                                         from epoch store",
                                    );
                                    ProofRetrievalError::NotFound { certificate_id }
                                })
                        } else {
                            // Certificate doesn't have epoch information, so no proof in epoch
                            // store
                            Ok(None)
                        }
                    }
                    Err(_) => {
                        // Certificate header not found, so no proof in epoch store
                        Ok(None)
                    }
                }
            }
        }
    }

    pub fn get_latest_settled_claim(
        &self,
        network_id: NetworkId,
        settled_height: Height,
    ) -> Result<Option<SettledClaim>, GetLatestSettledClaimError> {
        // Iterate from the given height down to 0
        for current_height in (0..=settled_height.as_u64()).rev().map(Height::from) {
            // Fetch certificate header for the current height
            let header_opt = self
                .state
                .get_certificate_header_by_cursor(network_id, current_height)?;

            let header = match header_opt {
                Some(h) => h,

                None => {
                    // No certificate at this height, return an error indicating inconsistent state
                    error!(
                        "No certificate header found for network {network_id} at height \
                         {current_height}, inconsistent state"
                    );
                    return Err(GetLatestSettledClaimError::InconsistentState {
                        network_id,
                        height: settled_height,
                    });
                }
            };

            // Only proceed if both epoch_number and certificate_index are present
            let (epoch_number, certificate_index) =
                match (header.epoch_number, header.certificate_index) {
                    (Some(epoch), Some(idx)) => (epoch, idx),
                    _ => {
                        // Missing epoch information, return an error indicating inconsistent state
                        error!(
                            "Missing epoch information in certificate header for network \
                             {network_id} at height {current_height}, inconsistent state"
                        );
                        return Err(GetLatestSettledClaimError::InconsistentState {
                            network_id,
                            height: settled_height,
                        });
                    }
                };

            // Fetch the certificate from the epoch store
            let certificate_opt = self
                .epochs_store
                .get_certificate(epoch_number, certificate_index)
                .map_err(GetLatestSettledClaimError::from)?;

            let certificate = match certificate_opt {
                Some(cert) => cert,
                None => {
                    // Settled certificate not found, return an error indicating inconsistent state
                    error!(
                        "Settled certificate not found in epoch store for network {network_id} at \
                         height {current_height}, inconsistent state"
                    );
                    return Err(GetLatestSettledClaimError::InconsistentState {
                        network_id,
                        height: settled_height,
                    });
                }
            };

            // Check for imported bridge exits
            if let Some(last_imported_exit) = certificate.imported_bridge_exits.last() {
                let global_index: U256 = last_imported_exit.global_index.into();
                let bridge_exit_hash = last_imported_exit.bridge_exit.hash();

                return Ok(Some(SettledClaim {
                    global_index: global_index.to_be_bytes().into(),
                    bridge_exit_hash,
                }));
            }
            // Keep iterating downwards if no imported bridge exits found and no
            // error happened
        }

        // No imported bridge exits found in any certificate from `settled_height` down
        // to 0
        Ok(None)
    }

    /// Assemble the current information of the specified network from
    /// the data in various sources.
    #[instrument(skip(self))]
    pub fn get_network_info(
        &self,
        network_id: NetworkId,
    ) -> Result<NetworkInfo, GetNetworkInfoError> {
        debug!("Received request to get the network state for rollup {network_id}");

        let mut network_info = self
            .state
            .get_network_info(network_id)
            .inspect_err(|error| {
                warn!(
                    ?error,
                    "Failed to retrieve network info for network {network_id} from the storage"
                );
            })
            .unwrap_or_else(|_| NetworkInfo::from_network_id(network_id));

        if network_info.settled_certificate_id.is_none() {
            // Get the latest settled certificate for the network
            let latest_settled_certificate =
                match self.get_latest_settled_certificate_header(network_id) {
                    Ok(cert) => cert,
                    Err(CertificateRetrievalError::NotFound { .. }) => {
                        warn!("No settled certificate found for network {network_id}");
                        None
                    }
                    Err(error) => {
                        error!(
                            ?error,
                            "Failed to get latest settled certificate for network {network_id}"
                        );
                        return Err(GetNetworkInfoError::InternalError {
                            network_id,
                            source: error.into(),
                        });
                    }
                };

            latest_settled_certificate.map(|cert| {
                network_info.settled_certificate_id = Some(cert.certificate_id);
                network_info.settled_height = Some(cert.height);
                network_info.settled_ler = Some(cert.new_local_exit_root);
                network_info.latest_epoch_with_settlement =
                    cert.epoch_number.map(|num| num.as_u64());

                if network_info.settled_pp_root.is_none() {
                    // Extract settled_pp_root from the settled certificate's proof public values
                    network_info.settled_pp_root = match self.get_proof(cert.certificate_id) {
                        Ok(Some(agglayer_types::Proof::SP1(sp1_proof))) => {
                            match pessimistic_proof::PessimisticProofOutput::bincode_codec()
                                .deserialize::<pessimistic_proof::PessimisticProofOutput>(
                                sp1_proof.public_values.as_slice(),
                            ) {
                                Ok(output) => Some(output.new_pessimistic_root),
                                Err(error) => {
                                    error!(
                                        ?error,
                                        "get network status: failed to deserialize pessimistic \
                                         proof output"
                                    );
                                    return Err(GetNetworkInfoError::InternalError {
                                        network_id,
                                        source: error.into(),
                                    });
                                }
                            }
                        }
                        Ok(None) => None,
                        Err(ProofRetrievalError::NotFound { .. }) => None,
                        Err(error) => {
                            error!(
                                ?error,
                                "get network status: failed to get proof for settled certificate \
                                 {certificate_id}",
                                certificate_id = cert.certificate_id
                            );
                            return Err(GetNetworkInfoError::InternalError {
                                network_id,
                                source: error.into(),
                            });
                        }
                    };

                    if network_info.settled_let_leaf_count.is_none() {
                        network_info.settled_let_leaf_count =
                            match self.state.read_local_network_state(network_id) {
                                Ok(local_network_state) => {
                                    local_network_state.map(|v| v.exit_tree.leaf_count as u64)
                                }
                                Err(error) => {
                                    error!(
                                        ?error,
                                        "get network status: failed to read local network state \
                                         for network {network_id}"
                                    );
                                    return Err(GetNetworkInfoError::InternalError {
                                        network_id,
                                        source: error.into(),
                                    });
                                }
                            };
                    }

                    if network_info.settled_claim.is_none() && network_info.settled_height.is_some()
                    {
                        // We can unwrap here because we just checked it's Some
                        let height = network_info.settled_height.unwrap();
                        // Get the last settled claim if we have a settled height
                        network_info.settled_claim = self
                            .get_latest_settled_claim(network_id, height)
                            .map_err(|error| {
                                error!(?error, "Failed to get last settled claim");
                                GetNetworkInfoError::InternalError {
                                    network_id,
                                    source: error.into(),
                                }
                            })?;
                    }
                }

                Ok(())
            });
        }

        if network_info.latest_pending_height.is_none() {
            let latest_pending_certificate =
                match self.get_latest_pending_certificate_header(network_id) {
                    Ok(cert) => cert,
                    Err(CertificateRetrievalError::NotFound { .. }) => {
                        info!("No latest pending certificate found for network {network_id}");
                        None
                    }
                    Err(error) => {
                        error!(
                            ?error,
                            "Failed to get latest pending certificate for network {network_id}"
                        );
                        return Err(GetNetworkInfoError::InternalError {
                            network_id,
                            source: error.into(),
                        });
                    }
                };

            if let Some(cert) = latest_pending_certificate {
                network_info.latest_pending_height = Some(cert.height);
                if let CertificateStatus::InError { ref error } = &cert.status {
                    network_info.latest_pending_error = Some(*error.clone());
                }

                network_info.latest_pending_status = Some(cert.status);
            }
        }

        if network_info.network_type == NetworkType::Unspecified {
            // Determine network type from the latest available certificate
            let aggchain_data = match self.get_latest_available_certificate_for_network(network_id)
            {
                Ok(Some(certificate)) => Ok(Some(certificate.aggchain_data)),
                Ok(None) if network_info.latest_pending_height.is_some() => {
                    // If there's no latest available certificate but we have a pending height,
                    // We can unwrap
                    let height = network_info.latest_pending_height.unwrap();
                    self.pending_store
                        .get_certificate(network_id, height)
                        .map_err(|error| {
                            error!(
                                ?error,
                                "Failed to get pending certificate at height {height} for network \
                                 {network_id}"
                            );
                            GetNetworkInfoError::InternalError {
                                network_id,
                                source: error.into(),
                            }
                        })
                        .map(|maybe_cert| maybe_cert.map(|cert| cert.aggchain_data))
                }
                Ok(None) => {
                    // No certificates at all, cannot determine network type
                    warn!(
                        "No certificates found for network {network_id}, cannot determine network \
                         type"
                    );
                    return Err(GetNetworkInfoError::UnknownNetworkType { network_id });
                }
                Err(error) => {
                    error!(?error, "Unable to determine network type");
                    Err(GetNetworkInfoError::InternalError {
                        network_id,
                        source: error.into(),
                    })
                }
            }?;

            if let Some(ref aggchain_data) = aggchain_data {
                network_info.network_type = aggchain_data.into();
            }
        }

        let network_is_disabled = self
            .state
            .is_network_disabled(&network_id)
            .map_err(|error| {
                error!(
                    ?error,
                    "Failed to check if network {network_id} is disabled in storage"
                );
                GetNetworkInfoError::InternalError {
                    network_id,
                    source: error.into(),
                }
            })?;

        match network_info.latest_pending_status {
            _ if network_is_disabled => {
                // If the network is disabled in storage, mark it as disabled
                network_info.network_status = NetworkStatus::Disabled;
            }
            None => {
                // No pending certificate means the network status is unknown
                network_info.network_status = NetworkStatus::Unknown;
            }
            Some(CertificateStatus::InError { .. }) => {
                // Network is in error if the latest pending certificate is in error
                network_info.network_status = NetworkStatus::Error;
            }
            _ => {
                // Otherwise, the network is active
                network_info.network_status = NetworkStatus::Active;
            }
        }

        Ok(network_info)
    }
}

impl<L1Rpc, PendingStore, StateStore, DebugStore, EpochsStore>
    AgglayerService<L1Rpc, PendingStore, StateStore, DebugStore, EpochsStore>
where
    PendingStore: PendingCertificateWriter + PendingCertificateReader + 'static,
    StateStore: StateReader + StateWriter + 'static,
    DebugStore: DebugReader + DebugWriter + 'static,
    L1Rpc: RollupContract + AggchainContract + L1TransactionFetcher + 'static,
{
    fn get_known_certificate_id_at_height(
        &self,
        network_id: NetworkId,
        height: Height,
    ) -> Result<Option<CertificateId>, agglayer_storage::error::Error> {
        // TODO This should be in a database transaction to get a consistent
        // view of the storage.
        if let Some(cert) = self.pending_store.get_certificate(network_id, height)? {
            return Ok(Some(cert.hash()));
        }
        let certificate_id = self
            .state
            .get_certificate_header_by_cursor(network_id, height)?
            .map(|header| header.certificate_id);
        if certificate_id.is_none() {
            debug!(%network_id, %height, "Pre-existing certificate not found");
        }
        Ok(certificate_id)
    }

    #[instrument(skip(self, certificate), level = "info")]
    async fn validate_pre_existing_certificate(
        &self,
        certificate: &Certificate,
    ) -> Result<(), CertificateSubmissionError> {
        let new_certificate_id = certificate.hash();
        // Get pre-existing certificate in pending
        if let Some(pre_existing_certificate_id) =
            self.get_known_certificate_id_at_height(certificate.network_id, certificate.height)?
        {
            warn!(
                pre_existing_certificate_id = pre_existing_certificate_id.to_string(),
                network_id = %certificate.network_id,
                height = %certificate.height,
                "Certificate already exists in store",
            );
            let header = self
                .state
                .get_certificate_header(&pre_existing_certificate_id)?;
            if header.is_some_and(|h| h.status.is_in_error()) {
                let settlement_tx_hash = self
                    .pending_store
                    .get_settlement_tx_hashes_for_certificate(pre_existing_certificate_id)?
                    .hashes()
                    .last()
                    .copied();
                match settlement_tx_hash {
                    None => {
                        info!(
                            "Replacing certificate {} that is in error",
                            pre_existing_certificate_id
                        );
                    }
                    Some(tx_hash) => {
                        let l1_transaction = self
                            .l1_rpc_provider
                            .fetch_transaction_receipt(tx_hash.into())
                            .await
                            .map_err(|error| {
                                warn!(
                                    "Failed to fetch transaction receipt for certificate {}: {}",
                                    pre_existing_certificate_id, error
                                );

                                CertificateSubmissionError::UnableToReplacePendingCertificate {
                                    reason: error.to_string(),
                                    height: certificate.height,
                                    network_id: certificate.network_id,
                                    stored_certificate_id: pre_existing_certificate_id,
                                    replacement_certificate_id: new_certificate_id,
                                    source: Some(error),
                                }
                            })?
                            .ok_or_else(|| {
                                let error = agglayer_contracts::L1RpcError::TransactionNotYetMined(
                                    tx_hash.to_string(),
                                );
                                warn!(
                                    "Failed to fetch transaction receipt for certificate \
                                     {pre_existing_certificate_id}: {error}"
                                );
                                CertificateSubmissionError::UnableToReplacePendingCertificate {
                                    reason: error.to_string(),
                                    height: certificate.height,
                                    network_id: certificate.network_id,
                                    stored_certificate_id: pre_existing_certificate_id,
                                    replacement_certificate_id: new_certificate_id,
                                    source: Some(error),
                                }
                            })?;

                        if !l1_transaction.status() {
                            info!(
                                %pre_existing_certificate_id,
                                %tx_hash,
                                ?l1_transaction,
                                "Replacing pending certificate in error that has already been settled, but transaction receipt status is in failure"
                            );
                        } else {
                            let message = "Unable to replace a certificate in error that has \
                                           already been settled";
                            warn!(%pre_existing_certificate_id, %tx_hash, ?l1_transaction, message);

                            return Err(
                                CertificateSubmissionError::UnableToReplacePendingCertificate {
                                    reason: message.to_string(),
                                    height: certificate.height,
                                    network_id: certificate.network_id,
                                    stored_certificate_id: pre_existing_certificate_id,
                                    replacement_certificate_id: new_certificate_id,
                                    source: None,
                                },
                            );
                        }
                    }
                }
            } else {
                let message = "Unable to replace a certificate that is not in error";
                info!(%pre_existing_certificate_id, message);

                return Err(
                    CertificateSubmissionError::UnableToReplacePendingCertificate {
                        reason: message.to_string(),
                        height: certificate.height,
                        network_id: certificate.network_id,
                        stored_certificate_id: pre_existing_certificate_id,
                        replacement_certificate_id: new_certificate_id,
                        source: None,
                    },
                );
            }
        }

        Ok(())
    }

    /// Verify that the signer of the given [`Certificate`] is the trusted
    /// sequencer for the rollup id it specified.
    #[instrument(skip(self, cert), fields(certificate_id = %cert.hash()), level = "debug")]
    pub(crate) async fn verify_cert_signature(
        &self,
        cert: &Certificate,
    ) -> Result<(), SignatureVerificationError> {
        // Verify any signature related data, fetch L1 context when needed.
        let fetch_sequencer_address = || async {
            self.l1_rpc_provider
                .get_trusted_sequencer_address(
                    cert.network_id.to_u32(),
                    self.config.proof_signers.clone(),
                )
                .await
                .map_err(|_| {
                    SignatureVerificationError::UnableToRetrieveTrustedSequencerAddress(
                        cert.network_id,
                    )
                })
        };

        // Fetching rollup contract address
        let fetch_multisig_context = || async {
            let rollup_address = self
                .l1_rpc_provider
                .get_rollup_contract_address(cert.network_id.into())
                .await
                .map_err(|source| {
                    SignatureVerificationError::UnableToRetrieveRollupContractAddress {
                        source,
                        network_id: cert.network_id,
                    }
                })?;

            let (signers, threshold) = self
                .l1_rpc_provider
                .get_multisig_context(rollup_address)
                .await
                .map_err(
                    |source| SignatureVerificationError::UnableToRetrieveMultisigContext {
                        source,
                        network_id: cert.network_id,
                    },
                )?;

            Ok::<MultisigCtx, SignatureVerificationError>(MultisigCtx {
                signers,
                threshold,
                prehash: cert.signature_commitment_values().multisig_commitment(),
            })
        };

        match &cert.aggchain_data {
            AggchainData::ECDSA { signature } => {
                cert.verify_legacy_ecdsa(fetch_sequencer_address().await?, signature)
            }
            AggchainData::Generic { signature, .. } => {
                cert.verify_aggchain_proof_signature(fetch_sequencer_address().await?, signature)
            }
            AggchainData::MultisigOnly { multisig } => {
                cert.verify_multisig(multisig.into(), fetch_multisig_context().await?)
            }
            AggchainData::MultisigAndAggchainProof { multisig, .. } => {
                cert.verify_multisig(multisig.into(), fetch_multisig_context().await?)
            }
        }
        .map_err(SignatureVerificationError::from_signer_error)
    }

    /// Verify the extra [`Certificate`] signature.
    #[instrument(skip_all, level = "debug")]
    pub(crate) fn verify_extra_cert_signature(
        &self,
        certificate: &Certificate,
        extra_signer: Option<&Address>,
        extra_signature: Option<Signature>,
    ) -> Result<(), SignatureVerificationError> {
        match (extra_signer, extra_signature) {
            // Extra signature expected and provided
            (Some(&expected_extra_signer), Some(extra_signature)) => certificate
                .verify_extra_signature(expected_extra_signer, extra_signature)
                .map_err(SignatureVerificationError::from_signer_error)?,
            // Extra signature is expected but missing
            (Some(&expected_signer), None) => {
                return Err(SignatureVerificationError::MissingExtraSignature {
                    network_id: certificate.network_id,
                    expected_signer,
                });
            }
            // Extra signature provided but not required
            (None, Some(_)) => {
                warn!("Unexpected extra signature provided");
            }
            // No extra signature provided nor required
            (None, None) => {}
        };

        Ok(())
    }

    #[instrument(skip(self, certificate), fields(hash, rollup_id = certificate.network_id.to_u32()), level = "info")]
    pub async fn send_certificate(
        &self,
        certificate: Certificate,
        extra_signature: Option<Signature>,
    ) -> Result<CertificateId, CertificateSubmissionError> {
        let hash = certificate.hash();
        let hash_string = hash.to_string();
        tracing::Span::current().record("hash", &hash_string);

        info!(
            certificate_id = %hash,
            network_id = %certificate.network_id.to_u32(),
            height = %certificate.height,
            "Received certificate"
        );
        self.validate_pre_existing_certificate(&certificate).await?;

        // Verify the extra certificate signature
        self.verify_extra_cert_signature(
            &certificate,
            self.config()
                .extra_certificate_signer
                .get(&certificate.network_id.to_u32()),
            extra_signature,
        )
        .map_err(|error| {
            error!(
                ?error,
                "Failed to verify the extra signature for the certificate"
            );
            CertificateSubmissionError::SignatureError(error)
        })?;

        // Verify the certificate signature
        self.verify_cert_signature(&certificate)
            .await
            .map_err(|error| {
                error!(
                    ?error,
                    "Failed to verify the signature within the certificate"
                );
                CertificateSubmissionError::SignatureError(error)
            })?;

        // TODO: Batch the different queries.
        // Insert the certificate into the pending store.
        self.pending_store
            .insert_pending_certificate(certificate.network_id, certificate.height, &certificate)
            .inspect_err(|e| error!("Failed to insert certificate into pending store: {e}"))?;

        // Insert the certificate header into the state store.
        self.state
            .insert_certificate_header(&certificate, CertificateStatus::Pending)
            .inspect_err(|e| error!("Failed to insert certificate into state store: {e}"))?;

        self.debug_store
            .add_certificate(&certificate)
            .inspect_err(|e| error!("Failed to insert certificate into debug store: {e}"))?;

        self.certificate_sender
            .send((
                certificate.network_id,
                certificate.height,
                certificate.hash(),
            ))
            .await
            .map_err(|error| {
                error!("Failed to send certificate: {error}");
                CertificateSubmissionError::OrchestratorNotResponsive
            })?;

        Ok(hash)
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TxStatus {
    Done,
    Pending,
    NotFound,
}

impl TxStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TxStatus::Done => "done",
            TxStatus::Pending => "pending",
            TxStatus::NotFound => "not found",
        }
    }
}

impl std::fmt::Display for TxStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
