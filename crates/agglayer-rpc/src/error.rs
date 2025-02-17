//! Error types for the top-level Agglayer service.

pub use agglayer_storage::error::Error as StorageError;
pub use agglayer_types::Digest;

pub use crate::rate_limiting::RateLimited as RateLimitedError;

#[derive(Debug, thiserror::Error)]
pub enum CertificateRetrievalError {
    #[error(transparent)]
    Storage(#[from] StorageError),

    #[error("Data for certificate {certificate_id} not found")]
    NotFound { certificate_id: Digest },
}

#[derive(Debug, thiserror::Error)]
pub enum CertificateSubmissionError {
    #[error(transparent)]
    Storage(#[from] StorageError),

    #[error("Failed to send the certificate to the orchestrator")]
    OrchestratorNotResponsive,

    #[error("Failed to validate certificate signature: {0}")]
    SignatureError(#[source] alloy_primitives::SignatureError),
}
