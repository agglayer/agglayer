use agglayer_contracts::L1RpcError;
use agglayer_types::{
    aggchain_proof::AggchainProofPublicValues, CertificateId, CertificateStatusError, Digest,
    Height, NetworkId,
};
use pessimistic_proof::{
    core::commitment::StateCommitment, error::ProofVerificationError, PessimisticProofOutput,
    ProofError,
};

#[derive(thiserror::Error, Debug)]
pub enum PreCertificationError {
    #[error("Storage error: {0}")]
    Storage(#[from] agglayer_storage::error::Error),

    #[error("proof already exists for network {0} at height {1} for certificate {2}")]
    ProofAlreadyExists(NetworkId, Height, CertificateId),
}

#[derive(thiserror::Error, Debug)]
pub enum CertificationError {
    #[error("certificate not found for network {0} at height {1}")]
    CertificateNotFound(NetworkId, Height),
    #[error(
        "Failed to retrieve the trusted sequencer address for network {0} during proving phase"
    )]
    TrustedSequencerNotFound(NetworkId),
    #[error("Failed to retrieve the last pessimistic root for network {0}")]
    LastPessimisticRootNotFound(NetworkId),
    #[error("Failed to retrieve the l1 info root for the l1 leaf count: {1} for certificate {0}")]
    L1InfoRootNotFound(CertificateId, u32),
    #[error("proof verification failed")]
    ProofVerificationFailed { source: ProofVerificationError },

    /// Rust native execution without aggchain proof stark verification failed
    /// on the given error.
    #[error("rust-native execution failed: {source:?}")]
    NativeExecutionFailed { source: ProofError },

    /// SP1 native execute call which includes the aggchain proof stark
    /// verification failed.
    #[error("sp1-native execution failed.")]
    Sp1ExecuteFailed(#[source] anyhow::Error),

    /// The PP public values differ between the ones computed during the
    /// rust native execution, and the ones computed by the sp1 zkvm execution.
    #[error(
        "Mismatch on the PP public values between rust native execution and sp1 zkvm execution. \
         {native_execution:?}. sp1 zkvm execution: {sp1_zkvm_execution:?}"
    )]
    MismatchPessimisticProofPublicValues {
        native_execution: Box<PessimisticProofOutput>,
        sp1_zkvm_execution: Box<PessimisticProofOutput>,
    },

    #[error("Type error: {source}")]
    Types { source: agglayer_types::Error },
    #[error("Serialize error")]
    Serialize { source: bincode::Error },
    #[error("Deserialize error")]
    Deserialize { source: bincode::Error },
    #[error("internal error: {0}")]
    InternalError(String),
    #[error("prover failed")]
    ProverFailed(String),
    #[error("prover returned unspecified error")]
    ProverReturnedUnspecifiedError,
    #[error("prover execution failed")]
    ProverExecutionFailed { source: ProofError },
    #[error("Storage error: {0}")]
    Storage(#[from] agglayer_storage::error::Error),
    #[error("rollup contract address not found")]
    RollupContractAddressNotFound(#[source] L1RpcError),
    #[error("Unable to find aggchain vkey")]
    UnableToFindAggchainVkey { source: L1RpcError },
    #[error("Aggchain proof vkey mismatch: expected {expected}, actual {actual}")]
    AggchainProofVkeyMismatch { expected: String, actual: String },
    #[error("Missing L1 info tree leaf count for generic aggchain data")]
    MissingL1InfoTreeLeafCountForGenericAggchainData,
    #[error("Unable to find aggchain hash")]
    UnableToFindAggchainHash(#[source] L1RpcError),
    /// Mismatch on the aggchain hash between the one fetched from the L1, and
    /// the one computed from the received Certificate.
    #[error("Aggchain hash mismatch. from l1: {from_l1}, from certificate: {from_certificate}.")]
    AggchainHashMismatch {
        from_l1: Digest,
        from_certificate: Digest,
    },
    /// Target state commitment mismatch between witness generation and native
    /// execution.
    #[error(
        "Target state commitment mismatch. after witness generation: {witness_generation:?}, \
         after native execution: {native_execution:?}"
    )]
    StateCommitmentMismatch {
        witness_generation: Box<StateCommitment>,
        native_execution: Box<StateCommitment>,
    },
    /// Aggchain proof public values mismatch between PP witness and the ones
    /// expected by the received aggchain proof.
    #[error(
        "Aggchain proof public values mismatch. expected by the PP: {from_witness:?}, expected by \
         the aggchain proof: {from_proof:?}"
    )]
    AggchainProofPublicValuesMismatch {
        from_proof: Box<AggchainProofPublicValues>,
        from_witness: Box<AggchainProofPublicValues>,
    },
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Clock error: {0}")]
    Clock(#[from] agglayer_clock::Error),

    #[error(transparent)]
    PreCertification(#[from] PreCertificationError),

    #[error(transparent)]
    Certification(#[from] CertificationError),

    #[error("Storage error: {0}")]
    Storage(#[from] agglayer_storage::error::Error),

    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("The status of the certificate is invalid")]
    InvalidCertificateStatus,

    #[error("The certificate header is not found")]
    NotFoundCertificateHeader,

    #[error("Unable to get verifier type for network")]
    UnableToGetVerifierType {
        certificate_id: CertificateId,
        network_id: NetworkId,
    },

    #[error("Failed to settle the certificate {certificate_id}: {error}")]
    SettlementError {
        certificate_id: CertificateId,
        error: String,
    },

    #[error("Failed to persist the state after {certificate_id}: {error}")]
    PersistenceError {
        certificate_id: CertificateId,
        error: String,
    },

    #[error("Failed to communicate with L1: {0}")]
    L1CommunicationError(#[source] agglayer_contracts::L1RpcError),
}

impl From<Error> for CertificateStatusError {
    fn from(value: Error) -> Self {
        match value {
            Error::L1CommunicationError(error) => {
                CertificateStatusError::InternalError(error.to_string())
            }
            Error::Clock(error) => CertificateStatusError::InternalError(error.to_string()),
            Error::PreCertification(pre_certification_error) => {
                CertificateStatusError::PreCertificationError(pre_certification_error.to_string())
            }
            Error::Certification(certification_error) => {
                CertificateStatusError::CertificationError(certification_error.to_string())
            }
            Error::Storage(error) => CertificateStatusError::InternalError(error.to_string()),
            Error::InternalError(error) => CertificateStatusError::InternalError(error),
            Error::UnableToGetVerifierType { network_id, .. } => {
                CertificateStatusError::InternalError(format!(
                    "Unable to get verifier type for NetworkId {network_id}"
                ))
            }
            Error::InvalidCertificateStatus => {
                CertificateStatusError::InternalError("InvalidCertificateStatus".to_string())
            }
            Error::NotFoundCertificateHeader => {
                CertificateStatusError::InternalError("NotFoundCertificateHeader".to_string())
            }
            Error::SettlementError { error, .. } => CertificateStatusError::SettlementError(error),
            Error::PersistenceError { error, .. } => {
                CertificateStatusError::InternalError(error.to_string())
            }
        }
    }
}
