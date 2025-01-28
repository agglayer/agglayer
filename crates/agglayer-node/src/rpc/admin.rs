use std::sync::Arc;

use agglayer_config::Config;
use agglayer_storage::stores::{
    DebugReader, DebugWriter, PendingCertificateReader, PendingCertificateWriter, StateReader,
    StateWriter,
};
use agglayer_types::{
    Certificate, CertificateHeader, CertificateId, CertificateStatus, Height, NetworkId,
};
use jsonrpsee::{
    core::async_trait,
    proc_macros::rpc,
    server::{ServerBuilder, ServerHandle},
};
use tower_http::{compression::CompressionLayer, cors::CorsLayer};
use tracing::{error, info, instrument};

use super::error::RpcResult;
use crate::rpc::{error::Error, rpc_middleware};

#[rpc(server, namespace = "admin")]
pub(crate) trait AdminAgglayer {
    #[method(name = "getCertificate")]
    async fn get_certificate(
        &self,
        certificate_id: CertificateId,
    ) -> RpcResult<(Certificate, Option<CertificateHeader>)>;

    #[method(name = "forcePushPendingCertificate")]
    async fn force_push_pending_certificate(
        &self,
        certificate: Certificate,
        status: CertificateStatus,
    ) -> RpcResult<()>;

    #[method(name = "removePendingCertificate")]
    async fn remove_pending_certificate(
        &self,
        network_id: NetworkId,
        height: Height,
        remove_proof: bool,
    ) -> RpcResult<()>;

    #[method(name = "removePendingProof")]
    async fn remove_pending_proof(&self, certificate_id: CertificateId) -> RpcResult<()>;
}

/// The Admin RPC agglayer service implementation.
pub(crate) struct AdminAgglayerImpl<PendingStore, StateStore, DebugStore> {
    pending_store: Arc<PendingStore>,
    state: Arc<StateStore>,
    debug_store: Arc<DebugStore>,
    config: Arc<Config>,
}

impl<PendingStore, StateStore, DebugStore> AdminAgglayerImpl<PendingStore, StateStore, DebugStore> {
    /// Create an instance of the admin RPC agglayer service.
    pub(crate) fn new(
        pending_store: Arc<PendingStore>,
        state: Arc<StateStore>,
        debug_store: Arc<DebugStore>,
        config: Arc<Config>,
    ) -> Self {
        Self {
            pending_store,
            state,
            debug_store,
            config,
        }
    }
}

impl<PendingStore, StateStore, DebugStore> AdminAgglayerImpl<PendingStore, StateStore, DebugStore>
where
    PendingStore: PendingCertificateWriter + PendingCertificateReader + 'static,
    StateStore: StateReader + StateWriter + 'static,
    DebugStore: DebugReader + DebugWriter + 'static,
{
    pub(crate) async fn start(self) -> anyhow::Result<ServerHandle> {
        // Create the RPC service
        let config = self.config.clone();
        let service = self.into_rpc();

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
            server_builder = server_builder
                .enable_ws_ping(jsonrpsee::server::PingConfig::default().ping_interval(duration));
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
            .layer(cors);

        let addr = config.admin_rpc_addr();

        let server = server_builder
            .set_http_middleware(middleware)
            .set_rpc_middleware(rpc_middleware::from_config(&config))
            .build(addr)
            .await?;

        info!("(admin) Listening on {addr}");

        Ok(server.start(service))
    }
}

impl<PendingStore, StateStore, DebugStore> Drop
    for AdminAgglayerImpl<PendingStore, StateStore, DebugStore>
{
    fn drop(&mut self) {
        info!("Shutting down the agglayer service");
    }
}

#[async_trait]
impl<PendingStore, StateStore, DebugStore> AdminAgglayerServer
    for AdminAgglayerImpl<PendingStore, StateStore, DebugStore>
where
    PendingStore: PendingCertificateWriter + PendingCertificateReader + 'static,
    StateStore: StateReader + StateWriter + 'static,
    DebugStore: DebugReader + DebugWriter + 'static,
{
    #[instrument(skip(self), fields(hash), level = "debug")]
    async fn get_certificate(
        &self,
        certificate_id: CertificateId,
    ) -> RpcResult<(Certificate, Option<CertificateHeader>)> {
        match self.debug_store.get_certificate(&certificate_id) {
            Ok(Some(cert)) => match self
                .state
                .get_certificate_header(&certificate_id)
                .map(|header| (cert, header))
            {
                Ok(result) => Ok(result),
                Err(error) => {
                    error!("Failed to get certificate header: {}", error);
                    Err(Error::internal("Unable to get certificate header"))
                }
            },
            Ok(None) => Err(Error::resource_not_found(format!(
                "Certificate({})",
                certificate_id
            ))),
            Err(error) => {
                error!("Failed to get certificate: {}", error);

                Err(Error::internal("Unable to get certificate"))
            }
        }
    }

    #[instrument(skip(self, certificate), level = "debug")]
    async fn force_push_pending_certificate(
        &self,
        certificate: Certificate,
        status: CertificateStatus,
    ) -> RpcResult<()> {
        match self.pending_store.insert_pending_certificate(
            certificate.network_id,
            certificate.height,
            &certificate,
        ) {
            Ok(_) => match self
                .state
                .update_certificate_header_status(&certificate.hash(), &status)
            {
                Ok(_) => Ok(()),
                Err(error) => {
                    error!("Failed to insert certificate header: {}", error);
                    Err(Error::internal("Unable to insert certificate header"))
                }
            },
            Err(error) => {
                error!("Failed to insert pending certificate: {}", error);
                Err(Error::internal("Unable to insert pending certificate"))
            }
        }
    }

    #[instrument(skip(self), level = "debug")]
    async fn remove_pending_proof(&self, certificate_id: CertificateId) -> RpcResult<()> {
        self.pending_store
            .remove_generated_proof(&certificate_id)
            .map_err(|error| {
                error!("Failed to remove generated proof: {}", error);
                Error::internal("Unable to remove generated proof")
            })
    }

    #[instrument(skip(self), level = "debug")]
    async fn remove_pending_certificate(
        &self,
        network_id: NetworkId,
        height: Height,
        remove_proof: bool,
    ) -> RpcResult<()> {
        let certificate_id = if let Some(certificate) = self
            .pending_store
            .get_certificate(network_id, height)
            .map_err(|error| {
                error!("Failed to get pending certificate: {}", error);
                Error::internal("Unable to get pending certificate")
            })? {
            certificate.hash()
        } else {
            return Err(Error::resource_not_found(format!(
                "PendingCertificate({:?}, {:?})",
                network_id, height
            )));
        };

        self.pending_store
            .remove_pending_certificate(network_id, height)
            .map_err(|error| {
                error!("Failed to remove pending certificate: {}", error);
                Error::internal("Unable to remove pending certificate")
            })?;

        if remove_proof {
            self.pending_store
                .remove_generated_proof(&certificate_id)
                .map_err(|error| {
                    error!("Failed to remove certificate header: {}", error);
                    Error::internal("Unable to remove certificate header")
                })?;
        }

        Ok(())
    }
}
