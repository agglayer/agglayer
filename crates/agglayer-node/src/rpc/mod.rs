use std::sync::Arc;

use agglayer_config::epoch::BlockClockConfig;
use agglayer_config::Config;
use agglayer_config::Epoch;
use agglayer_storage::columns::latest_settled_certificate_per_network::SettledCertificate;
use agglayer_storage::stores::DebugReader;
use agglayer_storage::stores::DebugWriter;
use agglayer_storage::stores::PendingCertificateReader;
use agglayer_storage::stores::PendingCertificateWriter;
use agglayer_storage::stores::StateReader;
use agglayer_storage::stores::StateWriter;
use agglayer_telemetry::KeyValue;
use agglayer_types::CertificateStatus;
use agglayer_types::EpochConfiguration;
use agglayer_types::{Certificate, CertificateHeader, CertificateId, Height, NetworkId};
use ethers::types::TransactionReceipt;
use ethers::{
    contract::{ContractError, ContractRevert},
    providers::Middleware,
    types::H256,
};
use futures::TryFutureExt;
use jsonrpsee::{
    core::async_trait,
    proc_macros::rpc,
    server::{middleware::http::ProxyGetRequestLayer, PingConfig, ServerBuilder, ServerHandle},
};
use tokio::{sync::mpsc, try_join};
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tracing::trace;
use tracing::warn;
use tracing::{debug, error, info, instrument};

use crate::{
    kernel::Kernel,
    rpc::error::{Error, RpcResult, StatusError},
    signed_tx::SignedTx,
};

mod error;
mod rpc_middleware;

#[cfg(test)]
mod tests;

pub(crate) mod admin;

#[rpc(server, namespace = "interop")]
trait Agglayer {
    #[method(name = "sendTx")]
    async fn send_tx(&self, tx: SignedTx) -> RpcResult<H256>;

    #[method(name = "getTxStatus")]
    async fn get_tx_status(&self, hash: H256) -> RpcResult<TxStatus>;

    #[method(name = "sendCertificate")]
    async fn send_certificate(&self, certificate: Certificate) -> RpcResult<CertificateId>;

    #[method(name = "getCertificateHeader")]
    async fn get_certificate_header(
        &self,
        certificate_id: CertificateId,
    ) -> RpcResult<CertificateHeader>;

    #[method(name = "getEpochConfiguration")]
    async fn get_epoch_configuration(&self) -> RpcResult<EpochConfiguration>;

    #[method(name = "getLatestKnownCertificateHeader")]
    async fn get_latest_known_certificate_header(
        &self,
        network_id: NetworkId,
    ) -> RpcResult<Option<CertificateHeader>>;

    #[method(name = "getLatestSettledCertificateHeader")]
    async fn get_latest_settled_certificate_header(
        &self,
        network_id: NetworkId,
    ) -> RpcResult<Option<CertificateHeader>>;

    #[method(name = "getLatestPendingCertificateHeader")]
    async fn get_latest_pending_certificate_header(
        &self,
        network_id: NetworkId,
    ) -> RpcResult<Option<CertificateHeader>>;
}

/// The RPC agglayer service implementation.
pub(crate) struct AgglayerImpl<Rpc, PendingStore, StateStore, DebugStore> {
    kernel: Kernel<Rpc>,
    certificate_sender: mpsc::Sender<(NetworkId, Height, CertificateId)>,
    pending_store: Arc<PendingStore>,
    state: Arc<StateStore>,
    debug_store: Arc<DebugStore>,
    config: Arc<Config>,
}

impl<Rpc, PendingStore, StateStore, DebugStore>
    AgglayerImpl<Rpc, PendingStore, StateStore, DebugStore>
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
}

impl<Rpc, PendingStore, StateStore, DebugStore> Drop
    for AgglayerImpl<Rpc, PendingStore, StateStore, DebugStore>
{
    fn drop(&mut self) {
        info!("Shutting down the agglayer service");
    }
}

impl<Rpc, PendingStore, StateStore, DebugStore>
    AgglayerImpl<Rpc, PendingStore, StateStore, DebugStore>
where
    Rpc: Middleware + 'static,
    PendingStore: PendingCertificateWriter + PendingCertificateReader + 'static,
    StateStore: StateReader + StateWriter + 'static,
    DebugStore: DebugReader + DebugWriter + 'static,
{
    pub(crate) async fn start(self) -> anyhow::Result<ServerHandle> {
        // Create the RPC service
        let config = self.config.clone();
        let mut service = self.into_rpc();

        // Register the system_health method to serve health checks.
        service.register_method(
            "system_health",
            |_, _, _| serde_json::json!({ "health": true }),
        )?;

        // Create the RPC server.
        let mut server_builder = ServerBuilder::new()
            // Set the maximum request body size. The default is 10MB.
            .max_request_body_size(config.rpc.max_request_body_size)
            // Set the maximum response body size. The default is 10MB.
            .max_response_body_size(config.rpc.max_response_body_size)
            // Set the maximum number of connections. The default is 100.
            .max_connections(config.rpc.max_connections)
            // Set the batch request limit. The default is unlimited.
            .set_batch_request_config(match config.rpc.batch_request_limit {
                None => jsonrpsee::server::BatchRequestConfig::Unlimited,
                Some(0) => jsonrpsee::server::BatchRequestConfig::Disabled,
                Some(n) => jsonrpsee::server::BatchRequestConfig::Limit(n),
            });

        // Enable WebSocket ping/pong with the configured interval.
        // By default, pings are disabled.
        if let Some(duration) = config.rpc.ping_interval {
            server_builder =
                server_builder.enable_ws_ping(PingConfig::default().ping_interval(duration));
        }

        // Create a CORS middleware to allow cross-origin requests.
        let cors = CorsLayer::new()
            .allow_methods([
                hyper::Method::POST,
                hyper::Method::GET,
                hyper::Method::OPTIONS,
            ])
            .allow_origin(tower_http::cors::Any)
            .allow_headers([hyper::header::CONTENT_TYPE]);

        // Create a middleware stack with the CORS middleware and a proxy layer for
        // health checks.
        let middleware = tower::ServiceBuilder::new()
            .layer(CompressionLayer::new())
            .layer(ProxyGetRequestLayer::new("/health", "system_health")?)
            .layer(cors);

        let addr = config.rpc_addr();

        let server = server_builder
            .set_http_middleware(middleware)
            .set_rpc_middleware(rpc_middleware::from_config(&config))
            .build(addr)
            .await?;

        info!("Listening on {addr}");

        Ok(server.start(service))
    }
    #[instrument(skip(self, certificate), level = "info")]
    async fn validate_pre_existing_certificate(
        &self,
        certificate: &Certificate,
    ) -> Result<(), Error> {
        // Get pre-existing certificate in pending
        if let Some(certificate) = self
            .pending_store
            .get_certificate(certificate.network_id, certificate.height)
            .map_err(|e| {
                error!("Failed to communicate with pending store: {e}");
                Error::internal(e.to_string())
            })?
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
                .get_certificate_header(&pre_existing_certificate_id)
                .map_err(|e| {
                    error!("Failed to communicate with state store: {e}");
                    Error::internal(e.to_string())
                })?
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
                            .kernel
                            .check_tx_status(H256::from_slice(tx_hash.as_ref()))
                            .await
                            .map_err(|e| {
                                error!("Failed to check transaction status: {e}");
                                Error::internal(e.to_string())
                            })?;

                        if matches!(l1_transaction, Some(TransactionReceipt { status: Some(status), .. }) if status.as_u64() == 0)
                        {
                            info!(
                                %pre_existing_certificate_id,
                                %tx_hash,
                                ?l1_transaction,
                                "Replacing pending certificate in error that has already been settled, but transaction recript status is in failure"
                            );
                        } else {
                            let message = "Unable to replace a pending certificate in error that \
                                           has already been settled";
                            warn!(%pre_existing_certificate_id, %tx_hash, ?l1_transaction, message);

                            return Err(Error::invalid_argument(message));
                        }
                    }
                }
            } else {
                let message = "Unable to replace a pending certificate that is not in error";
                info!(%pre_existing_certificate_id, message);

                return Err(Error::invalid_argument(message));
            }
        }

        Ok(())
    }
}

#[async_trait]
impl<Rpc, PendingStore, StateStore, DebugStore> AgglayerServer
    for AgglayerImpl<Rpc, PendingStore, StateStore, DebugStore>
where
    Rpc: Middleware + 'static,
    PendingStore: PendingCertificateWriter + PendingCertificateReader + 'static,
    StateStore: StateReader + StateWriter + 'static,
    DebugStore: DebugReader + DebugWriter + 'static,
{
    #[instrument(skip(self, tx), fields(hash, rollup_id = tx.tx.rollup_id), level = "info")]
    async fn send_tx(&self, tx: SignedTx) -> RpcResult<H256> {
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

        if !self.kernel.check_rollup_registered(tx.tx.rollup_id) {
            // Return an invalid params error if the rollup is not registered.
            return Err(Error::rollup_not_registered(tx.tx.rollup_id));
        }

        self.kernel.verify_tx_signature(&tx).await.map_err(|e| {
            error!(error = %e, hash, "Failed to verify the signature of transaction {hash}: {e}");
            Error::signature_mismatch(e)
        })?;

        agglayer_telemetry::VERIFY_SIGNATURE.add(1, metrics_attrs_tx);

        // Reserve a rate limiting slot.
        let guard = self
            .kernel
            .rate_limiter()
            .reserve_send_tx(tx.tx.rollup_id, tokio::time::Instant::now())?;

        agglayer_telemetry::CHECK_TX.add(1, metrics_attrs);

        // Run all the verification checks in parallel.
        try_join!(
            self.kernel
                .verify_proof_eth_call(&tx)
                .map_err(|e| {
                    let zkevm_error = decode_contract_error::<
                        _,
                        agglayer_contracts::polygon_zk_evm::PolygonZkEvmErrors,
                    >(&e);

                    let rollup_error = decode_contract_error::<
                        _,
                        agglayer_contracts::polygon_rollup_manager::PolygonRollupManagerErrors,
                    >(&e);

                    let error = match (zkevm_error, rollup_error) {
                        (Some(zkevm_error), _) => zkevm_error,
                        (_, Some(rollup_error)) => rollup_error,
                        (_, _) => e.to_string(),
                    };

                    error!(
                        error_code = %e,
                        error,
                        hash,
                        "Failed to dry-run the verify_batches_trusted_aggregator for transaction \
                         {hash}: {error}"
                    );
                    Error::dry_run(error)
                })
                .map_ok(|_| {
                    agglayer_telemetry::EXECUTE.add(1, metrics_attrs);
                }),
            self.kernel
                .verify_proof_zkevm_node(&tx)
                .map_err(|e| {
                    error!(
                        error = %e,
                        hash,
                        "Failed to verify the batch local_exit_root and state_root of transaction \
                         {hash}: {e}"
                    );
                    Error::root_verification(e)
                })
                .map_ok(|_| {
                    agglayer_telemetry::VERIFY_ZKP.add(1, metrics_attrs);
                })
        )?;

        // Settle the proof on-chain and return the transaction hash.
        let receipt = self.kernel.settle(&tx, guard).await.map_err(|e| {
            error!(
                error = %e,
                hash,
                "Failed to settle transaction {hash} on L1: {e}"
            );
            Error::settlement(e)
        })?;

        agglayer_telemetry::SETTLE.add(1, metrics_attrs);

        info!(hash, "Successfully settled transaction {hash}");

        Ok(receipt.transaction_hash)
    }

    #[instrument(skip(self), fields(hash = hash.to_string()), level = "info")]
    async fn get_tx_status(&self, hash: H256) -> RpcResult<TxStatus> {
        debug!("Received request to get transaction status for hash {hash}");
        let receipt = self.kernel.check_tx_status(hash).await.map_err(|e| {
            error!("Failed to get transaction status for hash {hash}: {e}");
            StatusError::tx_status(e)
        })?;

        let current_block = self.kernel.current_l1_block_height().await.map_err(|e| {
            error!("Failed to get current L1 block: {e}");
            StatusError::l1_block(e)
        })?;

        let receipt = receipt.ok_or_else(|| StatusError::tx_not_found(hash))?;

        let status = match receipt.block_number {
            Some(block_number) if block_number < current_block => "done",
            Some(_) => "pending",
            None => "not found",
        };
        Ok(status.to_string())
    }

    #[instrument(skip(self, certificate), fields(hash, rollup_id = *certificate.network_id), level = "info")]
    async fn send_certificate(&self, certificate: Certificate) -> RpcResult<CertificateId> {
        let hash = certificate.hash();
        let hash_string = hash.to_string();
        tracing::Span::current().record("hash", &hash_string);

        info!(
            %hash,
            "Received certificate {hash} for rollup {} at height {}", *certificate.network_id, certificate.height
        );

        self.kernel.verify_cert_signature(&certificate).await.map_err(|e| {
            error!(error = %e, hash = hash_string, "Failed to verify the signature of certificate {hash}: {e}");
            Error::signature_mismatch(e)
        })?;

        self.validate_pre_existing_certificate(&certificate).await?;

        // TODO: Batch the different queries.
        // Insert the certificate into the pending store.
        _ = self
            .pending_store
            .insert_pending_certificate(certificate.network_id, certificate.height, &certificate)
            .map_err(|e| {
                error!("Failed to insert certificate into pending store: {e}");
                Error::internal(e.to_string())
            })?;

        // Insert the certificate header into the state store.
        _ = self
            .state
            .insert_certificate_header(&certificate, CertificateStatus::Pending)
            .map_err(|e| {
                error!("Failed to insert certificate into state store: {e}");
                Error::internal(e.to_string())
            })?;

        _ = self.debug_store.add_certificate(&certificate).map_err(|e| {
            error!("Failed to insert certificate into debug store: {e}");
            Error::internal(e.to_string())
        });

        if let Err(error) = self
            .certificate_sender
            .send((
                certificate.network_id,
                certificate.height,
                certificate.hash(),
            ))
            .await
        {
            error!("Failed to send certificate: {error}");

            return Err(Error::send_certificate(error));
        }

        Ok(hash)
    }

    async fn get_certificate_header(
        &self,
        certificate_id: CertificateId,
    ) -> RpcResult<CertificateHeader> {
        trace!("Received request to get certificate header for certificate {certificate_id}");
        match self.state.get_certificate_header(&certificate_id) {
            Ok(Some(header)) => Ok(header),
            Ok(None) => Err(Error::resource_not_found(format!(
                "Certificate({})",
                certificate_id
            ))),
            Err(error) => {
                error!("Failed to get certificate header: {}", error);

                Err(Error::internal("Unable to get certificate header"))
            }
        }
    }

    async fn get_epoch_configuration(&self) -> RpcResult<EpochConfiguration> {
        info!("Received request to get epoch configuration");

        if let Epoch::BlockClock(BlockClockConfig {
            epoch_duration,
            genesis_block,
        }) = self.config.epoch
        {
            Ok(EpochConfiguration {
                epoch_duration: epoch_duration.into(),
                genesis_block,
            })
        } else {
            Err(Error::internal(
                "AggLayer isn't configured with a BlockClock configuration, thus no \
                 EpochConfiguration is available",
            ))
        }
    }

    async fn get_latest_known_certificate_header(
        &self,
        network_id: NetworkId,
    ) -> RpcResult<Option<CertificateHeader>> {
        debug!(
            "Received request to get the latest known certificate header for rollup {}",
            network_id
        );

        let settled_certificate_id_and_height = self
            .state
            .get_latest_settled_certificate_per_network(&network_id)
            .map_err(|e| {
                error!("Failed to get latest settled certificate: {e}");

                Error::internal(e.to_string())
            })?
            .map(|(_, SettledCertificate(id, height, _, _))| (id, height));

        let proven_certificate_id_and_height = self
            .pending_store
            .get_latest_proven_certificate_per_network(&network_id)
            .map_err(|e| {
                error!("Failed to get latest proven certificate: {e}");
                Error::internal(e.to_string())
            })?
            .map(|(_, height, id)| (id, height));

        let pending_certificate_id_and_height = self
            .pending_store
            .get_latest_pending_certificate_for_network(&network_id)
            .map_err(|e| {
                error!("Failed to get latest pending certificate: {e}");
                Error::internal(e.to_string())
            })?;

        let certificate_id = [
            settled_certificate_id_and_height,
            proven_certificate_id_and_height,
            pending_certificate_id_and_height,
        ]
        .into_iter()
        .flatten()
        .max_by(|x, y| x.1.cmp(&y.1))
        .map(|v| v.0);

        if let Some(certificate_id) = certificate_id {
            match self.state.get_certificate_header(&certificate_id) {
                Ok(Some(header)) => Ok(Some(header)),
                Ok(None) => Err(Error::resource_not_found(format!(
                    "Certificate({})",
                    certificate_id
                ))),
                Err(error) => {
                    error!(
                        hash = certificate_id.to_string(),
                        "Failed to get certificate header: {}", error
                    );

                    Err(Error::internal("Unable to get certificate header"))
                }
            }
        } else {
            Ok(None)
        }
    }

    async fn get_latest_settled_certificate_header(
        &self,
        network_id: NetworkId,
    ) -> RpcResult<Option<CertificateHeader>> {
        let id = match self
            .state
            .get_latest_settled_certificate_per_network(&network_id)
        {
            Ok(Some((_, SettledCertificate(id, _, _, _)))) => id,
            Ok(None) => {
                return Ok(None);
            }
            Err(e) => {
                error!("Failed to get latest settled certificate header: {e}");
                return Err(Error::internal(e.to_string()));
            }
        };

        match self.state.get_certificate_header(&id) {
            Ok(Some(header)) => Ok(Some(header)),
            Ok(None) => Err(Error::resource_not_found(format!("Certificate({})", id))),
            Err(e) => {
                error!("Failed to get certificate header: {e}");
                Err(Error::internal(e.to_string()))
            }
        }
    }

    async fn get_latest_pending_certificate_header(
        &self,
        network_id: NetworkId,
    ) -> RpcResult<Option<CertificateHeader>> {
        let id = match self
            .pending_store
            .get_latest_pending_certificate_for_network(&network_id)
        {
            Ok(Some((id, _height))) => id,
            Ok(None) => {
                return Ok(None);
            }
            Err(e) => {
                error!("Failed to get latest settled certificate header: {e}");
                return Err(Error::internal(e.to_string()));
            }
        };

        match self.state.get_certificate_header(&id) {
            Ok(Some(CertificateHeader {
                status: CertificateStatus::Settled,
                ..
            })) => Ok(None),
            Ok(Some(header)) => Ok(Some(header)),
            Ok(None) => Err(Error::resource_not_found(format!("Certificate({})", id))),
            Err(e) => {
                error!("Failed to get certificate header: {e}");
                Err(Error::internal(e.to_string()))
            }
        }
    }
}

fn decode_contract_error<M: Middleware, E: ContractRevert + std::fmt::Debug>(
    e: &ContractError<M>,
) -> Option<String> {
    e.decode_contract_revert::<E>()
        .map(|err| format!("{:?}", err))
}

type TxStatus = String;
