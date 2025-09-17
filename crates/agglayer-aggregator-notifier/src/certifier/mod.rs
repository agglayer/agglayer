use std::{panic::AssertUnwindSafe, sync::Arc};

use agglayer_certificate_orchestrator::{CertificationError, Certifier, CertifierOutput};
use agglayer_config::Config;
use agglayer_contracts::{aggchain::AggchainContract, RollupContract};
use agglayer_prover_types::v1::{
    generate_proof_request::Stdin, pessimistic_proof_service_client::PessimisticProofServiceClient,
    ErrorKind, GenerateProofRequest, GenerateProofResponse,
};
use agglayer_storage::stores::{PendingCertificateReader, PendingCertificateWriter};
use agglayer_types::{
    aggchain_proof::AggchainData, bincode, Certificate, Digest, Height, LocalNetworkStateData,
    NetworkId, Proof,
};
use eyre::{eyre, Context as _};
use pessimistic_proof::{
    core::{commitment::StateCommitment, generate_pessimistic_proof, AggchainHashValues},
    local_state::LocalNetworkState,
    multi_batch_header::MultiBatchHeader,
    unified_bridge::{
        AggchainProofPublicValues, ImportedBridgeExitCommitmentValues,
        ImportedBridgeExitCommitmentVersion,
    },
    NetworkState, PessimisticProofOutput,
};
use prover_executor::{sp1_blocking, sp1_fast};
use sp1_sdk::{
    CpuProver, Prover, SP1ProofWithPublicValues, SP1Stdin, SP1VerificationError, SP1VerifyingKey,
};
use tonic::{codec::CompressionEncoding, transport::Channel};
use tracing::{debug, error, info, instrument, warn};

use crate::ELF;

mod l1_context;

#[cfg(test)]
mod tests;

#[derive(Clone)]
pub struct CertifierClient<PendingStore, L1Rpc> {
    /// The pending store to fetch and store certificates and proofs.
    pending_store: Arc<PendingStore>,
    /// The prover service client.
    prover: PessimisticProofServiceClient<Channel>,
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
    ) -> eyre::Result<Self> {
        debug!("Initializing the CertifierClient verifier...");
        let (verifier, verifying_key) = sp1_blocking({
            let mock_verifier = config.mock_verifier;
            move || {
                let verifier = if mock_verifier {
                    sp1_sdk::ProverClient::builder().mock().build()
                } else {
                    sp1_sdk::ProverClient::builder().cpu().build()
                };
                let (_, verifying_key) = verifier.setup(ELF);
                (verifier, verifying_key)
            }
        })
        .await
        .context("Failed setting up SP1 verifier")?;
        debug!("CertifierClient verifier successfully initialized!");

        debug!("Connecting to the prover service...");

        let prover = PessimisticProofServiceClient::connect(prover)
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
    ) -> eyre::Result<()> {
        // This fail_point is use to make the verification pass or fail
        fail::fail_point!(
            "notifier::certifier::certify::before_verifying_proof",
            |_| {
                let verifier = sp1_sdk::ProverClient::builder().mock().build();
                let (_, verifying_key) = verifier.setup(ELF);

                Ok(verifier.verify(proof, &verifying_key)?)
            }
        );

        Ok(sp1_fast(|| verifier.verify(proof, verifying_key))
            .context("Failed verifying sp1 proof")??)
    }
}

#[async_trait::async_trait]
impl<PendingStore, L1Rpc> Certifier for CertifierClient<PendingStore, L1Rpc>
where
    PendingStore: PendingCertificateReader + PendingCertificateWriter + 'static,
    L1Rpc: RollupContract + AggchainContract + Send + Sync + 'static,
{
    #[instrument(skip(self, state, height), fields(certificate_id, %network_id), level = "info")]
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
        tracing::Span::current().record("certificate_id", certificate_id.to_string());

        let mut prover_client = self.prover.clone();
        let pending_store = self.pending_store.clone();
        let verifier = self.verifier.clone();
        let verifying_key = self.verifying_key.clone();

        let mut state = state.clone();
        let (multi_batch_header, initial_state, pv_native) =
            self.witness_generation(&certificate, &mut state).await?;

        info!("Successfully generated the witness for the PP for the Certificate {certificate_id}");

        let network_state = pessimistic_proof::NetworkState::from(initial_state);
        let mut stdin = sp1_fast(|| {
            let mut stdin = SP1Stdin::new();
            stdin.write(&network_state);
            stdin.write(&multi_batch_header);
            stdin
        })
        .map_err(CertificationError::Other)?;

        // Writing the proof to the stdin if needed
        // At this point, we have the proof and the verifying key coming from the chain
        // The witness execution already checked that the vk in the proof is valid and
        // the multibatch header is configured to use the hash from L1
        match certificate.aggchain_data {
            AggchainData::ECDSA { .. } => {}
            AggchainData::MultisigOnly(_) => {}
            AggchainData::Generic { ref proof, .. } => {
                let agglayer_types::aggchain_proof::Proof::SP1Stark(stark_proof) = proof;

                // This operation is unwind safe: if it errors, we will discard stdin and
                // stark_proof anyway.
                sp1_fast(AssertUnwindSafe(|| {
                    stdin.write_proof((*stark_proof.proof).clone(), stark_proof.vkey.vk.clone())
                }))
                .map_err(CertificationError::Other)?;
            }
            AggchainData::MultisigAndAggchainProof {
                ref aggchain_proof, ..
            } => {
                let agglayer_types::aggchain_proof::Proof::SP1Stark(stark_proof) =
                    &aggchain_proof.proof;
                // This operation is unwind safe: if it errors, we will discard stdin and
                // stark_proof anyway.
                sp1_fast(AssertUnwindSafe(|| {
                    stdin.write_proof((*stark_proof.proof).clone(), stark_proof.vkey.vk.clone());
                }))
                .map_err(CertificationError::Other)?;
            }
        };

        // SP1 native execution which includes the aggchain proof stark verification
        let (pv_sp1_execute, _report) = {
            // Do not verify the deferred proof if we are in mock mode
            let deferred_proof_verification = !self.config.mock_verifier;
            let (pv, report) = sp1_blocking({
                let verifier = self.verifier.clone();
                let stdin = stdin.clone();
                move || {
                    verifier
                        .execute(ELF, &stdin)
                        .deferred_proof_verification(deferred_proof_verification)
                        .run()
                }
            })
            .await
            .map_err(CertificationError::Other)?
            .map_err(|e| CertificationError::Sp1ExecuteFailed(eyre!(e)))?;

            let pv_sp1_execute: PessimisticProofOutput = PessimisticProofOutput::bincode_codec()
                .deserialize(pv.as_slice())
                .map_err(|source| CertificationError::Deserialize { source })?;

            (pv_sp1_execute, report)
        };

        if pv_sp1_execute != pv_native {
            return Err(CertificationError::MismatchPessimisticProofPublicValues {
                native_execution: Box::new(pv_native),
                sp1_zkvm_execution: Box::new(pv_sp1_execute),
            });
        }

        info!(
            public_inputs = ?pv_sp1_execute,
            "Successfully executed the PP program locally"
        );

        let request = GenerateProofRequest {
            stdin: Some(Stdin::Sp1Stdin(
                sp1_fast(|| bincode::default().serialize(&stdin))
                    .map_err(CertificationError::Other)?
                    .map_err(|source| CertificationError::Serialize { source })?
                    .into(),
            )),
        };

        info!("Sending the Proof generation request to the agglayer-prover service...");
        let prover_response: tonic::Response<GenerateProofResponse> = prover_client
            .generate_proof(request)
            .await
            .map_err(|source_error| {
                debug!("Failed to generate the p-proof: {:?}", source_error);
                if let Ok(error) = bincode::default()
                    .deserialize::<agglayer_prover_types::v1::GenerateProofError>(
                        source_error.details(),
                    )
                {
                    match error.error_type() {
                        ErrorKind::UnableToExecuteProver => {
                            CertificationError::InternalError("Unable to execute prover".into())
                        }
                        ErrorKind::ProverFailed => {
                            CertificationError::ProverFailed(source_error.message().to_string())
                        }
                        ErrorKind::ProofVerificationFailed => {
                            let proof_error: Result<
                                pessimistic_proof::error::ProofVerificationError,
                                _,
                            > = bincode::default().deserialize(&error.error);

                            match proof_error {
                                Ok(error) => {
                                    CertificationError::ProofVerificationFailed { source: error }
                                }
                                Err(_source) => {
                                    warn!(
                                        "Failed to deserialize the error details coming from the \
                                         prover: {source_error:?}"
                                    );

                                    CertificationError::InternalError(
                                        source_error.message().to_string(),
                                    )
                                }
                            }
                        }

                        ErrorKind::ExecutorFailed => {
                            let proof_error: Result<pessimistic_proof::ProofError, _> =
                                bincode::default().deserialize(&error.error);

                            match proof_error {
                                Ok(error) => {
                                    CertificationError::ProverExecutionFailed { source: error }
                                }
                                Err(_source) => {
                                    warn!(
                                        "Failed to deserialize the error details coming from the \
                                         prover: {source_error:?}"
                                    );

                                    CertificationError::InternalError(
                                        source_error.message().to_string(),
                                    )
                                }
                            }
                        }
                        ErrorKind::Unspecified => {
                            CertificationError::InternalError(source_error.message().to_string())
                        }
                    }
                } else {
                    warn!(
                        "Failed to deserialize the error details coming from the prover: \
                         {source_error:?}"
                    );

                    CertificationError::InternalError(source_error.message().to_string())
                }
            })?;

        let proof = prover_response.into_inner().proof;
        let proof: Proof = sp1_fast(|| bincode::default().deserialize(&proof))
            .map_err(CertificationError::Other)?
            .map_err(|source| CertificationError::Deserialize { source })?;

        debug!("Proof successfully generated!");

        let Proof::SP1(ref proof_to_verify) = proof;

        debug!("Verifying the generated p-proof...");

        if let Err(error) = Self::verify_proof(verifier, &verifying_key, proof_to_verify) {
            error!("Failed to verify the p-proof: {:?}", error);
            match error.downcast::<SP1VerificationError>() {
                Ok(error) => Err(CertificationError::ProofVerificationFailed {
                    source: error.into(),
                }),
                Err(error) => Err(CertificationError::Other(error)),
            }
        } else {
            info!("Successfully generated and verified the p-proof!");

            // TODO: Check if the key already exists
            pending_store.insert_generated_proof(&certificate_id, &proof)?;

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

    async fn witness_generation(
        &self,
        certificate: &Certificate,
        state: &mut LocalNetworkStateData,
    ) -> Result<(MultiBatchHeader, LocalNetworkState, PessimisticProofOutput), CertificationError>
    {
        // Fetch all the necessary context from the L1
        let ctx_from_l1 = self.fetch_l1_context(certificate).await?;

        debug!("Context fetched from the L1: {ctx_from_l1:?}");

        let initial_state = LocalNetworkState::from(state.clone());

        let multi_batch_header = state
            .apply_certificate(certificate, ctx_from_l1)
            .map_err(|source| CertificationError::Types { source })?;

        let aggchain_hash_values = AggchainHashValues::from(&multi_batch_header.aggchain_data);

        debug!(
            "Unrolled aggchain data values: {:?}",
            multi_batch_header.aggchain_data
        );
        debug!("Unrolled aggchain hash values: {:?}", aggchain_hash_values);
        debug!(
            "Aggchain hash computed by the agglayer node and PP: {:?}",
            aggchain_hash_values.hash()
        );

        let targets_witness_generation: StateCommitment = {
            let ns: LocalNetworkState = state.clone().into();
            NetworkState::from(ns).get_state_commitment()
        };

        // Perform the native PP execution without the STARK verification in order to
        // cross check the target roots.
        let (pv, targets_native_execution) = tokio::task::spawn_blocking({
            let initial_state = initial_state.clone();
            let multi_batch_header = multi_batch_header.clone();
            move || generate_pessimistic_proof(initial_state.into(), &multi_batch_header)
        })
        .await
        .map_err(|e| CertificationError::InternalError(e.to_string()))?
        .map_err(|source| CertificationError::NativeExecutionFailed { source })?;

        // Verify consistency on the aggchain proof public values if provided in the
        // optional context
        if let AggchainData::Generic {
            public_values: Some(pv_from_proof),
            aggchain_params,
            ..
        } = &certificate.aggchain_data
        {
            // Fetching rollup contract address
            let rollup_address = self
                .l1_rpc
                .get_rollup_contract_address(certificate.network_id.to_u32())
                .await
                .map_err(CertificationError::RollupContractAddressNotFound)?;

            // Verify matching on the aggchain hash between the L1 and the agglayer
            let l1_aggchain_hash: Digest = self
                .l1_rpc
                .get_aggchain_hash(rollup_address, certificate.custom_chain_data.clone().into())
                .await
                .map_err(CertificationError::UnableToFindAggchainHash)?
                .into();

            let computed_aggchain_hash = multi_batch_header.aggchain_data.aggchain_hash();

            if l1_aggchain_hash != computed_aggchain_hash {
                return Err(CertificationError::AggchainHashMismatch {
                    from_l1: l1_aggchain_hash,
                    from_certificate: computed_aggchain_hash,
                });
            }

            // Consistency check across these 2 sources:
            //
            // - Public values expected by the proof (i.e., the valid ones to succeed the
            //   proof verification, provided as metadata in the Certificate as-is)
            //
            // - Public values expected by the PP (i.e., the ones used to verify the
            //   aggchain proof in the PP)
            debug!("Public values expected by the certificate's aggchain-proof: {pv_from_proof:?}");

            let pv_from_pp_witness = AggchainProofPublicValues {
                prev_local_exit_root: initial_state.exit_tree.get_root(),
                new_local_exit_root: targets_native_execution.exit_root.into(),
                l1_info_root: multi_batch_header.l1_info_root,
                origin_network: multi_batch_header.origin_network,
                commit_imported_bridge_exits: ImportedBridgeExitCommitmentValues {
                    claims: multi_batch_header
                        .imported_bridge_exits
                        .iter()
                        .map(|(exit, _)| exit.to_indexed_exit_hash())
                        .collect(),
                }
                .commitment(ImportedBridgeExitCommitmentVersion::V3),
                aggchain_params: *aggchain_params,
            };

            if **pv_from_proof != pv_from_pp_witness {
                error!("Mismatch on the aggchain proof public values.");
                return Err(CertificationError::AggchainProofPublicValuesMismatch {
                    from_proof: pv_from_proof.clone(),
                    from_witness: Box::new(pv_from_pp_witness),
                });
            }
        }

        if targets_witness_generation != targets_native_execution {
            return Err(CertificationError::StateCommitmentMismatch {
                witness_generation: Box::new(targets_witness_generation),
                native_execution: Box::new(targets_native_execution),
            });
        }

        Ok((multi_batch_header, initial_state, pv))
    }
}
