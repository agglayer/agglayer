use agglayer_types::primitives::SignatureError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Wrong bytes length: expected {expected}, got {actual}")]
    WrongBytesLength { expected: usize, actual: usize },

    #[error("Wrong vector length: expected {expected}, got {actual}")]
    WrongVectorLength { expected: usize, actual: usize },

    #[error("Missing required field: {0}")]
    MissingField(&'static str),

    #[error("Invalid leaf type {0}")]
    InvalidLeafType(i32),

    #[error("Invalid certificate status {0}")]
    InvalidCertificateStatus(i32),

    #[error("While parsing field {0}")]
    ParsingField(&'static str, #[source] Box<Error>),

    #[error("While parsing signature")]
    ParsingSignature(#[source] SignatureError),

    #[error("While deserializing SP1v4 proof")]
    DeserializingSp1v4Proof(#[source] bincode::Error),
}
