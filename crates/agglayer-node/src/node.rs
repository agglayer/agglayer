use std::{convert::Infallible, num::NonZeroU64, sync::Arc};

use agglayer_aggregator_notifier::{CertifierClient, EpochPackerClient};
use agglayer_certificate_orchestrator::CertificateOrchestrator;
use agglayer_clock::{BlockClock, Clock, TimeClock};
use agglayer_config::{Config, Epoch};
use agglayer_contracts::{
    polygon_rollup_manager::PolygonRollupManager,
    polygon_zkevm_global_exit_root_v2::PolygonZkEVMGlobalExitRootV2, L1RpcClient,
};
use agglayer_signer::ConfiguredSigner;
use agglayer_storage::{
    storage::DB,
    stores::{
        debug::DebugStore, epochs::EpochsStore, pending::PendingStore, state::StateStore,
        PerEpochReader as _,
    },
};
use alloy::providers::WsConnect;
use anyhow::Result;
use ethers::{
    middleware::MiddlewareBuilder,
    providers::{Http, Provider},
    signers::Signer,
};
use http::{Request, Response};
use tokio::{sync::mpsc, task::JoinHandle};
use tokio_util::sync::CancellationToken;
use tonic::{
    body::{boxed, BoxBody},
    server::NamedService,
};
use tower::Service;
use tower::ServiceExt as _;
use tracing::{debug, error, info, warn};

use crate::{
    epoch_synchronizer::EpochSynchronizer, kernel::Kernel, rpc::AgglayerImpl,
    service::AgglayerService,
};

pub(crate) mod api;

pub(crate) struct Node {
    pub(crate) rpc_handle: JoinHandle<()>,
    pub(crate) certificate_orchestrator_handle: JoinHandle<()>,
}

#[buildstructor::buildstructor]
impl Node {
    /// Function that setups and starts the Agglayer node.
    ///
    /// The available methods are:
    ///
    /// - `builder`: Creates a new builder instance.
    /// - `config`: Sets the configuration.
    /// - `start`: Starts the Agglayer node.
    ///
    /// # Examples
    /// ```no_compile
    /// # use std::sync::Arc;
    /// # use agglayer_config::Config;
    /// # use agglayer_node::Node;
    /// # use tokio_util::sync::CancellationToken;
    /// # use anyhow::Result;
    /// #
    /// async fn start_node() -> Result<()> {
    ///    let config: Arc<Config> = Arc::new(Config::default());
    ///
    ///    Node::builder()
    ///      .config(config)
    ///      .cancellation_token(CancellationToken::new())
    ///      .start()
    ///      .await?;
    ///
    ///    Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The L1 node URL is invalid.
    /// - The configured signer is invalid.
    /// - The RPC server failed to start.
    /// - The [`TimeClock`] failed to start.
    #[builder(entry = "builder", exit = "start", visibility = "pub(crate)")]
    pub(crate) async fn start(
        config: Arc<Config>,
        cancellation_token: CancellationToken,
    ) -> Result<Self> {

        // Bind the core to the RPC server.
        let json_rpc_router = AgglayerImpl::new(service).start().await?;

        let mut grpc_router = axum::Router::new();
        grpc_router = add_rpc_service(
            grpc_router,
            agglayer_grpc_api::Server {}.start(config.clone()),
        );
        let (v1, v1alpha) = agglayer_grpc_api::Server::reflection();
        grpc_router = add_rpc_service(grpc_router, v1);
        grpc_router = add_rpc_service(grpc_router, v1alpha);

        let health_router =
            axum::Router::new().route("/health", axum::routing::get(api::rest::health));

        let router = axum::Router::new()
            .merge(health_router)
            .merge(json_rpc_router)
            .nest("/grpc", grpc_router);

        let listener = tokio::net::TcpListener::bind(config.rpc_addr()).await?;
        let api_graceful_shutdown = cancellation_token.clone();
        info!(on = %config.rpc_addr(), "API listening");

        let api_server = axum::serve(listener, router)
            .with_graceful_shutdown(async move { api_graceful_shutdown.cancelled().await });

        let rpc_handle = tokio::spawn(async move {
            _ = api_server.await;
            debug!("Node RPC shutdown requested.");
        });

        let node = Self {
            rpc_handle,
            certificate_orchestrator_handle,
        };

        Ok(node)
    }

    pub(crate) async fn await_shutdown(self) {
        tokio::select! {
            _ = self.rpc_handle => {
            }
            _ = self.certificate_orchestrator_handle => {
            }
        }
        debug!("Node shutdown completed.");
    }
}

fn add_rpc_service<S>(rpc_server: axum::Router, rpc_service: S) -> axum::Router
where
    S: Service<Request<BoxBody>, Response = Response<BoxBody>, Error = Infallible>
        + NamedService
        + Clone
        + Sync
        + Send
        + 'static,
    S::Future: Send + 'static,
    S::Error: Into<anyhow::Error> + Send,
{
    rpc_server.route_service(
        &format!("/{}/{{*rest}}", S::NAME),
        rpc_service.map_request(|r: Request<axum::body::Body>| r.map(boxed)),
    )
}
