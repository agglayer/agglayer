use agglayer_prover_types::{
    v1::{
        generate_proof_request::Stdin, pessimistic_proof_service_server::PessimisticProofService,
    },
    ErrorWrapper,
};
use agglayer_telemetry::prover::{
    PROVING_REQUEST_FAILED, PROVING_REQUEST_RECV, PROVING_REQUEST_SUCCEEDED,
};
use prover_executor::{sp1_fast, ProofType, Request, Response};
use sp1_sdk::SP1Stdin;
use tonic::Status;
use tower::{buffer::Buffer, util::BoxService, Service, ServiceExt};
use tracing::{debug, error, warn};

pub struct ProverRPC {
    executor: Buffer<BoxService<Request, Response, prover_executor::Error>, Request>,
}

impl ProverRPC {
    pub fn new(
        executor: Buffer<BoxService<Request, Response, prover_executor::Error>, Request>,
    ) -> Self {
        Self { executor }
    }
}

#[tonic::async_trait]
impl PessimisticProofService for ProverRPC {
    async fn generate_proof(
        &self,
        request: tonic::Request<agglayer_prover_types::v1::GenerateProofRequest>,
    ) -> Result<tonic::Response<agglayer_prover_types::v1::GenerateProofResponse>, tonic::Status>
    {
        let metrics_attrs = &[];
        PROVING_REQUEST_RECV.add(1, metrics_attrs);
        debug!("Got a request from {:?}", request.remote_addr());

        let request_inner = request.into_inner();
        let stdin: SP1Stdin = match request_inner.stdin {
            Some(Stdin::Sp1Stdin(stdin)) => {
                sp1_fast(|| agglayer_prover_types::bincode::default().deserialize(&stdin))
                    .map_err(|_| tonic::Status::invalid_argument("Unable to deserialize stdin"))?
                    .map_err(|_| tonic::Status::invalid_argument("Unable to deserialize stdin"))?
            }
            None => {
                return Err(tonic::Status::invalid_argument("stdin is required"));
            }
        };

        let mut executor = self.executor.clone();
        let executor = executor
            .ready()
            .await
            .map_err(|_error| tonic::Status::internal("Unable to get proof executor"))?;

        let request = Request {
            stdin,
            proof_type: ProofType::Plonk,
        };

        match executor.call(request).await {
            Ok(result) => {
                let response = agglayer_prover_types::v1::GenerateProofResponse {
                    proof: sp1_fast(|| {
                        agglayer_prover_types::bincode::default()
                            .serialize(&agglayer_prover_types::Proof::SP1(result.proof))
                    })
                    .map_err(|_| tonic::Status::internal("Unable to serialize generated proof"))?
                    .map_err(|_| tonic::Status::internal("Unable to serialize generated proof"))?
                    .into(),
                };

                PROVING_REQUEST_SUCCEEDED.add(1, metrics_attrs);
                return Ok(tonic::Response::new(response));
            }
            Err(error) => {
                PROVING_REQUEST_FAILED.add(1, metrics_attrs);
                if let Some(error) = error.downcast_ref::<prover_executor::Error>() {
                    error!("Failed to generate proof: {}", error);

                    let response: Status =
                        ErrorWrapper::try_into_status(error).unwrap_or_else(|inner_error| {
                            warn!("Unable to serialize the prover error: {}", inner_error);
                            tonic::Status::invalid_argument(error.to_string())
                        });

                    return Err(response);
                } else {
                    error!("Failed to generate proof: {:?}", error);

                    return Err(tonic::Status::internal("Failed to generate proof"));
                }
            }
        }
    }
}
