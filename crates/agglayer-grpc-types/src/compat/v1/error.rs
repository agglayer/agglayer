use agglayer_types::primitives::SignatureError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("wrong bytes length: expected {expected}, got {actual}")]
    WrongBytesLength { expected: usize, actual: usize },

    #[error("wrong vector length: expected {expected}, got {actual}")]
    WrongVectorLength { expected: usize, actual: usize },

    #[error("missing required field: {0}")]
    MissingField(&'static str),

    #[error("invalid leaf type {0}")]
    InvalidLeafType(i32),

    #[error("invalid certificate status {0}")]
    InvalidCertificateStatus(i32),

    #[error("failed parsing field {0}")]
    ParsingField(&'static str, #[source] Box<Error>),

    #[error("failed parsing signature")]
    ParsingSignature(#[source] SignatureError),

    #[error("failed deserializing SP1v4 proof")]
    DeserializingProof(#[source] bincode::Error),

    #[error("failed serializing SP1v4 proof")]
    SerializingProof(#[source] bincode::Error),
}
