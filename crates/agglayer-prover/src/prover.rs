use std::sync::Arc;

use agglayer_config::prover::ProverConfig;
use agglayer_prover_types::v1::proof_generation_service_server::ProofGenerationServiceServer;
use anyhow::Result;
use tokio_util::sync::CancellationToken;
use tonic::transport::Server;
use tracing::debug;

use crate::rpc::ProverRPC;

pub(crate) struct Prover {}

#[buildstructor::buildstructor]
impl Prover {
    /// Function that setups and starts the Agglayer Prover.
    ///
    /// The available methods are:
    ///
    /// - `builder`: Creates a new builder instance.
    /// - `config`: Sets the configuration.
    /// - `start`: Starts the Agglayer prover.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The gRPC server failed to start.
    #[builder(entry = "builder", exit = "start", visibility = "pub(crate)")]
    pub(crate) async fn start(
        _config: Arc<ProverConfig>,
        cancellation_token: CancellationToken,
    ) -> Result<Self> {
        let addr = "[::1]:50051".parse().unwrap();
        let rpc = ProverRPC::default();

        let svc = ProofGenerationServiceServer::new(rpc);

        let layer = tower::ServiceBuilder::new().into_inner();

        Server::builder()
            .layer(layer)
            .add_service(svc)
            .serve_with_shutdown(addr, cancellation_token.cancelled())
            .await?;

        Ok(Self {})
    }

    pub(crate) async fn await_shutdown(self) {
        debug!("Node shutdown started.");
        // _ = join!(self.rpc_handle, self.certificate_orchestrator_handle);
        debug!("Node shutdown completed.");
    }
}
