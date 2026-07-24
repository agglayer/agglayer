//! Support for structured errors in RPC.

use agglayer_rate_limiting::RateLimited as RateLimitedError;
use agglayer_rpc::CertificateSubmissionError;
use agglayer_types::RpcErrorCode;
use alloy::primitives::B256;
use jsonrpsee::types::error::ErrorObjectOwned;
use serde::Serialize;

use crate::service::{self, SendTxError, TxStatusError};

#[derive(PartialEq, Eq, Serialize, Debug, Clone, thiserror::Error)]
#[serde(rename_all = "kebab-case")]
pub enum ValidationError {
    #[error("Dry run failed")]
    DryRun { detail: String },

    #[error("State root verification failed")]
    RootVerification { detail: String },
}

impl ValidationError {
    fn dry_run(err: impl ToString) -> Self {
        let detail = err.to_string();
        Self::DryRun { detail }
    }
}

#[derive(PartialEq, Eq, Serialize, Debug, Clone, thiserror::Error)]
#[serde(rename_all = "kebab-case")]
pub enum SettlementError {
    #[error("No receipt")]
    NoReceipt,

    #[error("IO error")]
    IoError(String),

    #[error("Contract error")]
    Contract { detail: String },
}

impl SettlementError {
    fn io_error(err: impl ToString) -> Self {
        Self::IoError(err.to_string())
    }
}

#[derive(PartialEq, Eq, Serialize, Debug, Clone, thiserror::Error)]
#[serde(rename_all = "kebab-case")]
pub enum StatusError {
    #[error("Transaction not found")]
    TxNotFound { hash: B256 },

    #[error("Failed to get the current L1 block")]
    L1Block { detail: String },

    #[error("Failed to get tx status")]
    TxStatus { detail: String },
}

impl From<TxStatusError> for StatusError {
    fn from(error: TxStatusError) -> Self {
        use TxStatusError as E;
        match error {
            E::StatusCheck(error) => {
                let detail = error.to_string();
                Self::TxStatus { detail }
            }
            E::L1BlockRetrieval(error) => {
                let detail = error.to_string();
                Self::L1Block { detail }
            }
            E::TxNotFound { hash } => Self::TxNotFound { hash },
        }
    }
}

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

    #[error("Rollup {rollup_id} not registered")]
    #[serde(rename_all = "kebab-case")]
    RollupNotRegistered { rollup_id: u32 },

    #[error("Rollup signature verification failed")]
    #[serde(rename_all = "kebab-case")]
    SignatureMismatch { detail: String },

    #[error("Validation failed: {0}")]
    Validation(#[from] ValidationError),

    #[error("L1 settlement error: {0}")]
    Settlement(#[from] SettlementError),

    #[error("Status retrieval error: {0}")]
    Status(#[from] StatusError),

    #[error("Cannot send certificate: {detail}")]
    SendCertificate { detail: String },

    #[error("Rate limited")]
    #[serde(rename_all = "kebab-case")]
    RateLimited {
        detail: String,
        error: RateLimitedError,
    },

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
            Self::RollupNotRegistered { .. } => RpcErrorCode::RollupNotRegistered.code(),
            Self::SignatureMismatch { .. } => RpcErrorCode::SignatureMismatch.code(),
            Self::Validation(_) => RpcErrorCode::ValidationFailure.code(),
            Self::Settlement(_) => RpcErrorCode::SettlementError.code(),
            Self::Status(_) => RpcErrorCode::StatusError.code(),
            Self::SendCertificate { .. } => RpcErrorCode::SendCertificate.code(),
            Self::RateLimited { .. } => RpcErrorCode::RateLimited.code(),
            Self::MethodDisabled { .. } => RpcErrorCode::MethodDisabled.code(),
            Self::Classified { code, .. } => code.code(),
        }
    }
}

impl From<SendTxError> for Error {
    fn from(err: SendTxError) -> Self {
        use SendTxError as E;
        match err {
            E::RateLimited(error) => error.into(),
            E::SignatureError(error) => {
                let detail = error.to_string();
                Self::SignatureMismatch { detail }
            }
            E::RollupNotRegistered { rollup_id } => Self::RollupNotRegistered { rollup_id },
            E::DryRunZkEvm(error) => ValidationError::DryRun {
                detail: format!("PolygonZkEvm contract error: {error:?}"),
            }
            .into(),
            E::DryRunRollupManager(error) => ValidationError::DryRun {
                detail: format!("PolygonRollupManager contract error {error:?}"),
            }
            .into(),
            E::DryRunOther(error) => ValidationError::dry_run(error).into(),
            E::RootVerification(error) => {
                let detail = error.to_string();
                ValidationError::RootVerification { detail }.into()
            }
            E::Settlement(error) => error.into(),
        }
    }
}

impl From<RateLimitedError> for Error {
    fn from(error: RateLimitedError) -> Self {
        let detail = error.to_string();
        Self::RateLimited { detail, error }
    }
}

impl From<service::error::SettlementError> for Error {
    fn from(error: service::error::SettlementError) -> Self {
        use service::error::SettlementError as E;
        match error {
            E::NoReceipt => SettlementError::NoReceipt.into(),
            E::ContractError(error) => {
                let detail = error.to_string();
                SettlementError::Contract { detail }.into()
            }
            E::ProviderError(error) => SettlementError::io_error(error).into(),
            E::RateLimited(error) => error.into(),
            error @ E::Timeout(_) => SettlementError::io_error(error).into(),
            E::PendingTransactionError(error) => SettlementError::io_error(error).into(),
            error @ E::ReceiptWithoutBlockNumberError(_) => SettlementError::io_error(error).into(),
            error @ E::InternalError(_) => SettlementError::io_error(error).into(),
        }
    }
}

impl From<TxStatusError> for Error {
    fn from(error: TxStatusError) -> Self {
        StatusError::from(error).into()
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
