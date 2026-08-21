use agglayer_types::{
    CertificateId, CertificateStatusError, EpochNumber, Height, NetworkId, SettlementJobId,
};

use crate::storage::{DBError, DBOpenError};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("DB error: {0}")]
    DBError(#[from] DBError),

    #[error("DB open error: {0}")]
    DBOpenError(DBOpenError),

    #[error(r#"An unexpected error occurred: {0}
        This is a critical bug that needs to be reported on `https://github.com/agglayer/agglayer/issues`"#)]
    Unexpected(String),

    #[error("No certificate found")]
    NoCertificate,

    #[error("No certificate header found")]
    NoCertificateHeader,

    #[error("No proof found")]
    NoProof,

    #[error("Unreadable proof for certificate {id}: {source}")]
    UnreadableProof {
        id: CertificateId,
        #[source]
        source: DBError,
    },

    #[error("The store is already in packing mode")]
    AlreadyInPackingMode,

    #[error("The epoch {0} is already finished")]
    AlreadyPacked(EpochNumber),

    #[error(transparent)]
    CertificateCandidateError(#[from] CertificateCandidateError),

    #[error("Unprocessed action: {0}")]
    UnprocessedAction(String),

    #[error("Settlement job {0} does not exist")]
    SettlementJobNotFound(SettlementJobId),

    #[error(
        "Settlement job {0} already has a terminal result; pass force to edit its attempts anyway"
    )]
    SettlementJobAlreadyCompleted(SettlementJobId),

    #[error("Settlement job {0} has no terminal result to remove")]
    SettlementJobNotCompleted(SettlementJobId),

    #[error("Settlement attempt {attempt} does not exist for job {job}")]
    SettlementAttemptNotFound { job: SettlementJobId, attempt: u64 },

    #[error("No result is recorded for settlement attempt {attempt} of job {job}")]
    SettlementAttemptResultNotRecorded { job: SettlementJobId, attempt: u64 },

    #[error("Inconsistent state for network: {network_id}")]
    InconsistentState { network_id: NetworkId },

    #[error("Inconsistent frontier")]
    InconsistentFrontier,

    #[error("Wrong value type")]
    WrongValueType,

    #[error("Smt node not found")]
    SmtNodeNotFound,

    #[error(transparent)]
    SettlementCompat(#[from] crate::types::settlement::compat::Error),

    #[error(
        "Invalid pending certificate height for network {0}: attempted to insert height {1}, but \
         latest pending height is {2}"
    )]
    InvalidPendingHeight(NetworkId, Height, Height),

    #[error("Certificate height {0} exceeds the range of the native Prometheus height gauge")]
    NetworkMetricHeightOutOfRange(Height),
}

impl From<Error> for CertificateStatusError {
    fn from(error: Error) -> Self {
        CertificateStatusError::InternalError(format!("{error:?}"))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CertificateCandidateError {
    #[error("Invalid certificate candidate for network {0} at height {1} for current epoch")]
    Invalid(NetworkId, Height),

    #[error(
        "Invalid certificate candidate for network {0}: {1} wasn't expected, current height {2}"
    )]
    UnexpectedHeight(NetworkId, Height, Height),

    #[error("Invalid certificate candidate for network {0}: {1} wasn't expected")]
    InconsistentCertificateContext(NetworkId, CertificateId),
}
