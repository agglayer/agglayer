use agglayer_types::{CertificateId, CertificateStatus, Height, NetworkId, ProofVerificationError};
use pessimistic_proof::ProofError;

#[derive(thiserror::Error, Debug)]
pub enum InitialCheckError {
    #[error("Storage error: {0}")]
    Storage(#[from] agglayer_storage::error::Error),

    #[error("Certificate submission failed")]
    CertificateSubmission,

    #[error("Certificate is in the past (height {height}, expecting {next_height})")]
    InPast { height: u64, next_height: u64 },

    #[error("Certificate is too far in the future (height {height}, max allowed {max_height})")]
    FarFuture { height: u64, max_height: u64 },

    #[error("Cannot replace an existing {status} certificate")]
    IllegalReplacement { status: CertificateStatus },

    #[error("Internal error")]
    Internal,
}

#[derive(thiserror::Error, Debug)]
pub enum PreCertificationError {
    #[error("Storage error: {0}")]
    Storage(#[from] agglayer_storage::error::Error),

    #[error("certificate not found for network {0} at height {1}")]
    CertificateNotFound(NetworkId, Height),
    #[error("proof already exists for network {0} at height {1} for certificate {2}")]
    ProofAlreadyExists(NetworkId, Height, CertificateId),
}

#[derive(thiserror::Error, Debug)]
pub enum CertificationError {
    #[error(
        "Failed to retrieve the trusted sequencer address for network {0} during proving phase"
    )]
    TrustedSequencerNotFound(NetworkId),
    #[error("Failed to retrieve the l1 info root for the l1 leaf count: {1} for certificate {0}")]
    L1InfoRootNotFound(CertificateId, u32),
    #[error("proof verification failed")]
    ProofVerificationFailed { source: ProofVerificationError },
    #[error("prover execution failed: {source}")]
    ProverExecutionFailed { source: ProofError },
    #[error("native execution failed: {source:?}")]
    NativeExecutionFailed { source: ProofError },
    #[error("Type error: {source}")]
    Types { source: agglayer_types::Error },
    #[error("Serialize error")]
    Serialize { source: bincode::Error },
    #[error("Deserialize error")]
    Deserialize { source: bincode::Error },
    #[error("internal error: {0}")]
    InternalError(String),
    #[error("Storage error: {0}")]
    Storage(#[from] agglayer_storage::error::Error),
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
    #[error("internal error")]
    InternalError,

    #[error("The status of the certificate is invalid")]
    InvalidCertificateStatus,

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
}
