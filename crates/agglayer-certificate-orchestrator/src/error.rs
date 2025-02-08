use agglayer_types::{CertificateId, CertificateStatusError, Height, NetworkId};
use pessimistic_proof::{error::ProofVerificationError, ProofError};

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
    #[error("internal error: {0}")]
    InternalError(String),

    #[error("The status of the certificate is invalid")]
    InvalidCertificateStatus,

    #[error("The certificate header is not found")]
    NotFoundCertificateHeader,

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
    L1CommunicationError(String),
}

impl From<Error> for CertificateStatusError {
    fn from(value: Error) -> Self {
        match value {
            Error::L1CommunicationError(error) => CertificateStatusError::InternalError(error),
            Error::Clock(error) => CertificateStatusError::InternalError(error.to_string()),
            Error::PreCertification(pre_certification_error) => {
                CertificateStatusError::PreCertificationError(pre_certification_error.to_string())
            }
            Error::Certification(certification_error) => {
                CertificateStatusError::CertificationError(certification_error.to_string())
            }
            Error::Storage(error) => CertificateStatusError::InternalError(error.to_string()),
            Error::InternalError(error) => CertificateStatusError::InternalError(error),
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
