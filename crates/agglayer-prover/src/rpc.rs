use agglayer_prover_types::v1::proof_generation_service_server::ProofGenerationService;
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
        debug!("Got a request from {:?}", request.remote_addr());
        let reply = agglayer_prover_types::v1::ProofGenerationResponse {};

        Ok(tonic::Response::new(reply))
    }
}
