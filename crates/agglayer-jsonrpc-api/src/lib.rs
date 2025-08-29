use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use agglayer_contracts::{L1TransactionFetcher, RollupContract};
use agglayer_storage::stores::{
    DebugReader, DebugWriter, PendingCertificateReader, PendingCertificateWriter, StateReader,
    StateWriter,
};
use agglayer_types::{
    Certificate, CertificateHeader, CertificateId, EpochConfiguration, NetworkId, NetworkStatus,
};
use alloy::{primitives::B256, providers::Provider};
use error::{Error, RpcResult};
use futures::FutureExt;
use hyper::StatusCode;
use jsonrpsee::{
    core::async_trait,
    proc_macros::rpc,
    server::{HttpBody, PingConfig, ServerBuilder},
};
use tower_http::{compression::CompressionLayer, cors::CorsLayer};
use tracing::info;
use unified_bridge::GlobalIndex;

use crate::{service::AgglayerService, signed_tx::SignedTx};

mod error;
pub mod kernel;
mod rpc_middleware;
pub mod service;
mod signed_tx;
mod zkevm_node_client;

#[cfg(test)]
pub mod tests;

#[cfg(any(test, feature = "testutils"))]
pub mod testutils;

pub mod admin;

#[rpc(server, namespace = "interop")]
trait Agglayer {
    #[method(name = "sendTx")]
    async fn send_tx(&self, tx: SignedTx) -> RpcResult<B256>;

    #[method(name = "getTxStatus")]
    async fn get_tx_status(&self, hash: B256) -> RpcResult<TxStatus>;

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

    #[method(name = "GetNetworkState")]
    async fn get_network_state(&self, network_id: NetworkId) -> RpcResult<NetworkStatus>;
}

/// The RPC agglayer service implementation.
pub struct AgglayerImpl<V0Rpc, Rpc, PendingStore, StateStore, DebugStore> {
    service: Arc<AgglayerService<V0Rpc>>,
    pub(crate) rpc_service:
        Arc<agglayer_rpc::AgglayerService<Rpc, PendingStore, StateStore, DebugStore>>,
}

impl<V0Rpc, Rpc, PendingStore, StateStore, DebugStore>
    AgglayerImpl<V0Rpc, Rpc, PendingStore, StateStore, DebugStore>
{
    /// Create an instance of the RPC agglayer service.
    pub fn new(
        service: Arc<AgglayerService<V0Rpc>>,
        rpc_service: Arc<agglayer_rpc::AgglayerService<Rpc, PendingStore, StateStore, DebugStore>>,
    ) -> Self {
        Self {
            service,
            rpc_service,
        }
    }
}

impl<V0Rpc, Rpc, PendingStore, StateStore, DebugStore> Drop
    for AgglayerImpl<V0Rpc, Rpc, PendingStore, StateStore, DebugStore>
{
    fn drop(&mut self) {
        info!("Shutting down the agglayer JsonRPC server");
    }
}

impl<V0Rpc, Rpc, PendingStore, StateStore, DebugStore>
    AgglayerImpl<V0Rpc, Rpc, PendingStore, StateStore, DebugStore>
where
    V0Rpc: Provider + Clone + 'static,
    Rpc: RollupContract + L1TransactionFetcher + 'static + Send + Sync,
    PendingStore: PendingCertificateWriter + PendingCertificateReader + 'static,
    StateStore: StateReader + StateWriter + 'static,
    DebugStore: DebugReader + DebugWriter + 'static,
{
    pub async fn start(self) -> anyhow::Result<axum::Router> {
        let config = self.rpc_service.config();

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
            .layer(cors);

        let service_builder =
            server_builder.set_rpc_middleware(rpc_middleware::from_config(config));

        let (stop_handle, server_handle) = jsonrpsee::server::stop_channel();
        // Server handle isn't used as we're relying on axum to manage the server
        // lifecycle.
        std::mem::forget(server_handle);

        let service = self.into_rpc();
        let service = JsonRpcService {
            service: service_builder
                .to_service_builder()
                .build(service, stop_handle),
        };

        Ok(axum::Router::new()
            .route("/", axum::routing::post_service(service.clone()))
            .route("/", axum::routing::get_service(service.clone()))
            .route("/json-rpc", axum::routing::post_service(service.clone()))
            .route("/json-rpc", axum::routing::get_service(service.clone()))
            .layer(middleware))
    }
}

#[async_trait]
impl<V0Rpc, Rpc, PendingStore, StateStore, DebugStore> AgglayerServer
    for AgglayerImpl<V0Rpc, Rpc, PendingStore, StateStore, DebugStore>
where
    V0Rpc: Provider + Clone + 'static,
    Rpc: RollupContract + L1TransactionFetcher + 'static + Send + Sync,
    PendingStore: PendingCertificateWriter + PendingCertificateReader + 'static,
    StateStore: StateReader + StateWriter + 'static,
    DebugStore: DebugReader + DebugWriter + 'static,
{
    async fn send_tx(&self, tx: SignedTx) -> RpcResult<B256> {
        Ok(self.service.send_tx(tx).await?)
    }

    async fn get_tx_status(&self, hash: B256) -> RpcResult<TxStatus> {
        Ok(self.service.get_tx_status(hash).await?.to_string())
    }

    async fn send_certificate(&self, certificate: Certificate) -> RpcResult<CertificateId> {
        // NOTE: Extra certificate signature is not supported on the json rpc api
        let extra_signature = None;

        Ok(self
            .rpc_service
            .send_certificate(certificate, extra_signature)
            .await?)
    }

    async fn get_certificate_header(
        &self,
        certificate_id: CertificateId,
    ) -> RpcResult<CertificateHeader> {
        Ok(self.rpc_service.fetch_certificate_header(certificate_id)?)
    }

    async fn get_epoch_configuration(&self) -> RpcResult<EpochConfiguration> {
        Ok(self.rpc_service.get_epoch_configuration().ok_or_else(|| {
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
            .rpc_service
            .get_latest_known_certificate_header(network_id)?;
        Ok(header)
    }

    async fn get_latest_settled_certificate_header(
        &self,
        network_id: NetworkId,
    ) -> RpcResult<Option<CertificateHeader>> {
        let header = self
            .rpc_service
            .get_latest_settled_certificate_header(network_id)?;
        Ok(header)
    }

    async fn get_latest_pending_certificate_header(
        &self,
        network_id: NetworkId,
    ) -> RpcResult<Option<CertificateHeader>> {
        let header = self
            .rpc_service
            .get_latest_pending_certificate_header(network_id)?;
        Ok(header)
    }

    async fn get_network_state(&self, network_id: NetworkId) -> RpcResult<NetworkStatus> {
        // Gather all the network status information needed to build the response

        // Get the latest settled certificate for the network
        let latest_settled_certificate = self
            .rpc_service
            .get_latest_settled_certificate_header(network_id)
            .map_err(|error| {
                tracing::error!(?error, "Failed to get latest settled certificate");
                Error::internal("Failed to get latest settled certificate")
            })?;

        // Get the latest pending certificate for the network
        let latest_pending_certificate = self
            .rpc_service
            .get_latest_pending_certificate_header(network_id)
            .map_err(|error| {
                tracing::error!(?error, "Failed to get latest pending certificate");
                Error::internal("Failed to get latest pending certificate")
            })?;

        // Determine network type from the latest available certificate
        let network_type = match self
            .rpc_service
            .get_latest_available_certificate_for_network(network_id)
        {
            Ok(Some(certificate)) => {
                // Determine network type based on aggchain_data variant
                match certificate.aggchain_data {
                    agglayer_types::aggchain_proof::AggchainData::ECDSA { .. } => "ECDSA",
                    agglayer_types::aggchain_proof::AggchainData::Generic { .. } => "Generic",
                }
            }
            Ok(None) => {
                // No certificate found, use default/unknown type
                tracing::warn!(
                    "No certificate found for network {}, using default network type",
                    network_id
                );
                "Unknown"
            }
            Err(error) => {
                // Error retrieving certificate, log and use default
                tracing::warn!(
                    ?error,
                    "Failed to get certificate for network {}, using default network type",
                    network_id
                );
                "Unknown"
            }
        };

        // TODO: Define network status. Could represent the healthiness of the network
        // in regard to the agglayer-node. We could have multiple kind of status
        // that could represent a network sending too many unprovable certs,
        // or even a network that didn't settle for N epochs and such (optional).
        let network_status = "TBD";

        // Extract settled certificate data
        let (settled_height, settled_cert_id, _settled_epoch) = latest_settled_certificate
            .as_ref()
            .map(|cert| (cert.height, cert.certificate_id, cert.epoch_number))
            .unwrap_or((
                agglayer_types::Height::from(0u64),
                agglayer_types::CertificateId::default(),
                None,
            ));

        // Get pending certificate error if exists
        let pending_error = latest_pending_certificate
            .as_ref()
            .and_then(|cert| match &cert.status {
                agglayer_types::CertificateStatus::InError { error } => Some(error.to_string()),
                _ => None,
            })
            .unwrap_or_default();

        // Get epoch with latest settlement from settled certificate header
        let latest_epoch_with_settlement = latest_settled_certificate
            .as_ref()
            .and_then(|cert| cert.epoch_number)
            .map(|epoch| epoch.as_u64())
            .unwrap_or(0);

        // Extract settled_pp_root from the settled certificate's proof public values
        let settled_pp_root = latest_settled_certificate.as_ref().and_then(|cert| {
            // Get the proof for the settled certificate
            self.rpc_service
                .get_proof(cert.certificate_id)
                .inspect_err(|error| {
                    tracing::error!(
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
                            tracing::error!(
                                ?error,
                                "get network status: failed to deserialize pessimistic proof \
                                 output"
                            );
                        })
                        .ok()
                        .map(|output| output.new_pessimistic_root)
                })
        });

        let settled_let_leaf_count = self
            .rpc_service
            .get_local_network_state(network_id)
            .map_err(|error| {
                tracing::error!(?error, "Failed to get latest network local state");
                Error::internal("Failed to get latest network local state")
            })?
            .map(|local_network_state| {
                // We return the leaf count of the latest local exit tree
                local_network_state.exit_tree.leaf_count as u64
            })
            .unwrap_or_else(|| {
                // If no local state is found, we assume 0 leaves
                tracing::warn!("No local network state found, assuming 0 leaves");
                0
            });

        let network_status = NetworkStatus {
            network_status: network_status.to_string(),
            network_type: network_type.to_string(),
            network_id,
            settled_height,
            settled_certificate_id: settled_cert_id,
            // Extract actual data from settled certificate when available
            settled_pp_root: settled_pp_root.unwrap_or_default(),
            settled_ler: latest_settled_certificate
                .as_ref()
                .map(|cert| cert.new_local_exit_root)
                .unwrap_or_default(),
            settled_let_leaf_count,
            settled_claim: agglayer_types::SettledClaim {
                global_index: GlobalIndex::new(network_id, 0),
                bridge_exit_hash: agglayer_types::Digest::default(),
            },
            latest_pending_height: latest_pending_certificate
                .as_ref()
                .map(|cert| cert.height.as_u64())
                .unwrap_or(0),
            latest_pending_status: latest_pending_certificate
                .as_ref()
                .map(|cert| format!("{}", cert.status))
                .unwrap_or_else(|| "Unknown".to_string()),
            latest_pending_error: pending_error,
            latest_epoch_with_settlement,
        };

        Ok(network_status)
    }
}

type TxStatus = String;

#[derive(Clone)]
struct JsonRpcService<S> {
    service: S,
}

impl<Service, Body> tower::Service<axum::http::Request<Body>> for JsonRpcService<Service>
where
    Service: tower::Service<
        axum::http::Request<Body>,
        Error = jsonrpsee::core::BoxError,
        Response = axum::http::Response<HttpBody>,
        Future: Send + 'static,
    >,
{
    type Response = axum::http::Response<HttpBody>;
    type Error = std::convert::Infallible;

    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service
            .poll_ready(cx)
            // We can assume that the underlying service is always ready.
            // [`jsonrpsee::server::TowerService`] is always ready.
            .map_err(|_| unreachable!("Underlying jsonrpsee service should not return an error"))
    }

    fn call(&mut self, req: axum::http::Request<Body>) -> Self::Future {
        self.service
            .call(req)
            .map(|result| match result {
                Ok(response) => Ok(response),
                Err(error) => Ok(axum::http::Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(HttpBody::from(error.to_string()))
                    // We can unwrap here because we know the status code and body are valid.
                    .unwrap()),
            })
            .boxed()
    }
}
