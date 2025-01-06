use std::sync::Arc;

use agglayer_certificate_orchestrator::{CertificationError, Certifier, CertifierOutput};
use agglayer_config::Config;
use agglayer_contracts::RollupContract;
use agglayer_prover_types::{
    default_bincode_options,
    v1::{
        proof_generation_service_client::ProofGenerationServiceClient, ProofGenerationRequest,
        ProofGenerationResponse,
    },
};
use agglayer_storage::stores::{PendingCertificateReader, PendingCertificateWriter};
use agglayer_types::{
    primitives::Address, Certificate, Height, LocalNetworkStateData, NetworkId, Proof,
};
use bincode::Options as _;
use pessimistic_proof::{
    generate_pessimistic_proof, local_exit_tree::hasher::Keccak256Hasher,
    multi_batch_header::MultiBatchHeader, LocalNetworkState,
};
use sp1_sdk::{CpuProver, Prover, SP1ProofWithPublicValues, SP1VerificationError, SP1VerifyingKey};
use tonic::{codec::CompressionEncoding, transport::Channel};
use tracing::{debug, error, info, instrument, warn};

use crate::ELF;

#[cfg(test)]
mod tests;

#[derive(Clone)]
pub struct CertifierClient<PendingStore, L1Rpc> {
    /// The pending store to fetch and store certificates and proofs.
    pending_store: Arc<PendingStore>,
    /// The prover service client.
    prover: ProofGenerationServiceClient<Channel>,
    /// The local CPU verifier to verify the generated proofs.
    verifier: Arc<CpuProver>,
    /// The verifying key of the SP1 proof system.
    verifying_key: SP1VerifyingKey,
    /// The L1 RPC client.
    l1_rpc: Arc<L1Rpc>,
    config: Arc<Config>,
}

impl<PendingStore, L1Rpc> CertifierClient<PendingStore, L1Rpc> {
    pub async fn try_new(
        prover: String,
        pending_store: Arc<PendingStore>,
        l1_rpc: Arc<L1Rpc>,
        config: Arc<Config>,
    ) -> anyhow::Result<Self> {
        debug!("Initializing the CertifierClient verifier...");
        let verifier = CpuProver::new();
        let (_, verifying_key) = verifier.setup(ELF);
        debug!("CertifierClient verifier successfully initialized!");

        debug!("Connecting to the prover service...");

        let prover = ProofGenerationServiceClient::connect(prover)
            .await?
            .max_decoding_message_size(config.prover.grpc.max_decoding_message_size)
            .max_encoding_message_size(config.prover.grpc.max_encoding_message_size)
            .send_compressed(CompressionEncoding::Zstd)
            .accept_compressed(CompressionEncoding::Zstd);
        debug!("Successfully connected to the prover service!");

        Ok(Self {
            pending_store,
            prover,
            verifier: Arc::new(verifier),
            verifying_key,
            l1_rpc,
            config,
        })
    }

    fn verify_proof(
        verifier: Arc<CpuProver>,
        verifying_key: &SP1VerifyingKey,
        proof: &SP1ProofWithPublicValues,
    ) -> Result<(), SP1VerificationError> {
        // This fail_point is use to make the verification pass or fail
        fail::fail_point!(
            "notifier::certifier::certify::before_verifying_proof",
            |_| {
                let verifier = sp1_sdk::MockProver::new();
                let (_, verifying_key) = verifier.setup(ELF);

                verifier.verify(proof, &verifying_key)
            }
        );

        verifier.verify(proof, verifying_key)
    }
}

#[async_trait::async_trait]
impl<PendingStore, L1Rpc> Certifier for CertifierClient<PendingStore, L1Rpc>
where
    PendingStore: PendingCertificateReader + PendingCertificateWriter + 'static,
    L1Rpc: RollupContract + Send + Sync + 'static,
{
    #[instrument(skip(self, state, height), fields(hash, %network_id), level = "info")]
    async fn certify(
        &self,
        state: LocalNetworkStateData,
        network_id: NetworkId,
        height: Height,
    ) -> Result<CertifierOutput, CertificationError> {
        debug!("Certifying the certificate of network {network_id} at height {height}");

        // Fetch certificate from storage
        let certificate = self
            .pending_store
            .get_certificate(network_id, height)?
            .ok_or(CertificationError::CertificateNotFound(network_id, height))?;

        let certificate_id = certificate.hash();
        tracing::Span::current().record("hash", certificate_id.to_string());

        let mut prover_client = self.prover.clone();
        let pending_store = self.pending_store.clone();
        let verifier = self.verifier.clone();
        let verifying_key = self.verifying_key.clone();

        let mut state = state.clone();
        let (multi_batch_header, initial_state) =
            self.witness_execution(&certificate, &mut state).await?;

        info!(
            "Successfully executed the native PP for the Certificate {}",
            certificate_id
        );

        let request = ProofGenerationRequest {
            initial_state: default_bincode_options()
                .serialize(&initial_state)
                .map_err(|source| CertificationError::Serialize { source })?,
            batch_header: default_bincode_options()
                .serialize(&multi_batch_header)
                .map_err(|source| CertificationError::Serialize { source })?,
        };

        info!("Sending the Proof generation request to the agglayer-prover service...");
        let prover_response: tonic::Response<ProofGenerationResponse> = prover_client
            .generate_proof(request)
            .await
            .map_err(|error| {
                debug!("Failed to generate the p-proof: {:?}", error);
                if let Ok(error) = default_bincode_options()
                    .deserialize::<agglayer_prover_types::Error>(error.details())
                {
                    match error {
                        agglayer_prover_types::Error::UnableToExecuteProver => {
                            CertificationError::InternalError("Unable to execute prover".into())
                        }
                        agglayer_prover_types::Error::ProverFailed(error) => {
                            CertificationError::InternalError(error)
                        }
                        agglayer_prover_types::Error::ProofVerificationFailed(error) => {
                            CertificationError::ProofVerificationFailed { source: error }
                        }
                        agglayer_prover_types::Error::ExecutorFailed(source) => {
                            CertificationError::NativeExecutionFailed { source }
                        }
                    }
                } else {
                    warn!(
                        "Failed to deserialize the error details coming from the prover: {error:?}"
                    );

                    CertificationError::InternalError(error.message().to_string())
                }
            })?;

        let proof = prover_response.into_inner().proof;
        let proof: Proof = default_bincode_options()
            .deserialize(&proof)
            .map_err(|source| CertificationError::Deserialize { source })?;

        debug!("Proof successfully generated!");

        let Proof::SP1(ref proof_to_verify) = proof;

        debug!("Verifying the generated p-proof...");

        if let Err(error) = Self::verify_proof(verifier, &verifying_key, proof_to_verify) {
            error!("Failed to verify the p-proof: {:?}", error);

            Err(CertificationError::ProofVerificationFailed {
                source: error.into(),
            })
        } else {
            info!("Successfully generated and verified the p-proof!");

            // TODO: Check if the key already exists
            pending_store.insert_generated_proof(&certificate.hash(), &proof)?;

            // Prune the SMTs of the state
            state
                .prune_stale_nodes()
                .map_err(|e| CertificationError::InternalError(e.to_string()))?;

            Ok(CertifierOutput {
                certificate,
                height,
                new_state: state,
                network: multi_batch_header.origin_network,
            })
        }
    }

    async fn witness_execution(
        &self,
        certificate: &Certificate,
        state: &mut LocalNetworkStateData,
    ) -> Result<(MultiBatchHeader<Keccak256Hasher>, LocalNetworkState), CertificationError> {
        let network_id = certificate.network_id;
        let certificate_id = certificate.hash();

        let signer = self
            .l1_rpc
            .get_trusted_sequencer_address(*network_id, self.config.proof_signers.clone())
            .await
            .map_err(|_| CertificationError::TrustedSequencerNotFound(network_id))?;

        let l1_info_leaf_count = certificate
            .l1_info_tree_leaf_count()
            .unwrap_or_else(|| self.l1_rpc.default_l1_info_tree_entry().0);

        let l1_info_root = self
            .l1_rpc
            .get_l1_info_root(l1_info_leaf_count)
            .await
            .map_err(|_| {
                CertificationError::L1InfoRootNotFound(certificate_id, l1_info_leaf_count)
            })?
            .into();

        let declared_l1_info_root = certificate
            .l1_info_root()
            .map_err(|source| CertificationError::Types { source })?;

        if let Some(declared) = declared_l1_info_root {
            if declared != l1_info_root {
                return Err(CertificationError::Types {
                    source: agglayer_types::Error::L1InfoRootIncorrect {
                        declared,
                        retrieved: l1_info_root,
                        leaf_count: l1_info_leaf_count,
                    },
                });
            }
        }

        let initial_state = LocalNetworkState::from(state.clone());

        let signer = Address::new(*signer.as_fixed_bytes());
        let multi_batch_header = state
            .apply_certificate(certificate, signer, l1_info_root)
            .map_err(|source| CertificationError::Types { source })?;

        // Perform the native PP execution
        let _ = generate_pessimistic_proof(initial_state.clone(), &multi_batch_header)
            .map_err(|source| CertificationError::NativeExecutionFailed { source })?;

        Ok((multi_batch_header, initial_state))
    }
}
