//! Error types for the top-level Agglayer service.

use agglayer_contracts::L1RpcError;
pub use agglayer_storage::error::Error as StorageError;
pub use agglayer_types::primitives::Digest;
use agglayer_types::NetworkId;
use ethers::{contract::ContractError, providers::Middleware, types::Address};

pub use crate::rate_limiting::RateLimited as RateLimitedError;

#[derive(Debug, thiserror::Error)]
pub enum CertificateRetrievalError {
    #[error(transparent)]
    Storage(#[from] StorageError),

    #[error("Data for certificate {certificate_id} not found")]
    NotFound { certificate_id: Digest },
}

#[derive(Debug, thiserror::Error)]
pub enum CertificateSubmissionError<Rpc: Middleware> {
    #[error(transparent)]
    Storage(#[from] StorageError),

    #[error("Failed to send the certificate to the orchestrator")]
    OrchestratorNotResponsive,

    #[error("Failed to validate certificate signature: {0}")]
    SignatureError(#[source] SignatureVerificationError<Rpc>),

    #[error("Unable to replace pending certificate at height {height} for network {network_id}")]
    UnableToReplacePendingCertificate {
        reason: String,
        height: u64,
        network_id: NetworkId,
        stored_certificate_id: Digest,
        replacement_certificate_id: Digest,
        #[source]
        source: Option<L1RpcError>,
    },
}

/// Errors related to signature verification process.
#[derive(thiserror::Error, Debug)]
pub enum SignatureVerificationError<Rpc: Middleware> {
    /// FEP (0.1): The signer could not be recovered from the [`SignedTx`].
    #[error("could not recover transaction signer: {0}")]
    CouldNotRecoverTxSigner(#[source] ethers::types::SignatureError),

    /// The signer could not be recovered from the certificate signature.
    #[error("could not recover certificate signer: {0}")]
    CouldNotRecoverCertSigner(#[source] alloy::primitives::SignatureError),

    /// The signer of the proof is not the trusted sequencer for the given
    /// rollup id.
    #[error("invalid signer: expected {trusted_sequencer}, got {signer}")]
    InvalidSigner {
        /// The recovered signer address.
        signer: Address,
        /// The trusted sequencer address.
        trusted_sequencer: Address,
    },

    #[error("unable to retrieve trusted sequencer address")]
    UnableToRetrieveTrustedSequencerAddress(NetworkId),

    /// Generic network error when attempting to retrieve the trusted sequencer
    /// address from the rollup contract.
    #[error("contract error: {0}")]
    ContractError(#[from] ContractError<Rpc>),

    /// Signature is missing.
    #[error("signature not provided")]
    SignatureMissing,
}

impl<Rpc: Middleware> SignatureVerificationError<Rpc> {
    pub fn from_signer_error(e: agglayer_types::SignerError) -> Self {
        match e {
            agglayer_types::SignerError::Missing => Self::SignatureMissing,
            agglayer_types::SignerError::Recovery(e) => Self::CouldNotRecoverCertSigner(e),
        }
    }
}
