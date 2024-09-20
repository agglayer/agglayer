use agglayer_prover_types::v1::proof_generation_service_server::ProofGenerationService;
use agglayer_telemetry::prover::{PROVING_REQUEST_RECV, PROVING_REQUEST_SUCCEEDED};
use tracing::debug;

#[derive(Default)]
pub struct ProverRPC {}

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
        let reply = agglayer_prover_types::v1::ProofGenerationResponse {};

        PROVING_REQUEST_SUCCEEDED.add(1, metrics_attrs);
        Ok(tonic::Response::new(reply))
    }
}
