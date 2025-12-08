use std::{net::SocketAddr, sync::Arc};

use agglayer_prover_config::DEFAULT_GRPC_MESSAGE_SIZE;
use agglayer_prover_types::{
    bincode,
    v1::{
        generate_proof_request::Stdin,
        pessimistic_proof_service_server::{
            PessimisticProofService, PessimisticProofServiceServer,
        },
    },
    Error,
};
use prover_executor::{sp1_blocking, sp1_fast};
use sp1_sdk::{CpuProver, Prover as _, ProverClient, SP1Stdin};
use tonic::{codec::CompressionEncoding, transport::Server};
use tracing::{debug, error, info, warn};

pub struct FakeProver {
    prover: Arc<CpuProver>,
    proving_key: sp1_sdk::SP1ProvingKey,
}

impl FakeProver {
    pub async fn new(elf: &'static [u8]) -> eyre::Result<Self> {
        sp1_blocking(|| {
            let prover = ProverClient::builder().mock().build();
            let (proving_key, _verifying_key) = prover.setup(elf);
            Self {
                proving_key,
                prover: Arc::new(prover),
            }
        })
        .await
    }
}

impl FakeProver {
    pub async fn spawn_at(
        fake_prover: Self,
        endpoint: SocketAddr,
        cancellation_token: tokio_util::sync::CancellationToken,
    ) -> Result<tokio::task::JoinHandle<Result<(), tonic::transport::Error>>, ()> {
        let svc = PessimisticProofServiceServer::new(fake_prover)
            .max_decoding_message_size(DEFAULT_GRPC_MESSAGE_SIZE)
            .max_encoding_message_size(DEFAULT_GRPC_MESSAGE_SIZE)
            .send_compressed(CompressionEncoding::Zstd)
            .accept_compressed(CompressionEncoding::Zstd);

        let (health_reporter, health_service) = tonic_health::server::health_reporter();

        health_reporter
            .set_serving::<PessimisticProofServiceServer<FakeProver>>()
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
impl PessimisticProofService for FakeProver {
    async fn generate_proof(
        &self,
        request: tonic::Request<agglayer_prover_types::v1::GenerateProofRequest>,
    ) -> Result<tonic::Response<agglayer_prover_types::v1::GenerateProofResponse>, tonic::Status>
    {
        debug!("Received proof generation request");
        let request_inner = request.into_inner();
        let stdin: SP1Stdin = match request_inner.stdin {
            Some(Stdin::Sp1Stdin(stdin)) => sp1_fast(|| bincode::default().deserialize(&stdin))
                .map_err(|_| tonic::Status::invalid_argument("Unable to deserialize stdin"))?
                .map_err(|_| tonic::Status::invalid_argument("Unable to deserialize stdin"))?,
            None => {
                return Err(tonic::Status::invalid_argument("stdin is required"));
            }
        };

        let result = sp1_fast(|| self.prover.prove(&self.proving_key, &stdin).plonk().run())
            .map_err(|error| Error::ProverFailed(error.to_string()))
            .and_then(|res| res.map_err(|error| Error::ProverFailed(error.to_string())));
        match result {
            Ok(proof) => {
                let proof = sp1_fast(|| {
                    bincode::default().serialize(&agglayer_prover_types::Proof::SP1(proof))
                })
                .map_err(|error| Error::ProverFailed(error.to_string()))
                .and_then(|res| res.map_err(|error| Error::ProverFailed(error.to_string())))
                .map_err(|error| {
                    warn!("FakeProver deserialization error: {}", error);
                    tonic::Status::invalid_argument(error.to_string())
                })?;
                debug!("Proof generated successfully, size: {}B", proof.len());
                Ok(tonic::Response::new(
                    agglayer_prover_types::v1::GenerateProofResponse {
                        proof: proof.into(),
                    },
                ))
            }
            Err(error) => {
                warn!("FakeProver error: {}", error);
                Err(tonic::Status::invalid_argument(error.to_string()))
            }
        }
    }
}
