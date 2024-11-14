use agglayer_prover_types::{
    default_bincode_options, v1::proof_generation_service_server::ProofGenerationService, Error,
};
use agglayer_telemetry::prover::{
    PROVING_REQUEST_FAILED, PROVING_REQUEST_RECV, PROVING_REQUEST_SUCCEEDED,
};
use bincode::Options;
use pessimistic_proof::{
    local_exit_tree::hasher::Keccak256Hasher, multi_batch_header::MultiBatchHeader,
    LocalNetworkState,
};
use tower::{buffer::Buffer, util::BoxService, Service, ServiceExt};
use tracing::{debug, error, warn};

use crate::executor::{Request, Response};

pub struct ProverRPC {
    executor: Buffer<BoxService<Request, Response, Error>, Request>,
}

impl ProverRPC {
    pub fn new(executor: Buffer<BoxService<Request, Response, Error>, Request>) -> Self {
        Self { executor }
    }
}

#[tonic::async_trait]
impl ProofGenerationService for ProverRPC {
    async fn generate_proof(
        &self,
        request: tonic::Request<agglayer_prover_types::v1::ProofGenerationRequest>,
    ) -> Result<tonic::Response<agglayer_prover_types::v1::ProofGenerationResponse>, tonic::Status>
    {
        let metrics_attrs = &[];
        PROVING_REQUEST_RECV.add(1, metrics_attrs);
        debug!("Got a request from {:?}", request.remote_addr());

        let request_inner = request.into_inner();
        let initial_state: LocalNetworkState = agglayer_prover_types::default_bincode_options()
            .deserialize(&request_inner.initial_state)
            .map_err(|_| tonic::Status::invalid_argument("Unable to deserialize initial state"))?;

        let batch_header: MultiBatchHeader<Keccak256Hasher> =
            agglayer_prover_types::default_bincode_options()
                .deserialize(&request_inner.batch_header)
                .map_err(|_| {
                    tonic::Status::invalid_argument("Unable to deserialize batch header")
                })?;

        let mut executor = self.executor.clone();
        let executor = executor
            .ready()
            .await
            .map_err(|_error| tonic::Status::internal("Unable to get proof executor"))?;

        match executor
            .call(Request {
                initial_state,
                batch_header,
            })
            .await
        {
            Ok(result) => {
                let response = agglayer_prover_types::v1::ProofGenerationResponse {
                    proof: agglayer_prover_types::default_bincode_options()
                        .serialize(&agglayer_types::Proof::SP1(result.proof))
                        .map_err(|_| {
                            tonic::Status::internal("Unable to serialize generated proof")
                        })?,
                };

                PROVING_REQUEST_SUCCEEDED.add(1, metrics_attrs);
                return Ok(tonic::Response::new(response));
            }
            Err(error) => {
                PROVING_REQUEST_FAILED.add(1, metrics_attrs);
                if let Some(error) = error.downcast_ref::<Error>() {
                    error!("Failed to generate proof: {}", error);
                    let response = match error {
                        Error::UnableToExecuteProver => {
                            tonic::Status::internal("Unable to execute prover")
                        }
                        Error::ExecutorFailed(inner_error) => {
                            if let Ok(bytes_error) = default_bincode_options().serialize(&error) {
                                tonic::Status::with_details(
                                    tonic::Code::InvalidArgument,
                                    error.to_string(),
                                    bytes_error.into(),
                                )
                            } else {
                                warn!("Unable to serialize Execution  error: {}", inner_error);
                                tonic::Status::invalid_argument(error.to_string())
                            }
                        }
                        Error::ProofVerificationFailed(inner_error) => {
                            if let Ok(bytes_error) = default_bincode_options().serialize(&error) {
                                tonic::Status::with_details(
                                    tonic::Code::InvalidArgument,
                                    error.to_string(),
                                    bytes_error.into(),
                                )
                            } else {
                                warn!(
                                    "Unable to serialize SP1 verification error: {}",
                                    inner_error
                                );
                                tonic::Status::invalid_argument(error.to_string())
                            }
                        }
                        Error::ProverFailed(_) => tonic::Status::internal(error.to_string()),
                    };

                    return Err(response);
                } else {
                    error!("Failed to generate proof: {:?}", error);

                    return Err(tonic::Status::internal("Failed to generate proof"));
                }
            }
        }
    }
}
