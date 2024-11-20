use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use agglayer_prover_types::v1::proof_generation_service_server::{
    ProofGenerationService, ProofGenerationServiceServer,
};
use agglayer_prover_types::Error;
use agglayer_types::Keccak256Hasher;
use bincode::Options;
use pessimistic_proof::multi_batch_header::MultiBatchHeader;
use pessimistic_proof::LocalNetworkState;
use sp1_sdk::MockProver;
use sp1_sdk::Prover;
use sp1_sdk::SP1Context;
use sp1_sdk::SP1ProofKind;
use tonic::codec::CompressionEncoding;
use tonic::transport::Server;
use tracing::info;
use tracing::warn;
use tracing::{debug, error};

use crate::executor::Request;
use crate::executor::ELF;

pub struct FakeProver {
    prover: Arc<MockProver>,
    proving_key: sp1_sdk::SP1ProvingKey,
}

impl Default for FakeProver {
    fn default() -> Self {
        let prover = MockProver::new();
        let (proving_key, _) = prover.setup(ELF);
        Self {
            proving_key,
            prover: Arc::new(prover),
        }
    }
}

impl FakeProver {
    pub async fn spawn_at(
        fake_prover: Self,
        endpoint: SocketAddr,
        cancellation_token: tokio_util::sync::CancellationToken,
    ) -> Result<tokio::task::JoinHandle<Result<(), tonic::transport::Error>>, ()> {
        let svc = ProofGenerationServiceServer::new(fake_prover)
            .send_compressed(CompressionEncoding::Zstd)
            .accept_compressed(CompressionEncoding::Zstd);

        let (mut health_reporter, health_service) = tonic_health::server::health_reporter();

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
        request: tonic::Request<agglayer_prover_types::v1::ProofGenerationRequest>,
    ) -> Result<tonic::Response<agglayer_prover_types::v1::ProofGenerationResponse>, tonic::Status>
    {
        debug!("Received proof generation request");
        let request = request.into_inner();
        let initial_state: LocalNetworkState = agglayer_prover_types::default_bincode_options()
            .deserialize(&request.initial_state)
            .map_err(|_| tonic::Status::invalid_argument("Unable to deserialize initial state"))?;
        let batch_header: MultiBatchHeader<Keccak256Hasher> =
            agglayer_prover_types::default_bincode_options()
                .deserialize(&request.batch_header)
                .map_err(|_| {
                    tonic::Status::invalid_argument("Unable to deserialize batch header")
                })?;
        let request = Request {
            initial_state,
            batch_header,
        };
        let stdin = request.into();

        let proof_opts = sp1_sdk::provers::ProofOpts {
            timeout: Some(Duration::from_secs(60)),
            ..Default::default()
        };
        let context = SP1Context::default();

        let result = self
            .prover
            .prove(
                &self.proving_key,
                stdin,
                proof_opts,
                context,
                SP1ProofKind::Plonk,
            )
            .map_err(|error| Error::ProverFailed(error.to_string()));
        match result {
            Ok(proof) => {
                let proof = agglayer_prover_types::default_bincode_options()
                    .serialize(&agglayer_types::Proof::SP1(proof))
                    .unwrap();
                debug!("Proof generated successfully, size: {}B", proof.len());
                Ok(tonic::Response::new(
                    agglayer_prover_types::v1::ProofGenerationResponse { proof },
                ))
            }
            Err(error) => {
                warn!("FakeProver error: {}", error);
                Err(tonic::Status::invalid_argument(error.to_string()))
            }
        }
    }
}
