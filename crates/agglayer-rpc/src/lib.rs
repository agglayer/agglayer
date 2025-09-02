use std::sync::Arc;

use agglayer_config::{epoch::BlockClockConfig, Config, Epoch};
use agglayer_contracts::{L1TransactionFetcher, RollupContract};
use agglayer_rate_limiting as rate_limiting;
use agglayer_storage::{
    columns::latest_settled_certificate_per_network::SettledCertificate,
    stores::{
        DebugReader, DebugWriter, EpochStoreReader, PendingCertificateReader,
        PendingCertificateWriter, StateReader, StateWriter,
    },
};
use agglayer_types::{
    aggchain_data::MultisigCtx, aggchain_proof::AggchainData, Address, Certificate,
    CertificateHeader, CertificateId, CertificateStatus, EpochConfiguration, Height, NetworkId,
    Signature,
};
use error::SignatureVerificationError;
use tokio::sync::mpsc;
use tracing::{debug, error, info, instrument, warn};

pub use self::error::{
    CertificateRetrievalError, CertificateSubmissionError, GetNetworkStateError,
};
use crate::{
    error::ProofRetrievalError,
    network_state::{NetworkState, NetworkType},
};

pub mod error;

pub mod network_state;

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
    StateStore: StateReader + 'static,
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
    /// Note: This includes pending certificates, proven certificates
    /// and settled certificates. If no certificate is found, return None.
    pub fn get_latest_available_certificate_for_network(
        &self,
        network_id: NetworkId,
    ) -> Result<Option<Certificate>, CertificateRetrievalError> {
        debug!("Received request to get the latest available certificate for rollup {network_id}");

        let latest_certificate_header = self
            .get_latest_known_certificate_header(network_id)
            .map_err(
                |error| CertificateRetrievalError::UnknownLatestCertificateHeader {
                    network_id,
                    source: Box::new(error),
                },
            )?;

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
                        return Err(CertificateRetrievalError::CertificateIdHashMismatch {
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
                            debug!("Found certificate {} in epoch store", certificate_id);
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
                Err(CertificateRetrievalError::NotFound { certificate_id })
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
        match self.pending_store.get_proof(certificate_id) {
            Ok(Some(proof)) => Ok(Some(proof)),
            Ok(None) => {
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
            Err(error) => {
                error!(
                    ?error,
                    "Failed to get proof for certificate {certificate_id} from pending store",
                );
                Err(ProofRetrievalError::Storage(error))
            }
        }
    }

    /// Assemble the current state of the specified network from
    /// the data in various sources.
    pub fn get_network_state(
        &self,
        network_id: NetworkId,
    ) -> Result<NetworkState, GetNetworkStateError> {
        debug!("Received request to get the network state for rollup {network_id}");

        // Get the latest settled certificate for the network
        let latest_settled_certificate = self
            .get_latest_settled_certificate_header(network_id)
            .inspect_err(|error| {
                warn!(?error, "Failed to get latest settled certificate");
            })
            .unwrap_or_default();

        let latest_pending_certificate = self
            .get_latest_pending_certificate_header(network_id)
            .inspect_err(|error| {
                warn!(?error, "Failed to get latest pending certificate");
            })
            .unwrap_or_default();

        // Determine network type from the latest available certificate
        let network_type = match self.get_latest_available_certificate_for_network(network_id) {
            Ok(Some(certificate)) => {
                // Determine network type based on aggchain_data variant
                match certificate.aggchain_data {
                    agglayer_types::aggchain_proof::AggchainData::ECDSA { .. } => {
                        Ok(NetworkType::Ecdsa)
                    }
                    agglayer_types::aggchain_proof::AggchainData::Generic { .. } => {
                        Ok(NetworkType::Generic)
                    }
                    agglayer_types::aggchain_proof::AggchainData::MultisigOnly { .. } => {
                        Ok(NetworkType::MultisigOnly)
                    }
                    agglayer_types::aggchain_proof::AggchainData::MultisigAndAggchainProof {
                        ..
                    } => Ok(NetworkType::MultisigAndAggchainProof),
                }
            }
            Ok(None) => Err(GetNetworkStateError::UnknownNetworkType { network_id }),
            Err(error) => {
                error!(?error, "Unable to determine network type");
                Err(GetNetworkStateError::UnknownNetworkType { network_id })
            }
        }?;

        // TODO: Define network status. Could represent the healthiness of the network
        // in regard to the agglayer-node. We could have multiple kind of status
        // that could represent a network sending too many unprovable certs,
        // or even a network that didn't settle for N epochs and such (optional).
        let network_status = network_state::NetworkStatus::Active;

        // Extract settled certificate data
        let settled_height = latest_settled_certificate.as_ref().map(|cert| cert.height);
        let settled_certificate_id = latest_settled_certificate
            .as_ref()
            .map(|cert| cert.certificate_id);

        // Extract settled_pp_root from the settled certificate's proof public values
        let settled_pp_root = latest_settled_certificate.as_ref().and_then(|cert| {
            // Get the proof for the settled certificate
            self.get_proof(cert.certificate_id)
                .inspect_err(|error| {
                    error!(
                        ?error,
                        "get network status: failed to get proof for settled certificate"
                    );
                })
                .ok()
                .flatten()
                .and_then(|proof| {
                    // Deserialize the proof's public values to get PessimisticProofOutput
                    let agglayer_types::Proof::SP1(sp1_proof) = proof;
                    pessimistic_proof::PessimisticProofOutput::bincode_codec()
                        .deserialize::<pessimistic_proof::PessimisticProofOutput>(
                            sp1_proof.public_values.as_slice(),
                        )
                        .inspect_err(|error| {
                            error!(
                                ?error,
                                "get network status: failed to deserialize pessimistic proof \
                                 output"
                            );
                        })
                        .ok()
                        .map(|output| output.new_pessimistic_root)
                })
        });

        let settled_ler = latest_settled_certificate
            .as_ref()
            .map(|cert| cert.new_local_exit_root);

        let settled_let_leaf_count = self
            .state
            .read_local_network_state(network_id)
            .inspect_err(|error| {
                error!(
                    ?error,
                    "Failed to get local network state for network {network_id}"
                );
            })
            .ok()
            .flatten()
            .map(|local_network_state| {
                // We return the leaf count of the latest local exit tree
                local_network_state.exit_tree.leaf_count as u64
            });

        let latest_pending_height = latest_pending_certificate
            .as_ref()
            .map(|cert| cert.height.as_u64());

        let latest_pending_status = latest_pending_certificate
            .as_ref()
            .map(|cert| cert.status.clone());

        // Get pending certificate error if exists
        let latest_pending_error = latest_pending_certificate
            .as_ref()
            .and_then(|cert| match &cert.status {
                agglayer_types::CertificateStatus::InError { error } => Some(*error.clone()),
                _ => None,
            });

        // Get epoch with latest settlement from settled certificate header
        let latest_epoch_with_settlement = latest_settled_certificate
            .as_ref()
            .and_then(|cert| cert.epoch_number.map(|num| num.as_u64()));

        // TODO: implement settled claim retrieval
        let settled_claim = None;

        Ok(NetworkState {
            network_status,
            network_type,
            network_id,
            settled_height,
            settled_certificate_id,
            settled_pp_root,
            settled_ler,
            settled_let_leaf_count,
            settled_claim,
            latest_pending_height,
            latest_pending_status,
            latest_pending_error,
            latest_epoch_with_settlement,
        })
    }

    pub fn get_current_epoch(&self) -> Option<agglayer_types::EpochNumber> {
        None // TODO: implement
    }
}

impl<L1Rpc, PendingStore, StateStore, DebugStore, EpochsStore>
    AgglayerService<L1Rpc, PendingStore, StateStore, DebugStore, EpochsStore>
where
    PendingStore: PendingCertificateWriter + PendingCertificateReader + 'static,
    StateStore: StateReader + StateWriter + 'static,
    DebugStore: DebugReader + DebugWriter + 'static,
    L1Rpc: RollupContract + L1TransactionFetcher + 'static,
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
                "Certificate already exists in store for network {} at height {}",
                certificate.network_id,
                certificate.height
            );
            if let Some(CertificateHeader {
                status: CertificateStatus::InError { .. },
                settlement_tx_hash,
                ..
            }) = self
                .state
                .get_certificate_header(&pre_existing_certificate_id)?
            {
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
    #[instrument(skip(self), level = "debug")]
    pub(crate) async fn verify_cert_signature(
        &self,
        cert: &Certificate,
    ) -> Result<(), SignatureVerificationError> {
        // Verify any signature related data, fetch L1 context when needed.
        let sequencer_address = self
            .l1_rpc_provider
            .get_trusted_sequencer_address(
                cert.network_id.to_u32(),
                self.config.proof_signers.clone(),
            )
            .await
            .map_err(|_| {
                SignatureVerificationError::UnableToRetrieveTrustedSequencerAddress(cert.network_id)
            })?;

        let multisig_ctx = MultisigCtx {
            signers: Default::default(), // TODO: to fetch from L1
            threshold: 1,                // TODO: to fetch from L1
            prehash: cert.signature_commitment_values().multisig_commitment(),
        };

        match &cert.aggchain_data {
            AggchainData::ECDSA { signature } => {
                cert.verify_legacy_ecdsa(sequencer_address, signature)
            }
            AggchainData::Generic { signature, .. } => {
                cert.verify_aggchain_proof_signature(sequencer_address, signature)
            }
            AggchainData::MultisigOnly(multisig) => {
                cert.verify_multisig(multisig.into(), multisig_ctx)
            }
            AggchainData::MultisigAndAggchainProof { multisig, .. } => {
                cert.verify_multisig(multisig.into(), multisig_ctx)
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
            %hash,
            "Received certificate {hash} for rollup {} at height {}", certificate.network_id.to_u32(), certificate.height
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
