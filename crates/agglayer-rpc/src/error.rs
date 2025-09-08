//! Error types for the top-level Agglayer service.
use agglayer_contracts::L1RpcError;
pub use agglayer_storage::error::Error as StorageError;
pub use agglayer_types::primitives::Digest;
use agglayer_types::{Address, CertificateId, Height, NetworkId, SignerError};
use alloy::contract::Error as ContractError;

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
    CouldNotRecoverCertSigner(#[source] SignerError),

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

    /// Signature is missing.
    #[error("signature not provided")]
    SignatureMissing,

    /// Extra Certificate signature is missing for the given network.
    #[error("missing extra signature from {expected_signer} for the network {network_id}")]
    MissingExtraSignature {
        network_id: NetworkId,
        expected_signer: Address,
    },

    /// The extra signature is invalid.
    #[error("invalid extra signature: {0}")]
    InvalidExtraSignature(#[source] SignerError),

    /// The pessimistic proof signature is invalid.
    #[error("invalid pessimistic proof signature: {0}")]
    InvalidPessimisticProofSignature(#[source] SignerError),

    /// The multisig is invalid.
    #[error("invalid multisig: {0}")]
    InvalidMultisig(#[source] SignerError),

    /// The rollup contract (zkevm or aggchain base contract) fails to be
    /// retrieved from the L1.
    #[error("unable to retrieve the rollup contract for the network {network_id}: {source}")]
    UnableToRetrieveRollupContractAddress {
        source: L1RpcError,
        network_id: NetworkId,
    },

    /// The multisig context (signers or threshold) fails to be retrieved from
    /// the L1.
    #[error("unable to retrieve the multisig context for the network {network_id}: {source}")]
    UnableToRetrieveMultisigContext {
        source: L1RpcError,
        network_id: NetworkId,
    },
}

impl SignatureVerificationError {
    pub fn from_signer_error(e: agglayer_types::SignerError) -> Self {
        match e {
            agglayer_types::SignerError::Missing => Self::SignatureMissing,
            e @ agglayer_types::SignerError::Recovery(_) => Self::CouldNotRecoverCertSigner(e),
            e @ agglayer_types::SignerError::InvalidExtraSignature { .. } => {
                Self::InvalidExtraSignature(e)
            }
            e @ agglayer_types::SignerError::InvalidPessimisticProofSignature { .. } => {
                Self::InvalidPessimisticProofSignature(e)
            }
            e @ agglayer_types::SignerError::InvalidMultisig(_) => Self::InvalidMultisig(e),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ProofRetrievalError {
    #[error(transparent)]
    Storage(#[from] StorageError),

    #[error("Proof for certificate {certificate_id} not found")]
    NotFound { certificate_id: CertificateId },
}

#[derive(Debug, thiserror::Error)]
pub enum GetLatestCertificateError {
    #[error(transparent)]
    Storage(#[from] StorageError),

    #[error("Unknown latest certificate header for network {network_id}")]
    UnknownLatestCertificateHeader {
        network_id: NetworkId,
        source: Box<CertificateRetrievalError>,
    },

    #[error(
        "Mismatch on the certificate id. expected: {expected}, re-computed from the certificate \
         in DB: {got}"
    )]
    CertificateIdHashMismatch {
        expected: CertificateId,
        got: CertificateId,
    },

    #[error("Latest certificate header for certificate {certificate_id} not found")]
    NotFound { certificate_id: CertificateId },
}

#[derive(Debug, thiserror::Error)]
pub enum GetLatestSettledClaimError {
    #[error(transparent)]
    Storage(#[from] StorageError),

    #[error("Cound not get latest settled claim, inconsistent state for {network_id}")]
    InconsistentState {
        network_id: NetworkId,
        height: Height,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum GetNetworkStateError {
    #[error("Unable to determine network type for network {network_id}")]
    UnknownNetworkType { network_id: NetworkId },

    #[error("Could not get network status for network {network_id}, internal error: {source}")]
    InternalError {
        network_id: NetworkId,
        #[source]
        source: eyre::Error,
    },
}
