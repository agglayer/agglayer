use std::net::SocketAddr;

use agglayer_prover_types::v1::proof_generation_service_server::{
    ProofGenerationService, ProofGenerationServiceServer,
};
use agglayer_types::Proof;
use bincode::Options;
use tonic::transport::Server;
use tracing::error;
use tracing::info;

#[derive(Default)]
pub struct FakeProver {}

impl FakeProver {
    pub async fn spawn_at(
        fake_prover: Self,
        endpoint: SocketAddr,
        cancellation_token: tokio_util::sync::CancellationToken,
    ) -> Result<tokio::task::JoinHandle<Result<(), tonic::transport::Error>>, ()> {
        let svc = ProofGenerationServiceServer::new(fake_prover);

        let (mut health_reporter, health_service) = tonic_health::server::health_reporter();

        health_reporter
            .set_serving::<ProofGenerationServiceServer<FakeProver>>()
            .await;

        health_reporter
            .set_serving::<ProofGenerationServiceServer<FakeProver>>()
            .await;

        let reflection = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(agglayer_prover_types::FILE_DESCRIPTOR_SET)
            .build_v1alpha()
            .expect("Cannot build gRPC because of FILE_DESCRIPTOR_SET error");
        let layer = tower::ServiceBuilder::new().into_inner();

        info!("Starting Agglayer Prover on {}", endpoint);
        let handle = tokio::spawn(async move {
            if let Err(error) = Server::builder()
                .layer(layer)
                .add_service(reflection)
                .add_service(health_service)
                .add_service(svc)
                .serve_with_shutdown(endpoint, cancellation_token.cancelled())
                .await
            {
                error!("Failed to start Agglayer Prover: {}", error);

                return Err(error);
            }

            Ok(())
        });

        Ok(handle)
    }
}

#[tonic::async_trait]
impl ProofGenerationService for FakeProver {
    async fn generate_proof(
        &self,
        _request: tonic::Request<agglayer_prover_types::v1::ProofGenerationRequest>,
    ) -> Result<tonic::Response<agglayer_prover_types::v1::ProofGenerationResponse>, tonic::Status>
    {
        Ok(tonic::Response::new(
            agglayer_prover_types::v1::ProofGenerationResponse {
                proof: agglayer_prover_types::default_bincode_options()
                    .serialize(&Proof::new_for_test())
                    .unwrap(),
            },
        ))
    }
}
