use std::sync::Arc;

use agglayer_config::{epoch::BlockClockConfig, Config, Epoch};
use agglayer_contracts::{AggchainContract, L1TransactionFetcher, RollupContract};
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
use crate::network_state::NetworkState;

pub mod error;

pub mod network_state;

/// The RPC agglayer service implementation.
pub struct AgglayerService<L1Rpc, PendingStore, StateStore, DebugStore, EpochsStore> {
    certificate_sender: mpsc::Sender<(NetworkId, Height, CertificateId)>,
    pub(crate) pending_store: Arc<PendingStore>,
    pub(crate) state: Arc<StateStore>,
    debug_store: Arc<DebugStore>,
    _epochs_store: Arc<EpochsStore>,
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
            _epochs_store: epochs_store,
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

    /// Assemble the current state of the specified network from
    /// the data in various sources.
    pub fn get_network_state(
        &self,
        network_id: NetworkId,
    ) -> Result<NetworkState, GetNetworkStateError> {
        // TODO: Implement the logic to retrieve the actual network state.
        Ok(NetworkState {
            network_status: network_state::NetworkStatus::Active,
            network_type: network_state::NetworkType::Generic,
            network_id,
            settled_height: None,
            settled_certificate_id: None,
            settled_pp_root: None,
            settled_ler: None,
            settled_let_leaf_count: None,
            settled_claim: None,
            latest_pending_height: None,
            latest_pending_status: None,
            latest_pending_error: None,
            latest_epoch_with_settlement: None,
        })
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
            AggchainData::MultisigOnly(multisig) => {
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
