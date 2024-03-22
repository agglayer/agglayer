//! The gRPC service implementation.
use ethers::{
    providers::Middleware,
    types::{SignatureError, H256},
};
use futures::TryFutureExt;
use thiserror::Error;
use tokio::try_join;
use tonic::{Request, Response, Status};

pub mod proto {
    tonic::include_proto!("agglayer");
}

use proto::{agglayer_server::Agglayer, SubmitProofResponse};
use tracing::instrument;

use crate::{
    kernel::Kernel,
    signed_proof::{Proof, ProofEncodingError, ProofManifest, SignedProof, Zkp},
};

/// Possible errors when unmarshalling a `SignedProof` from the protocol buffer
/// input.
#[derive(Error, Debug)]
pub(crate) enum SignedProofValidationError {
    /// The manifest struct is missing from the input.
    #[error("missing manifest")]
    MissingManifest,
    /// The ZKP struct is missing from the input.
    #[error("missing zkp")]
    MissingZkp,
    /// The signature is invalid.
    #[error("invalid signature: {0}")]
    InvalidSignature(SignatureError),
    /// The proof encoding is invalid.
    #[error("invalid proof: {0}")]
    InvalidProof(ProofEncodingError),
    /// The length of a hash is invalid.
    #[error("invalid hash length: expected {expected:?} bytes, got {got:?} bytes")]
    InvalidHashLength { expected: usize, got: usize },
}

/// Safe conversion of a byte slice into an `H256`.
///
/// [`H256::from_slice`] will panic if the input slice is not 32 bytes long.
fn try_slice_into_h256(input: &[u8]) -> Result<H256, SignedProofValidationError> {
    if input.len() != 32 {
        return Err(SignedProofValidationError::InvalidHashLength {
            expected: 32,
            got: input.len(),
        });
    }
    Ok(H256::from_slice(input))
}

/// Conversion of a protocol buffer [`proto::SignedProof`] into a
/// [`SignedProof`].
///
/// Protocol buffers are fairly limited in the types they can represent, so we
/// implement a conversion to our well-typed `SignedProof` type, which may
/// fail given malformed input.
impl TryFrom<proto::SignedProof> for SignedProof {
    type Error = SignedProofValidationError;

    fn try_from(input: proto::SignedProof) -> Result<Self, Self::Error> {
        let manifest = input
            .manifest
            .ok_or(SignedProofValidationError::MissingManifest)?;
        let zkp = manifest.zkp.ok_or(SignedProofValidationError::MissingZkp)?;
        let proof =
            Proof::try_from_slice(&zkp.proof).map_err(SignedProofValidationError::InvalidProof)?;
        let signature = input
            .signature
            .as_slice()
            .try_into()
            .map_err(SignedProofValidationError::InvalidSignature)?;
        let new_state_root = try_slice_into_h256(&zkp.new_state_root)?;
        let new_local_exit_root = try_slice_into_h256(&zkp.new_local_exit_root)?;

        Ok(Self {
            manifest: ProofManifest {
                rollup_id: manifest.rollup_id,
                last_verified_batch: manifest.last_verified_batch,
                new_verified_batch: manifest.new_verified_batch,
                zkp: Zkp {
                    new_state_root,
                    new_local_exit_root,
                    proof,
                },
            },
            signature,
        })
    }
}

/// The gRPC agglayer service implementation.
pub(crate) struct AgglayerImpl<Rpc> {
    kernel: Kernel<Rpc>,
}

impl<Rpc> AgglayerImpl<Rpc> {
    /// Create an instance of the gRPC agglayer service.
    pub(crate) fn new(kernel: Kernel<Rpc>) -> Self {
        Self { kernel }
    }
}

#[tonic::async_trait]
impl<Rpc> Agglayer for AgglayerImpl<Rpc>
where
    Rpc: Send + Sync + Middleware + 'static,
{
    #[instrument(skip(self, request))]
    async fn submit_proof(
        &self,
        request: Request<proto::SignedProof>,
    ) -> Result<Response<SubmitProofResponse>, Status> {
        // Convert the protocol buffer into a well-typed `SignedProof`.
        let proof = SignedProof::try_from(request.into_inner())
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        // Run all the verification checks in parallel.
        try_join!(
            self.kernel
                .verify_signature(&proof)
                .map_err(|e| Status::permission_denied(e.to_string())),
            self.kernel
                .verify_proof_eth_call(&proof)
                .map_err(|e| Status::invalid_argument(e.to_string())),
            self.kernel
                .verify_proof_zkevm_node(&proof)
                .map_err(|e| Status::invalid_argument(e.to_string()))
        )?;

        // Settle the proof on-chain and return the transaction hash.
        let receipt = self
            .kernel
            .settle(&proof)
            .await
            .map_err(|e| Status::aborted(e.to_string()))?;

        Ok(Response::new(SubmitProofResponse {
            tx_hash: receipt.transaction_hash.to_string(),
        }))
    }
}
