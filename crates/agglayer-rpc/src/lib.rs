use std::sync::Arc;

use agglayer_config::{epoch::BlockClockConfig, Config, Epoch};
use agglayer_contracts::{L1TransactionFetcher, RollupContract};
use agglayer_rate_limiting as rate_limiting;
use agglayer_storage::{
    columns::latest_settled_certificate_per_network::SettledCertificate,
    stores::{
        DebugReader, DebugWriter, PendingCertificateReader, PendingCertificateWriter, StateReader,
        StateWriter,
    },
};
use agglayer_types::{
    Address, Certificate, CertificateHeader, CertificateId, CertificateStatus, EpochConfiguration,
    Height, NetworkId, Signature,
};
use error::SignatureVerificationError;
use tokio::sync::mpsc;
use tracing::{debug, error, info, instrument, warn};

pub use self::error::{CertificateRetrievalError, CertificateSubmissionError};

pub mod error;

/// The RPC agglayer service implementation.
pub struct AgglayerService<L1Rpc, PendingStore, StateStore, DebugStore> {
    certificate_sender: mpsc::Sender<(NetworkId, Height, CertificateId)>,
    pub(crate) pending_store: Arc<PendingStore>,
    pub(crate) state: Arc<StateStore>,
    debug_store: Arc<DebugStore>,
    config: Arc<Config>,
    l1_rpc_provider: Arc<L1Rpc>,
}

impl<L1Rpc, PendingStore, StateStore, DebugStore>
    AgglayerService<L1Rpc, PendingStore, StateStore, DebugStore>
{
    /// Create an instance of the RPC agglayer service.
    pub fn new(
        certificate_sender: mpsc::Sender<(NetworkId, Height, CertificateId)>,
        pending_store: Arc<PendingStore>,
        state: Arc<StateStore>,
        debug_store: Arc<DebugStore>,
        config: Arc<Config>,
        l1_rpc_provider: Arc<L1Rpc>,
    ) -> Self {
        Self {
            certificate_sender,
            pending_store,
            state,
            debug_store,
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

impl<L1Rpc, PendingStore, StateStore, DebugStore> Drop
    for AgglayerService<L1Rpc, PendingStore, StateStore, DebugStore>
{
    fn drop(&mut self) {
        info!("Shutting down the agglayer RPC service");
    }
}

impl<L1Rpc, PendingStore, StateStore, DebugStore>
    AgglayerService<L1Rpc, PendingStore, StateStore, DebugStore>
where
    PendingStore: PendingCertificateReader + 'static,
    StateStore: StateReader + 'static,
    DebugStore: DebugReader + 'static,
    L1Rpc: Send + Sync + 'static,
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
}

impl<L1Rpc, PendingStore, StateStore, DebugStore>
    AgglayerService<L1Rpc, PendingStore, StateStore, DebugStore>
where
    PendingStore: PendingCertificateWriter + PendingCertificateReader + 'static,
    StateStore: StateReader + StateWriter + 'static,
    DebugStore: DebugReader + DebugWriter + 'static,
    L1Rpc: RollupContract + L1TransactionFetcher + 'static,
{
    #[instrument(skip(self, certificate), level = "info")]
    async fn validate_pre_existing_certificate(
        &self,
        certificate: &Certificate,
    ) -> Result<(), CertificateSubmissionError> {
        let new_certificate_id = certificate.hash();
        // Get pre-existing certificate in pending
        if let Some(certificate) = self
            .pending_store
            .get_certificate(certificate.network_id, certificate.height)?
        {
            let pre_existing_certificate_id = certificate.hash();
            warn!(
                pre_existing_certificate_id = pre_existing_certificate_id.to_string(),
                "Certificate already exists in pending store for network {} at height {}",
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
                            "Replacing pending certificate {} that is in error",
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
                            let message = "Unable to replace a pending certificate in error that \
                                           has already been settled";
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
                let message = "Unable to replace a pending certificate that is not in error";
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

        let signer = cert
            .signer()
            .map_err(SignatureVerificationError::from_signer_error)?
            .into_array()
            .into();

        // ECDSA-k256 signature verification works by recovering the public key from the
        // signature, and then checking that it is the expected one.
        if signer != sequencer_address {
            return Err(SignatureVerificationError::InvalidSigner {
                signer,
                trusted_sequencer: sequencer_address,
            });
        }

        Ok(())
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
            (Some(&expected_extra_signer), Some(extra_signature)) => {
                // Retrieve the signer from the extra signature
                let retrieved_extra_signer = certificate
                    .signer_from_signature(extra_signature)
                    .map_err(SignatureVerificationError::from_signer_error)?
                    .into_array()
                    .into();

                // Verify that the signature is performed by the expected authority
                if retrieved_extra_signer != expected_extra_signer {
                    return Err(SignatureVerificationError::InvalidExtraSignature {
                        expected: expected_extra_signer,
                        got: retrieved_extra_signer,
                    });
                }
            }
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

        // Verify the involved signatures
        // TODO: For now both commitments are enforced to be the V2 version.
        // Ideally we want the aggsender to sign the V3 version.
        // Consequently, we'll need to
        //   - return the commitment version of the signed message for both signatures
        //   - verify that they are both considering the exact same message (and so same
        //     commitment version)
        //   - verify that the commitment version is the expected one from the last PP
        //     root in L1 (same as what is done in the witness generation)
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

        self.verify_cert_signature(&certificate)
            .await
            .map_err(|error| {
                error!(?error, "Failed to verify the signature of certificate");
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
