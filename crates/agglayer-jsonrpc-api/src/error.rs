//! Support for structured errors in RPC.

use agglayer_rpc::CertificateSubmissionError;
use agglayer_types::RpcErrorCode;
use jsonrpsee::types::error::ErrorObjectOwned;
use serde::Serialize;

/// Application-level RPC errors returned by AggLayer.
///
/// Implementation note:
/// RPC errors contain three pieces of information.
/// They are obtained as follows:
/// * `"code"` (the numeric error code) is taken from a call to [Self::code].
/// * `"message"` comes from the `Display` trait impl provided by `thiserror`.
/// * The `"data"` field comes from the `Serialize` trait impl.
#[derive(PartialEq, Eq, Serialize, Debug, Clone, thiserror::Error)]
#[serde(rename_all = "kebab-case")]
pub enum Error {
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Cannot send certificate: {detail}")]
    SendCertificate { detail: String },

    #[error("Resource not found: {0}")]
    ResourceNotFound(String),

    #[error("The {method} method is disabled")]
    MethodDisabled { method: &'static str },

    #[error("{message}")]
    #[serde(rename_all = "kebab-case")]
    Classified { code: RpcErrorCode, message: String },

    #[error("Internal error: {0}")]
    Internal(String),
}

impl Error {
    pub fn internal<S: Into<String>>(detail: S) -> Self {
        Self::Internal(detail.into())
    }

    /// Get the jsonrpc error code for this error.
    pub fn code(&self) -> i32 {
        match self {
            Self::InvalidArgument(_) => jsonrpsee::types::error::INVALID_PARAMS_CODE,
            Self::Internal(_) => jsonrpsee::types::error::INTERNAL_ERROR_CODE,
            Self::ResourceNotFound { .. } => RpcErrorCode::NotFound.code(),
            Self::SendCertificate { .. } => RpcErrorCode::SendCertificate.code(),
            Self::MethodDisabled { .. } => RpcErrorCode::MethodDisabled.code(),
            Self::Classified { code, .. } => code.code(),
        }
    }
}

impl From<CertificateSubmissionError> for Error {
    fn from(error: CertificateSubmissionError) -> Self {
        let detail = error.to_string();
        Self::SendCertificate { detail }
    }
}

impl From<agglayer_rpc::CertificateRetrievalError> for Error {
    fn from(err: agglayer_rpc::CertificateRetrievalError) -> Self {
        match err {
            agglayer_rpc::CertificateRetrievalError::Storage(error) => {
                Self::internal(error.to_string())
            }
            agglayer_rpc::CertificateRetrievalError::NotFound { certificate_id } => {
                Self::ResourceNotFound(format!("Certificate({certificate_id})"))
            }
        }
    }
}

impl From<agglayer_rpc::GetNetworkInfoError> for Error {
    fn from(err: agglayer_rpc::GetNetworkInfoError) -> Self {
        // Since NetworkStateRetrievalError is currently empty, convert to internal
        // error
        Self::internal(format!("Network state retrieval error: {err}"))
    }
}

// This impl establishes the integration with `jsonrpsee` errors.
impl From<Error> for ErrorObjectOwned {
    fn from(err: Error) -> Self {
        ErrorObjectOwned::owned(err.code(), err.to_string(), Some(err))
    }
}

/// Type returned from RPC methods, uses [RpcError].
pub type RpcResult<T> = Result<T, Error>;
