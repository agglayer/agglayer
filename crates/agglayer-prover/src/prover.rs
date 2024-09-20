use std::sync::Arc;

use agglayer_config::prover::ProverConfig;
use agglayer_prover_types::v1::proof_generation_service_server::ProofGenerationServiceServer;
use anyhow::Result;
use tokio::join;
use tokio_util::sync::CancellationToken;
use tonic::transport::Server;
use tracing::{debug, error, info};

use crate::rpc::ProverRPC;

pub(crate) struct Prover {
    handle: tokio::task::JoinHandle<Result<(), tonic::transport::Error>>,
}

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
        config: Arc<ProverConfig>,
        cancellation_token: CancellationToken,
    ) -> Result<Self> {
        let rpc = ProverRPC::default();

        let svc = ProofGenerationServiceServer::new(rpc);

        let (mut health_reporter, health_service) = tonic_health::server::health_reporter();

        health_reporter
            .set_serving::<ProofGenerationServiceServer<ProverRPC>>()
            .await;

        health_reporter
            .set_serving::<ProofGenerationServiceServer<ProverRPC>>()
            .await;

        let reflection = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(agglayer_prover_types::FILE_DESCRIPTOR_SET)
            .build_v1alpha()
            .expect("Cannot build gRPC because of FILE_DESCRIPTOR_SET error");

        let layer = tower::ServiceBuilder::new().into_inner();

        info!("Starting Agglayer Prover on {}", config.grpc_endpoint);
        let handle = tokio::spawn(async move {
            if let Err(error) = Server::builder()
                .layer(layer)
                .add_service(reflection)
                .add_service(health_service)
                .add_service(svc)
                .serve_with_shutdown(config.grpc_endpoint, cancellation_token.cancelled())
                .await
            {
                error!("Failed to start Agglayer Prover: {}", error);

                return Err(error);
            }

            Ok(())
        });

        Ok(Self { handle })
    }

    pub(crate) async fn await_shutdown(self) {
        debug!("Node shutdown started.");
        _ = join!(self.handle);
        debug!("Node shutdown completed.");
    }
}
