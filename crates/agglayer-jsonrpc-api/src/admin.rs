use std::sync::Arc;

use agglayer_config::Config;
use agglayer_storage::stores::{
    DebugReader, DebugWriter, PendingCertificateReader, PendingCertificateWriter, StateReader,
    StateWriter,
};
use agglayer_types::{
    Certificate, CertificateHeader, CertificateId, CertificateStatus, CertificateStatusError,
    Height, NetworkId,
};
use jsonrpsee::{core::async_trait, proc_macros::rpc, server::ServerBuilder};
use tower_http::{compression::CompressionLayer, cors::CorsLayer};
use tracing::{error, info, instrument, warn};

use super::error::RpcResult;
use crate::{error::Error, rpc_middleware, JsonRpcService};

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

    #[method(name = "forceSetCertificateStatus")]
    async fn force_set_certificate_status(
        &self,
        certificate_id: CertificateId,
        status: CertificateStatus,
    ) -> RpcResult<()>;

    #[method(name = "setLatestPendingCertificate")]
    async fn set_latest_pending_certificate(&self, certificate_id: CertificateId) -> RpcResult<()>;

    #[method(name = "setLatestProvenCertificate")]
    async fn set_latest_proven_certificate(&self, certificate_id: CertificateId) -> RpcResult<()>;

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
pub struct AdminAgglayerImpl<PendingStore, StateStore, DebugStore> {
    pending_store: Arc<PendingStore>,
    state: Arc<StateStore>,
    debug_store: Arc<DebugStore>,
    config: Arc<Config>,
}

impl<PendingStore, StateStore, DebugStore> AdminAgglayerImpl<PendingStore, StateStore, DebugStore> {
    /// Create an instance of the admin RPC agglayer service.
    pub fn new(
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
    pub async fn start(self) -> eyre::Result<axum::Router> {
        // Create the RPC service
        let config = self.config.clone();

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

        let service_builder =
            server_builder.set_rpc_middleware(rpc_middleware::from_config(&config));
        let (stop_handle, server_handle) = jsonrpsee::server::stop_channel();
        std::mem::forget(server_handle);

        let service = self.into_rpc();
        let service = JsonRpcService {
            service: service_builder
                .to_service_builder()
                .build(service, stop_handle),
        };

        Ok(axum::Router::new()
            .route("/", axum::routing::get_service(service.clone()))
            .route("/", axum::routing::post_service(service.clone()))
            .layer(middleware))
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
            Ok(None) => Err(Error::ResourceNotFound(format!(
                "Certificate({certificate_id})"
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
        warn!(
            hash = certificate.hash().to_string(),
            ?certificate,
            "(ADMIN) Forcing push of pending certificate: {}",
            certificate.hash()
        );
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
    async fn force_set_certificate_status(
        &self,
        certificate_id: CertificateId,
        status: CertificateStatus,
    ) -> RpcResult<()> {
        warn!(
            ?certificate_id,
            ?status,
            "(ADMIN) Forcing status of certificate"
        );
        self.state
            .update_certificate_header_status(&certificate_id, &status)
            .map_err(|error| {
                error!(?error, "Failed to update certificate status");
                Error::internal("Unable to update certificate status")
            })?;
        Ok(())
    }

    #[instrument(skip(self, certificate_id), level = "debug")]
    async fn set_latest_pending_certificate(&self, certificate_id: CertificateId) -> RpcResult<()> {
        warn!(
            hash = certificate_id.to_string(),
            "(ADMIN) Setting latest pending certificate: {}", certificate_id
        );
        let certificate = if let Some(certificate) = self
            .state
            .get_certificate_header(&certificate_id)
            .map_err(|error| {
                error!("Failed to get certificate header: {}", error);
                Error::internal("Unable to get certificate header")
            })? {
            certificate
        } else {
            return Err(Error::ResourceNotFound(format!(
                "CertificateHeader({certificate_id})"
            )));
        };

        match self
            .pending_store
            .set_latest_pending_certificate_per_network(
                &certificate.network_id,
                &certificate.height,
                &certificate.certificate_id,
            ) {
            Ok(_) => Ok(()),
            Err(error) => {
                error!("Failed to update latest pending certificate: {}", error);
                Err(Error::internal(
                    "Unable to update latest pending certificate",
                ))
            }
        }
    }

    #[instrument(skip(self, certificate_id), level = "debug")]
    async fn set_latest_proven_certificate(&self, certificate_id: CertificateId) -> RpcResult<()> {
        warn!(
            hash = certificate_id.to_string(),
            "(ADMIN) Setting latest proven certificate: {}", certificate_id
        );
        let certificate = if let Some(certificate) = self
            .state
            .get_certificate_header(&certificate_id)
            .map_err(|error| {
                error!("Failed to get certificate header: {}", error);
                Error::internal("Unable to get certificate header")
            })? {
            certificate
        } else {
            return Err(Error::ResourceNotFound(format!(
                "CertificateHeader({certificate_id})"
            )));
        };

        match self
            .pending_store
            .set_latest_proven_certificate_per_network(
                &certificate.network_id,
                &certificate.height,
                &certificate.certificate_id,
            ) {
            Ok(_) => Ok(()),
            Err(error) => {
                error!("Failed to update latest proven certificate: {}", error);
                Err(Error::internal(
                    "Unable to update latest proven certificate",
                ))
            }
        }
    }

    #[instrument(skip(self), level = "debug")]
    async fn remove_pending_proof(&self, certificate_id: CertificateId) -> RpcResult<()> {
        warn!(
            hash = certificate_id.to_string(),
            "(ADMIN) Removing pending proof: {}", certificate_id
        );

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
        warn!(
            "(ADMIN) Removing pending certificate for network {} at height {}",
            network_id, height
        );
        let certificate_id = if let Some(certificate) = self
            .pending_store
            .get_certificate(network_id, height)
            .map_err(|error| {
                error!("Failed to get pending certificate: {}", error);
                Error::internal("Unable to get pending certificate")
            })? {
            certificate.hash()
        } else {
            return Err(Error::ResourceNotFound(format!(
                "PendingCertificate({network_id:?}, {height:?})",
            )));
        };

        self.pending_store
            .remove_pending_certificate(network_id, height)
            .map_err(|error| {
                error!("Failed to remove pending certificate: {error}");
                Error::internal("Unable to remove pending certificate")
            })?;

        // Update certificate status to InError in the state store
        let error_status = CertificateStatus::error(CertificateStatusError::InternalError(
            "Certificate removed from pending store by administrator".to_string(),
        ));
        self.state
            .update_certificate_header_status(&certificate_id, &error_status)
            .map_err(|error| {
                error!(
                    %certificate_id,
                    ?error,
                    "Failed to update certificate status in the state store on pending removal"
                );
                Error::internal(format!(
                    "Unable to update certificate_id: {certificate_id} status in the state store \
                     on pending removal"
                ))
            })?;

        if remove_proof {
            self.pending_store
                .remove_generated_proof(&certificate_id)
                .map_err(|error| {
                    error!( %certificate_id, ?error, "Failed to remove generated proof");
                    Error::internal(format!(
                        "Failed to remove generated proof for certificate_id: {certificate_id}"
                    ))
                })?;
        }

        Ok(())
    }
}
