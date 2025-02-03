use std::sync::Arc;

use agglayer_config::{epoch::BlockClockConfig, Config, Epoch};
use agglayer_storage::{
    columns::latest_settled_certificate_per_network::SettledCertificate,
    stores::{
        DebugReader, DebugWriter, PendingCertificateReader, PendingCertificateWriter, StateReader,
        StateWriter,
    },
};
use agglayer_telemetry::KeyValue;
use agglayer_types::{
    Certificate, CertificateHeader, CertificateId, CertificateStatus, EpochConfiguration, Height,
    NetworkId,
};
use ethers::{providers::Middleware, types::H256};
use futures::future::try_join;
use tokio::sync::mpsc;
use tracing::{debug, error, info, instrument, trace};

pub use self::error::{
    CertificateRetrievalError, CertificateSubmissionError, SendTxError, TxStatusError,
};
use crate::{kernel::Kernel, signed_tx::SignedTx};

pub mod error;

/// The RPC agglayer service implementation.
pub(crate) struct AgglayerService<Rpc, PendingStore, StateStore, DebugStore> {
    kernel: Kernel<Rpc>,
    certificate_sender: mpsc::Sender<(NetworkId, Height, CertificateId)>,
    pending_store: Arc<PendingStore>,
    state: Arc<StateStore>,
    debug_store: Arc<DebugStore>,
    config: Arc<Config>,
}

impl<Rpc, PendingStore, StateStore, DebugStore>
    AgglayerService<Rpc, PendingStore, StateStore, DebugStore>
{
    /// Create an instance of the RPC agglayer service.
    pub(crate) fn new(
        kernel: Kernel<Rpc>,
        certificate_sender: mpsc::Sender<(NetworkId, Height, CertificateId)>,
        pending_store: Arc<PendingStore>,
        state: Arc<StateStore>,
        debug_store: Arc<DebugStore>,
        config: Arc<Config>,
    ) -> Self {
        Self {
            kernel,
            certificate_sender,
            pending_store,
            state,
            debug_store,
            config,
        }
    }

    /// Get access to the configuration.
    pub fn config(&self) -> &Config {
        &self.config
    }
}

impl<Rpc, PendingStore, StateStore, DebugStore> Drop
    for AgglayerService<Rpc, PendingStore, StateStore, DebugStore>
{
    fn drop(&mut self) {
        info!("Shutting down the agglayer service");
    }
}

impl<Rpc, PendingStore, StateStore, DebugStore>
    AgglayerService<Rpc, PendingStore, StateStore, DebugStore>
where
    Rpc: Middleware + 'static,
    PendingStore: PendingCertificateWriter + PendingCertificateReader + 'static,
    StateStore: StateReader + StateWriter + 'static,
    DebugStore: DebugReader + DebugWriter + 'static,
{
    #[instrument(skip(self, tx), fields(hash, rollup_id = tx.tx.rollup_id), level = "info")]
    pub async fn send_tx(&self, tx: SignedTx) -> Result<H256, SendTxError<Rpc>> {
        let hash = format!("{:?}", tx.hash());
        tracing::Span::current().record("hash", &hash);

        info!(
            hash,
            "Received transaction {hash} for rollup {}", tx.tx.rollup_id
        );
        let rollup_id_str = tx.tx.rollup_id.to_string();
        let metrics_attrs_tx = &[
            KeyValue::new("rollup_id", rollup_id_str),
            KeyValue::new("type", "tx"),
        ];
        let metrics_attrs = &metrics_attrs_tx[..1];

        agglayer_telemetry::SEND_TX.add(1, metrics_attrs);

        let rollup_id = tx.tx.rollup_id;
        if !self.kernel.check_rollup_registered(rollup_id) {
            // Return an invalid params error if the rollup is not registered.
            return Err(SendTxError::RollupNotRegistered { rollup_id });
        }

        self.kernel.verify_tx_signature(&tx).await.inspect_err(|e| {
            error!(error = %e, hash, "Failed to verify the signature of transaction {hash}: {e}");
        })?;

        agglayer_telemetry::VERIFY_SIGNATURE.add(1, metrics_attrs_tx);

        // Reserve a rate limiting slot.
        let guard = self
            .kernel
            .rate_limiter()
            .reserve_send_tx(tx.tx.rollup_id, tokio::time::Instant::now())?;

        agglayer_telemetry::CHECK_TX.add(1, metrics_attrs);

        // Run all the verification checks in parallel.
        let _ = try_join(
            async {
                self.kernel
                    .verify_proof_eth_call(&tx)
                    .await
                    .map_err(|e| {
                        let error = SendTxError::dry_run(&e);
                        error!(
                            error_code = %e,
                            error = error.to_string(),
                            hash,
                            "Failed to dry-run the verify_batches_trusted_aggregator for \
                             transaction {hash}: {error}"
                        );
                        error
                    })
                    .inspect(|_| agglayer_telemetry::EXECUTE.add(1, metrics_attrs))
            },
            async {
                self.kernel
                    .verify_proof_zkevm_node(&tx)
                    .await
                    .map_err(|e| {
                        error!(
                            error = %e,
                            hash,
                            "Failed to verify the batch local_exit_root and state_root of \
                             transaction {hash}: {e}"
                        );
                        SendTxError::RootVerification(e)
                    })
                    .inspect(|_| agglayer_telemetry::VERIFY_ZKP.add(1, metrics_attrs))
            },
        )
        .await?;

        // Settle the proof on-chain and return the transaction hash.
        let receipt = self.kernel.settle(&tx, guard).await.inspect_err(|e| {
            error!(
                error = %e,
                hash,
                "Failed to settle transaction {hash} on L1: {e}"
            )
        })?;

        agglayer_telemetry::SETTLE.add(1, metrics_attrs);

        info!(hash, "Successfully settled transaction {hash}");

        Ok(receipt.transaction_hash)
    }

    #[instrument(skip(self), fields(hash = hash.to_string()), level = "info")]
    pub async fn get_tx_status(&self, hash: H256) -> Result<TxStatus, TxStatusError<Rpc>> {
        debug!("Received request to get transaction status for hash {hash}");

        let receipt = self.kernel.check_tx_status(hash).await.map_err(|e| {
            error!("Failed to get transaction status for hash {hash}: {e}");
            TxStatusError::StatusCheck(e)
        })?;

        let current_block = self.kernel.current_l1_block_height().await.map_err(|e| {
            error!("Failed to get current L1 block: {e}");
            TxStatusError::L1BlockRetrieval(e)
        })?;

        let receipt = receipt.ok_or_else(|| TxStatusError::TxNotFound { hash })?;

        let status = match receipt.block_number {
            Some(block_number) if block_number < current_block => TxStatus::Done,
            Some(_) => TxStatus::Pending,
            None => TxStatus::NotFound,
        };
        Ok(status)
    }

    #[instrument(skip(self, certificate), fields(hash, rollup_id = *certificate.network_id), level = "info")]
    pub async fn send_certificate(
        &self,
        certificate: Certificate,
    ) -> Result<CertificateId, CertificateSubmissionError<Rpc>> {
        let hash = certificate.hash();
        let hash_string = hash.to_string();
        tracing::Span::current().record("hash", &hash_string);

        info!(
            %hash,
            "Received certificate {hash} for rollup {} at height {}", *certificate.network_id, certificate.height
        );

        self.kernel.verify_cert_signature(&certificate).await.map_err(|e| {
            error!(error = %e, hash = hash_string, "Failed to verify the signature of certificate {hash}: {e}");
            CertificateSubmissionError::SignatureError(e)
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

    pub fn get_certificate_header(
        &self,
        certificate_id: CertificateId,
    ) -> Result<CertificateHeader, CertificateRetrievalError> {
        trace!("Received request to get certificate header for certificate {certificate_id}");
        self.fetch_certificate_header(certificate_id)
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
            settled_certificate_id_and_height,
            proven_certificate_id_and_height,
            pending_certificate_id_and_height,
        ]
        .into_iter()
        .flatten()
        .max_by(|x, y| x.1.cmp(&y.1))
        .map(|v| v.0);

        certificate_id.map_or(Ok(None), |certificate_id| {
            self.fetch_certificate_header(certificate_id).map(Some)
        })
    }

    pub async fn debug_get_certificate(
        &self,
        certificate_id: CertificateId,
    ) -> Result<(Certificate, Option<CertificateHeader>), CertificateRetrievalError> {
        let cert = self
            .debug_store
            .get_certificate(&certificate_id)
            .inspect_err(|e| error!("Failed to get certificate: {e}"))?
            .ok_or(CertificateRetrievalError::NotFound { certificate_id })?;
        let header = self
            .state
            .get_certificate_header(&certificate_id)
            .inspect_err(|e| error!("Failed to get certificate header: {e}"))?;

        Ok((cert, header))
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
    fn fetch_certificate_header(
        &self,
        certificate_id: CertificateId,
    ) -> Result<CertificateHeader, CertificateRetrievalError> {
        self.state
            .get_certificate_header(&certificate_id)
            .inspect_err(|err| error!("Failed to get certificate header: {err}"))?
            .ok_or(CertificateRetrievalError::NotFound { certificate_id })
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
