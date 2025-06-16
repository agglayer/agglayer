//! Error types for the top-level Agglayer service.

use agglayer_contracts::L1RpcError;
pub use agglayer_storage::error::Error as StorageError;
pub use agglayer_types::primitives::Digest;
use agglayer_types::{CertificateId, Height, NetworkId};
use alloy::{contract::Error as ContractError, primitives::Address};

pub use crate::rate_limiting::RateLimited as RateLimitedError;

#[derive(Debug, thiserror::Error)]
pub enum CertificateRetrievalError {
    #[error(transparent)]
    Storage(#[from] StorageError),

    #[error("Data for certificate {certificate_id} not found")]
    NotFound { certificate_id: CertificateId },
}

#[derive(Debug, thiserror::Error)]
pub enum CertificateSubmissionError {
    #[error(transparent)]
    Storage(#[from] StorageError),

    #[error("Failed to send the certificate to the orchestrator")]
    OrchestratorNotResponsive,

    #[error("Failed to validate certificate signature: {0}")]
    SignatureError(#[source] SignatureVerificationError),

    #[error("Unable to replace pending certificate at height {height} for network {network_id}")]
    UnableToReplacePendingCertificate {
        reason: String,
        height: Height,
        network_id: NetworkId,
        stored_certificate_id: CertificateId,
        replacement_certificate_id: CertificateId,
        #[source]
        source: Option<L1RpcError>,
    },
}

/// Errors related to signature verification process.
#[derive(thiserror::Error, Debug)]
pub enum SignatureVerificationError {
    /// FEP (0.1): The signer could not be recovered from the [`SignedTx`].
    #[error("could not recover transaction signer: {0}")]
    CouldNotRecoverTxSigner(#[source] alloy::primitives::SignatureError),

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
    ContractError(#[from] ContractError),

    /// SP1-based Aggchain proof not yet supported.
    #[error("SP1-based Aggchain proof not yet supported")]
    SP1AggchainProofUnsupported,
}
