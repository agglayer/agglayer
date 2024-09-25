use std::sync::Arc;

use agglayer_certificate_orchestrator::{Certifier, CertifierOutput, Error};
use agglayer_prover_types::{
    default_bincode_options,
    v1::{
        proof_generation_service_client::ProofGenerationServiceClient, ProofGenerationRequest,
        ProofGenerationResponse,
    },
};
use agglayer_storage::stores::{PendingCertificateReader, PendingCertificateWriter};
use agglayer_types::{Height, LocalNetworkStateData, NetworkId, Proof};
use bincode::Options as _;
use futures::future::BoxFuture;
use pessimistic_proof::{generate_pessimistic_proof, LocalNetworkState};
use reth_primitives::Address;
use sp1_sdk::{CpuProver, Prover as _, SP1VerifyingKey};
use tonic::transport::Channel;
use tracing::{error, info};

use crate::ELF;

#[derive(Clone)]
pub struct CertifierClient<PendingStore> {
    pending_store: Arc<PendingStore>,
    prover: ProofGenerationServiceClient<Channel>,
    verifier: Arc<CpuProver>,
    verifying_key: SP1VerifyingKey,
}

impl<PendingStore> CertifierClient<PendingStore> {
    pub async fn try_new(prover: String, pending_store: Arc<PendingStore>) -> anyhow::Result<Self> {
        let verifier = CpuProver::new();
        let (_, verifying_key) = verifier.setup(ELF);

        Ok(Self {
            pending_store,
            prover: ProofGenerationServiceClient::connect(prover).await?,
            verifier: Arc::new(verifier),
            verifying_key,
        })
    }
}

impl<PendingStore> Certifier for CertifierClient<PendingStore>
where
    PendingStore: PendingCertificateReader + PendingCertificateWriter + 'static,
{
    fn certify(
        &self,
        mut state: LocalNetworkStateData,
        network_id: NetworkId,
        height: Height,
    ) -> Result<BoxFuture<Result<CertifierOutput, Error>>, Error> {
        // Fetch certificate from storage
        let certificate = self
            .pending_store
            .get_certificate(network_id, height)?
            .ok_or(Error::CertificateNotFound(network_id, height))?;

        let certificate_id = certificate.hash();

        if self.pending_store.get_proof(certificate_id)?.is_some() {
            return Err(Error::ProofAlreadyExists(network_id, height));
        }

        let signer = Address::new([0; 20]); // TODO: put the trusted sequencer address

        let initial_state = LocalNetworkState::from(state.clone());
        let multi_batch_header = state.apply_certificate(&certificate, signer)?;

        // Perform the native PP execution
        let _ = generate_pessimistic_proof(initial_state.clone(), &multi_batch_header)
            .map_err(Error::NativeExecutionFailed)?;

        info!(
            "Successfully executed the native PP for the Certificate {:?}",
            certificate.hash()
        );

        let request = ProofGenerationRequest {
            initial_state: default_bincode_options()
                .serialize(&initial_state)
                .map_err(|_| Error::Serialize)?,
            batch_header: default_bincode_options()
                .serialize(&multi_batch_header)
                .map_err(|_| Error::Serialize)?,
        };

        let mut prover_client = self.prover.clone();

        Ok(Box::pin(async move {
            let prover_response: tonic::Response<ProofGenerationResponse> =
                prover_client.generate_proof(request).await.map_err(|_| {
                    Error::ProverExecutionFailed(anyhow::Error::msg("Fail to generate proof"))
                })?;

            let proof: Proof = default_bincode_options()
                .deserialize(&prover_response.into_inner().proof)
                .map_err(|_| Error::Deserialize)?;

            let Proof::SP1(ref proof_to_verify) = proof;

            if let Err(error) = self.verifier.verify(proof_to_verify, &self.verifying_key) {
                error!("Failed to verify the p-proof: {:?}", error);

                Err(Error::ProofVerificationFailed)
            } else {
                info!("Successfully generated and verified the p-proof!");

                // TODO: Check if the key already exists
                self.pending_store
                    .insert_generated_proof(&certificate.hash(), &proof)?;

                Ok(CertifierOutput {
                    certificate,
                    height,
                    new_state: state,
                    network: multi_batch_header.origin_network,
                })
            }
        }))
    }
}
