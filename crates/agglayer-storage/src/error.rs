use agglayer_types::{EpochNumber, Height, NetworkId};

use crate::storage::DBError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("DB error: {0}")]
    DBError(#[from] DBError),

    #[error(r#"An unexpected error occurred: {0}
        This is a critical bug that needs to be reported on `https://github.com/agglayer/agglayer/issues`"#)]
    Unexpected(String),

    #[error("No certificate found")]
    NoCertificate,

    #[error("No proof found")]
    NoProof,

    #[error("The store is already in packing mode")]
    AlreadyInPackingMode,

    #[error("The epoch {0} is already finished")]
    AlreadyPacked(EpochNumber),

    #[error(transparent)]
    CertificateCandidateError(#[from] CertificateCandidateError),

    #[error("Unprocessed action: {0}")]
    UnprocessedAction(String),

    #[error("Inconsistent state for network: {network_id}")]
    InconsistentState { network_id: NetworkId },

    #[error("Inconsistent frontier")]
    InconsistentFrontier,

    #[error("Wrong value type")]
    WrongValueType,

    #[error("Smt node not found")]
    SmtNodeNotFound,
}

#[derive(Debug, thiserror::Error)]
pub enum CertificateCandidateError {
    #[error("Invalid certificate candidate for network {0} at height {1} for current epoch")]
    Invalid(NetworkId, Height),

    #[error(
        "Invalid certificate candidate for network {0}: {1} wasn't expected, current height {2}"
    )]
    UnexpectedHeight(NetworkId, Height, Height),
}
