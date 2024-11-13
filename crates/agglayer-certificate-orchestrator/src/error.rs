use agglayer_types::{CertificateId, Height, NetworkId, ProofVerificationError};
use pessimistic_proof::ProofError;

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
    // #[error("certificate not found for network {0} at height {1}")]
    // CertificateNotFound(NetworkId, Height),
    // #[error("proof already exists for network {0} at height {1} for certificate {2}")]
    // ProofAlreadyExists(NetworkId, Height, CertificateId),
    #[error(
        "Failed to retrieve the trusted sequencer address for network {1} during proving phase \
         for {0}"
    )]
    TrustedSequencerNotFound(CertificateId, NetworkId),
    #[error("Failed to retrieve the l1 info root for the l1 leaf count: {1} for certificate {0}")]
    L1InfoRootNotFound(CertificateId, u32),
    #[error("proof verification failed")]
    ProofVerificationFailed {
        certificate_id: CertificateId,
        source: ProofVerificationError,
    },
    #[error("prover execution failed: {source}")]
    ProverExecutionFailed {
        certificate_id: CertificateId,
        source: ProofError,
    },
    #[error("native execution failed: {source:?}")]
    NativeExecutionFailed {
        certificate_id: CertificateId,
        source: ProofError,
    },
    #[error("Type error: {source}")]
    Types {
        certificate_id: CertificateId,
        source: agglayer_types::Error,
    },
    #[error("Serialize error")]
    Serialize {
        certificate_id: CertificateId,
        source: bincode::Error,
    },
    #[error("Deserialize error")]
    Deserialize {
        certificate_id: CertificateId,
        source: bincode::Error,
    },
    #[error("internal error")]
    InternalError,
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
}
