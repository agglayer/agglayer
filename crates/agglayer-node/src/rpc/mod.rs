use std::sync::Arc;

use agglayer_storage::stores::{
    DebugReader, DebugWriter, PendingCertificateReader, PendingCertificateWriter, StateReader,
    StateWriter,
};
use agglayer_types::{
    Certificate, CertificateHeader, CertificateId, EpochConfiguration, NetworkId,
};
use ethers::{providers::Middleware, types::H256};
use jsonrpsee::{
    core::async_trait,
    proc_macros::rpc,
    server::{middleware::http::ProxyGetRequestLayer, PingConfig, ServerBuilder, ServerHandle},
};
use tower_http::{compression::CompressionLayer, cors::CorsLayer};
use tracing::info;

use self::error::{Error, RpcResult};
use crate::{service::AgglayerService, signed_tx::SignedTx};

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
    service: Arc<AgglayerService<Rpc, PendingStore, StateStore, DebugStore>>,
}

impl<Rpc, PendingStore, StateStore, DebugStore>
    AgglayerImpl<Rpc, PendingStore, StateStore, DebugStore>
{
    /// Create an instance of the RPC agglayer service.
    pub(crate) fn new(
        service: Arc<AgglayerService<Rpc, PendingStore, StateStore, DebugStore>>,
    ) -> Self {
        Self { service }
    }
}

impl<Rpc, PendingStore, StateStore, DebugStore> Drop
    for AgglayerImpl<Rpc, PendingStore, StateStore, DebugStore>
{
    fn drop(&mut self) {
        info!("Shutting down the agglayer JsonRPC server");
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
        let config = self.service.config();

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
            .set_rpc_middleware(rpc_middleware::from_config(config))
            .build(addr)
            .await?;

        // Create the RPC service
        let mut service = self.into_rpc();

        // Register the system_health method to serve health checks.
        service.register_method(
            "system_health",
            |_, _, _| serde_json::json!({ "health": true }),
        )?;

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
    async fn send_tx(&self, tx: SignedTx) -> RpcResult<H256> {
        Ok(self.service.send_tx(tx).await?)
    }

    async fn get_tx_status(&self, hash: H256) -> RpcResult<TxStatus> {
        Ok(self.service.get_tx_status(hash).await?.to_string())
    }

    async fn send_certificate(&self, certificate: Certificate) -> RpcResult<CertificateId> {
        self.validate_pre_existing_certificate(&certificate).await?;
        Ok(self.service.send_certificate(certificate).await?)
    }

    async fn get_certificate_header(
        &self,
        certificate_id: CertificateId,
    ) -> RpcResult<CertificateHeader> {
        Ok(self.service.get_certificate_header(certificate_id)?)
    }

    async fn get_epoch_configuration(&self) -> RpcResult<EpochConfiguration> {
        Ok(self.service.get_epoch_configuration().ok_or_else(|| {
            Error::internal(
                "AggLayer isn't configured with a BlockClock configuration, thus no \
                 EpochConfiguration is available",
            )
        })?)
    }

    async fn get_latest_known_certificate_header(
        &self,
        network_id: NetworkId,
    ) -> RpcResult<Option<CertificateHeader>> {
        let header = self
            .service
            .get_latest_known_certificate_header(network_id)?;
        Ok(header)
    }

    async fn get_latest_settled_certificate_header(
        &self,
        network_id: NetworkId,
    ) -> RpcResult<Option<CertificateHeader>> {
        let header = self
            .service
            .get_latest_settled_certificate_header(network_id)?;
        Ok(header)
    }

    async fn get_latest_pending_certificate_header(
        &self,
        network_id: NetworkId,
    ) -> RpcResult<Option<CertificateHeader>> {
        let header = self
            .service
            .get_latest_pending_certificate_header(network_id)?;
        Ok(header)
    }
}

type TxStatus = String;
