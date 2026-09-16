use std::sync::Arc;

use agglayer_config::{epoch::BlockClockConfig, Config, Epoch};
use agglayer_contracts::{AggchainContract, L1TransactionFetcher, RollupContract};
use agglayer_rate_limiting as rate_limiting;
use agglayer_storage::{
    columns::latest_settled_certificate_per_network::SettledCertificate,
    stores::{
        async_api::AsyncStateReaderExt, DebugReader, DebugWriter, NetworkInfoReader,
        PendingCertificateReader, PendingCertificateWriter, SettlementReader, StateReader,
        StateWriter,
    },
};
use agglayer_types::{
    aggchain_data::MultisigCtx, aggchain_proof::AggchainData, Certificate, CertificateHeader,
    CertificateId, CertificateStatus, ContractCallOutcome, EpochConfiguration, Height, NetworkId,
    NetworkInfo, NetworkStatus,
};
use agglayer_utils::task::spawn_blocking_in_current_span;
use error::SignatureVerificationError;
use tokio::sync::mpsc;
use tracing::{debug, error, info, instrument, warn};

pub use self::error::{CertificateRetrievalError, CertificateSubmissionError, GetNetworkInfoError};

pub mod error;
#[cfg(test)]
mod tests;

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
    StateStore: NetworkInfoReader + StateReader + 'static,
    DebugStore: DebugReader + 'static,
    L1Rpc: Send + Sync + 'static,
{
    fn latest_settled_id_and_height_blocking(
        state: &StateStore,
        network_id: &NetworkId,
    ) -> Result<Option<(CertificateId, Height)>, agglayer_storage::error::Error> {
        Ok(state
            .get_latest_settled_certificate_per_network(network_id)
            .inspect_err(|e| error!("Failed to get latest settled certificate: {e}"))?
            .map(|(_, SettledCertificate(id, height, _, _))| (id, height)))
    }

    fn latest_proven_id_and_height_blocking(
        pending_store: &PendingStore,
        network_id: &NetworkId,
    ) -> Result<Option<(CertificateId, Height)>, agglayer_storage::error::Error> {
        Ok(pending_store
            .get_latest_proven_certificate_per_network(network_id)
            .inspect_err(|e| error!("Failed to get latest proven certificate: {e}"))?
            .map(|(_, height, id)| (id, height)))
    }

    pub async fn get_latest_known_certificate_header(
        &self,
        network_id: NetworkId,
    ) -> Result<Option<CertificateHeader>, CertificateRetrievalError> {
        let pending_store = self.pending_store.clone();
        let state = self.state.clone();

        spawn_blocking_in_current_span(move || {
            Self::get_latest_known_certificate_header_blocking(
                pending_store.as_ref(),
                state.as_ref(),
                network_id,
            )
        })
        .await
        .expect("latest-known-certificate query task panicked")
    }

    fn get_latest_known_certificate_header_blocking(
        pending_store: &PendingStore,
        state: &StateStore,
        network_id: NetworkId,
    ) -> Result<Option<CertificateHeader>, CertificateRetrievalError> {
        debug!(
            "Received request to get the latest known certificate header for rollup {network_id}",
        );

        let settled_certificate_id_and_height =
            Self::latest_settled_id_and_height_blocking(state, &network_id)?;
        let proven_certificate_id_and_height =
            Self::latest_proven_id_and_height_blocking(pending_store, &network_id)?;

        let pending_certificate_id_and_height = pending_store
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
            Some(certificate_id) => {
                Self::fetch_certificate_header_blocking(state, certificate_id).map(Some)
            }
        }
    }

    pub async fn get_latest_settled_certificate_header(
        &self,
        network_id: NetworkId,
    ) -> Result<Option<CertificateHeader>, CertificateRetrievalError> {
        let state = self.state.clone();

        spawn_blocking_in_current_span(move || {
            Self::get_latest_settled_certificate_header_blocking(state.as_ref(), network_id)
        })
        .await
        .expect("latest-settled-certificate query task panicked")
    }

    fn get_latest_settled_certificate_header_blocking(
        state: &StateStore,
        network_id: NetworkId,
    ) -> Result<Option<CertificateHeader>, CertificateRetrievalError> {
        let id = match state
            .get_latest_settled_certificate_per_network(&network_id)
            .inspect_err(|e| error!("Failed to get latest settled certificate id: {e}"))?
        {
            Some((_, SettledCertificate(id, _, _, _))) => id,
            None => return Ok(None),
        };

        Self::fetch_certificate_header_blocking(state, id).map(Some)
    }

    pub async fn get_latest_pending_certificate_header(
        &self,
        network_id: NetworkId,
    ) -> Result<Option<CertificateHeader>, CertificateRetrievalError> {
        let pending_store = self.pending_store.clone();
        let state = self.state.clone();

        spawn_blocking_in_current_span(move || {
            Self::get_latest_pending_certificate_header_blocking(
                pending_store.as_ref(),
                state.as_ref(),
                network_id,
            )
        })
        .await
        .expect("latest-pending-certificate query task panicked")
    }

    fn get_latest_pending_certificate_header_blocking(
        pending_store: &PendingStore,
        state: &StateStore,
        network_id: NetworkId,
    ) -> Result<Option<CertificateHeader>, CertificateRetrievalError> {
        let id = match pending_store
            .get_latest_pending_certificate_for_network(&network_id)
            .inspect_err(|e| error!("Failed to get latest pending certificate id: {e}"))?
        {
            Some((id, _height)) => id,
            None => return Ok(None),
        };

        Self::get_pending_certificate_header_blocking(state, id)
    }

    fn get_pending_certificate_header_blocking(
        state: &StateStore,
        certificate_id: CertificateId,
    ) -> Result<Option<CertificateHeader>, CertificateRetrievalError> {
        Self::fetch_certificate_header_blocking(state, certificate_id).map(|header| {
            match header.status {
                CertificateStatus::Pending
                | CertificateStatus::Proven
                | CertificateStatus::Candidate
                | CertificateStatus::InError { .. } => Some(header),
                CertificateStatus::Settled => None,
            }
        })
    }

    /// Get the certificate header, raising an error if not found.
    pub async fn fetch_certificate_header(
        &self,
        certificate_id: CertificateId,
    ) -> Result<CertificateHeader, CertificateRetrievalError> {
        self.state
            .get_certificate_header_async(certificate_id)
            .await
            .inspect_err(|err| error!("Failed to get certificate header: {err}"))?
            .ok_or(CertificateRetrievalError::NotFound { certificate_id })
    }

    fn fetch_certificate_header_blocking(
        state: &StateStore,
        certificate_id: CertificateId,
    ) -> Result<CertificateHeader, CertificateRetrievalError> {
        state
            .get_certificate_header(&certificate_id)
            .inspect_err(|err| error!("Failed to get certificate header: {err}"))?
            .ok_or(CertificateRetrievalError::NotFound { certificate_id })
    }

    /// Assemble the current information of the specified network from
    /// the data in various sources.
    #[instrument(skip(self))]
    pub async fn get_network_info(
        &self,
        network_id: NetworkId,
    ) -> Result<NetworkInfo, GetNetworkInfoError> {
        let pending_store = self.pending_store.clone();
        let state = self.state.clone();

        spawn_blocking_in_current_span(move || {
            Self::get_network_info_blocking(pending_store.as_ref(), state.as_ref(), network_id)
        })
        .await
        .expect("network-info query task panicked")
    }

    /// Every field comes from storage: no epoch database is opened, no
    /// certificate or proof is decoded, and nothing is written.
    fn get_network_info_blocking(
        pending_store: &PendingStore,
        state: &StateStore,
        network_id: NetworkId,
    ) -> Result<NetworkInfo, GetNetworkInfoError> {
        debug!("Received request to get the network state for rollup {network_id}");

        // Storage owns the complete settlement snapshot, including any legacy
        // fallback. Further point reads here could attach a newer settlement
        // to the claim and aggregates from that snapshot.
        let mut network_info = state.get_network_info(network_id).map_err(|error| {
            error!(
                ?error,
                "Failed to retrieve network info for network {network_id} from the storage"
            );
            GetNetworkInfoError::InternalError {
                network_id,
                source: error.into(),
            }
        })?;

        let latest_pending_certificate = match network_info.latest_pending_certificate_id {
            Some(certificate_id) => {
                Self::get_pending_certificate_header_blocking(state, certificate_id)
            }
            None => Self::get_latest_pending_certificate_header_blocking(
                pending_store,
                state,
                network_id,
            ),
        };

        let latest_pending_certificate = match latest_pending_certificate {
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

        // Certificate-owned pending data always comes from the header,
        // including on cache hits. A cached pointer whose header is
        // already settled is omitted by
        // `get_pending_certificate_header_blocking` and must not leak stale
        // response fields.
        network_info.latest_pending_certificate_id = None;
        network_info.latest_pending_height = None;
        network_info.latest_pending_status = None;
        network_info.latest_pending_error = None;

        if let Some(cert) = latest_pending_certificate {
            network_info.latest_pending_certificate_id = Some(cert.certificate_id);
            network_info.latest_pending_height = Some(cert.height);
            if let CertificateStatus::InError { ref error } = &cert.status {
                network_info.latest_pending_error = Some(*error.clone());
            }

            network_info.latest_pending_status = Some(cert.status);
        }

        let network_is_disabled = state.is_network_disabled(&network_id).map_err(|error| {
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
                // Network is in error if the latest pending certificate is in
                // error
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

impl<L1Rpc, PendingStore, StateStore, DebugStore>
    AgglayerService<L1Rpc, PendingStore, StateStore, DebugStore>
where
    StateStore: StateReader + SettlementReader + 'static,
{
    /// Refuse replacing a certificate whose settlement job may still settle
    /// (or already has settled) this height on L1.
    ///
    /// The settlement tx hash is only recorded on the header after a terminal
    /// success, so an in-flight or spuriously-failed settlement leaves an
    /// `InError` certificate with no hash while its transaction is still live
    /// on L1 (and gets re-broadcast by settlement-service startup recovery).
    /// Replacing the certificate then races the old transaction with the
    /// replacement's: the old one settles first (lower nonce, same wallet) and
    /// L1 diverges from the certificate we track at this height. Only a
    /// terminally reverted settlement job makes replacement safe.
    // TODO: `CertificateSubmissionError` should become `eyre::Error` soon anyway.
    #[allow(clippy::result_large_err)]
    fn ensure_no_live_settlement_job_blocking(
        state: &StateStore,
        network_id: NetworkId,
        height: Height,
        pre_existing_certificate_id: CertificateId,
        replacement_certificate_id: CertificateId,
    ) -> Result<(), CertificateSubmissionError> {
        let Some(settlement_job_id) =
            state.get_certificate_settlement_job_id(&pre_existing_certificate_id)?
        else {
            return Ok(());
        };

        // Replacement is the risky operation here, so the allowing arm is the
        // strict one: only an explicit terminal revert lets it through. Any
        // outcome added to `ContractCallOutcome` in the future must take an
        // explicit stance on replacement to make this match exhaustive again.
        let result = state.get_settlement_job_result(&settlement_job_id)?;
        let message = match result.as_ref().map(|r| &r.contract_call_result.outcome) {
            Some(ContractCallOutcome::Revert) => {
                info!(
                    %pre_existing_certificate_id,
                    %settlement_job_id,
                    "Settlement job of the replaced certificate terminally reverted; replacement \
                     allowed"
                );
                return Ok(());
            }
            None => {
                "Unable to replace a certificate in error whose settlement job is still in flight"
            }
            Some(ContractCallOutcome::Success) => {
                "Unable to replace a certificate in error whose settlement job has already \
                 succeeded"
            }
        };

        warn!(%pre_existing_certificate_id, %settlement_job_id, message);
        Err(
            CertificateSubmissionError::UnableToReplacePendingCertificate {
                reason: message.to_string(),
                height,
                network_id,
                stored_certificate_id: pre_existing_certificate_id,
                replacement_certificate_id,
                source: None,
            },
        )
    }

    // TODO: `CertificateSubmissionError` should become `eyre::Error` soon
    // anyway.
    #[cfg(test)]
    #[allow(clippy::result_large_err)]
    fn ensure_no_live_settlement_job(
        &self,
        certificate: &Certificate,
        pre_existing_certificate_id: CertificateId,
        replacement_certificate_id: CertificateId,
    ) -> Result<(), CertificateSubmissionError> {
        Self::ensure_no_live_settlement_job_blocking(
            self.state.as_ref(),
            certificate.network_id,
            certificate.height,
            pre_existing_certificate_id,
            replacement_certificate_id,
        )
    }
}

impl<L1Rpc, PendingStore, StateStore, DebugStore>
    AgglayerService<L1Rpc, PendingStore, StateStore, DebugStore>
where
    PendingStore: PendingCertificateWriter + PendingCertificateReader + 'static,
    StateStore: StateReader + StateWriter + SettlementReader + 'static,
    DebugStore: DebugReader + DebugWriter + 'static,
    L1Rpc: RollupContract + AggchainContract + L1TransactionFetcher + 'static,
{
    fn get_known_certificate_id_at_height_blocking(
        pending_store: &PendingStore,
        state: &StateStore,
        network_id: NetworkId,
        height: Height,
    ) -> Result<Option<CertificateId>, agglayer_storage::error::Error> {
        // TODO This should be in a database transaction to get a consistent
        // view of the storage.
        if let Some(cert) = pending_store.get_certificate(network_id, height)? {
            return Ok(Some(cert.hash()));
        }
        let certificate_id = state
            .get_certificate_header_by_cursor(network_id, height)?
            .map(|header| header.certificate_id);
        Ok(certificate_id)
    }

    // TODO: `CertificateSubmissionError` should become `eyre::Error` soon
    // anyway.
    #[allow(clippy::result_large_err)]
    fn check_replacement_storage_blocking(
        pending_store: &PendingStore,
        state: &StateStore,
        network_id: NetworkId,
        height: Height,
        new_certificate_id: CertificateId,
    ) -> Result<
        Option<(CertificateId, Option<agglayer_types::SettlementTxHash>)>,
        CertificateSubmissionError,
    > {
        let Some(pre_existing_certificate_id) = Self::get_known_certificate_id_at_height_blocking(
            pending_store,
            state,
            network_id,
            height,
        )?
        else {
            return Ok(None);
        };

        warn!(
            pre_existing_certificate_id = pre_existing_certificate_id.to_string(),
            "Certificate already exists in store for network {} at height {}", network_id, height
        );

        if let Some(CertificateHeader {
            status: CertificateStatus::InError { .. },
            settlement_tx_hash,
            ..
        }) = state.get_certificate_header(&pre_existing_certificate_id)?
        {
            Self::ensure_no_live_settlement_job_blocking(
                state,
                network_id,
                height,
                pre_existing_certificate_id,
                new_certificate_id,
            )?;
            Ok(Some((pre_existing_certificate_id, settlement_tx_hash)))
        } else {
            let message = "Unable to replace a certificate that is not in error";
            info!(%pre_existing_certificate_id, message);

            Err(
                CertificateSubmissionError::UnableToReplacePendingCertificate {
                    reason: message.to_string(),
                    height,
                    network_id,
                    stored_certificate_id: pre_existing_certificate_id,
                    replacement_certificate_id: new_certificate_id,
                    source: None,
                },
            )
        }
    }

    // TODO: `CertificateSubmissionError` should become `eyre::Error` soon
    // anyway.
    #[allow(clippy::result_large_err)]
    #[instrument(skip(self, certificate), level = "info")]
    async fn validate_pre_existing_certificate(
        &self,
        certificate: &Certificate,
    ) -> Result<(), CertificateSubmissionError> {
        let new_certificate_id = certificate.hash();
        let network_id = certificate.network_id;
        let height = certificate.height;
        let pending_store = self.pending_store.clone();
        let state = self.state.clone();
        let pre_existing = spawn_blocking_in_current_span(move || {
            Self::check_replacement_storage_blocking(
                pending_store.as_ref(),
                state.as_ref(),
                network_id,
                height,
                new_certificate_id,
            )
        })
        .await
        .expect("certificate replacement validation task panicked")?;

        if let Some((pre_existing_certificate_id, settlement_tx_hash)) = pre_existing {
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
                        .fetch_transaction_receipt(tx_hash)
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
                            let error =
                                agglayer_contracts::L1RpcError::TransactionNotYetMined(tx_hash);
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
                        let message = "Unable to replace a certificate in error that has already \
                                       been settled";
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

    // TODO: `CertificateSubmissionError` should become `eyre::Error` soon
    // anyway.
    #[allow(clippy::result_large_err)]
    #[instrument(skip(self, certificate), fields(hash, rollup_id = certificate.network_id.to_u32()), level = "info")]
    pub async fn send_certificate(
        &self,
        certificate: Certificate,
    ) -> Result<CertificateId, CertificateSubmissionError> {
        let hash = certificate.hash();
        let hash_string = hash.to_string();
        tracing::Span::current().record("hash", &hash_string);

        info!(
            %hash,
            "Received certificate {hash} for rollup {} at height {}", certificate.network_id.to_u32(), certificate.height
        );
        self.validate_pre_existing_certificate(&certificate).await?;

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

        // Reserve the orchestrator notification slot before writing anything:
        // the blocking task below owns the permit, so once persistence starts,
        // dropping this future can no longer separate the RocksDB writes from
        // the orchestrator notification.
        let notification_permit = self
            .certificate_sender
            .clone()
            .reserve_owned()
            .await
            .map_err(|error| {
                error!("Failed to send certificate: {error}");
                CertificateSubmissionError::OrchestratorNotResponsive
            })?;

        let pending_store = self.pending_store.clone();
        let state = self.state.clone();
        let debug_store = self.debug_store.clone();
        let notification = (certificate.network_id, certificate.height, hash);
        spawn_blocking_in_current_span(move || {
            // TODO: Batch the different queries.
            pending_store
                .insert_pending_certificate(
                    certificate.network_id,
                    certificate.height,
                    &certificate,
                )
                .inspect_err(|e| error!("Failed to insert certificate into pending store: {e}"))?;

            // Inserting a `Pending` header also requests the backup of the
            // newly accepted certificate, so this write must stay ordered
            // after the pending-store insert above: the backup has to capture
            // the certificate body together with the header.
            state
                .insert_certificate_header(&certificate, CertificateStatus::Pending)
                .inspect_err(|e| error!("Failed to insert certificate into state store: {e}"))?;

            debug_store
                .add_certificate(&certificate)
                .inspect_err(|e| error!("Failed to insert certificate into debug store: {e}"))?;

            notification_permit.send(notification);

            Ok::<_, CertificateSubmissionError>(())
        })
        .await
        .expect("certificate persistence task panicked")?;

        Ok(hash)
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TxStatus {
    Done,
    Pending,
}

impl TxStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TxStatus::Done => "done",
            TxStatus::Pending => "pending",
        }
    }
}

impl std::fmt::Display for TxStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
