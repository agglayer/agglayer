//! Support for structured errors in RPC.

use ethers::{middleware::Middleware, types::H256};
use jsonrpsee::types::error::ErrorObjectOwned;
use serde::Serialize;

use crate::{
    rate_limiting::RateLimited as RateLimitedError,
    service::{
        self, CertificateRetrievalError, CertificateSubmissionError, SendTxError, TxStatusError,
    },
};

/// JsonRPC error codes.
pub mod code {
    /// Rollup is not registered.
    pub const ROLLUP_NOT_REGISTERED: i32 = -10001;

    /// Rollup signature verification failure.
    pub const SIGNATURE_MISMATCH: i32 = -10002;

    /// Proof or state validation failed.
    pub const VALIDATION_FAILURE: i32 = -10003;

    /// L1 settlement failure.
    pub const SETTLEMENT_ERROR: i32 = -10004;

    /// Transaction status retrieval error.
    pub const STATUS_ERROR: i32 = -10005;

    /// Error submitting a certificate.
    pub const SEND_CERTIFICATE: i32 = -10006;

    /// Transaction settlement has been rate limited.
    pub const RATE_LIMITED: i32 = -10007;

    /// Resource not found.
    pub const RESOURCE_NOT_FOUND: i32 = -10008;
}

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
    TxNotFound { hash: H256 },

    #[error("Failed to get the current L1 block")]
    L1Block { detail: String },

    #[error("Failed to get tx status")]
    TxStatus { detail: String },
}

impl<Rpc: 'static + Middleware> From<TxStatusError<Rpc>> for StatusError {
    fn from(error: TxStatusError<Rpc>) -> Self {
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
            Self::Internal(_) => jsonrpsee::types::error::INTERNAL_ERROR_CODE,
            Self::ResourceNotFound { .. } => code::RESOURCE_NOT_FOUND,
            Self::RollupNotRegistered { .. } => code::ROLLUP_NOT_REGISTERED,
            Self::SignatureMismatch { .. } => code::SIGNATURE_MISMATCH,
            Self::Validation(_) => code::VALIDATION_FAILURE,
            Self::Settlement(_) => code::SETTLEMENT_ERROR,
            Self::Status(_) => code::STATUS_ERROR,
            Self::SendCertificate { .. } => code::SEND_CERTIFICATE,
            Self::RateLimited { .. } => code::RATE_LIMITED,
        }
    }
}

impl<Rpc: 'static + Middleware> From<SendTxError<Rpc>> for Error {
    fn from(err: SendTxError<Rpc>) -> Self {
        use SendTxError as E;
        match err {
            E::RateLimited(error) => error.into(),
            E::SignatureError(error) => {
                let detail = error.to_string();
                Self::SignatureMismatch { detail }
            }
            E::RollupNotRegistered { rollup_id } => Self::RollupNotRegistered { rollup_id },
            E::DryRunZkEvm(error) => ValidationError::dry_run(error).into(),
            E::DryRunRollupManager(error) => ValidationError::dry_run(error).into(),
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

impl<Rpc: 'static + Middleware> From<service::error::SettlementError<Rpc>> for Error {
    fn from(error: service::error::SettlementError<Rpc>) -> Self {
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
        }
    }
}

impl<Rpc: 'static + Middleware> From<TxStatusError<Rpc>> for Error {
    fn from(error: TxStatusError<Rpc>) -> Self {
        StatusError::from(error).into()
    }
}

impl<Rpc: Middleware> From<CertificateSubmissionError<Rpc>> for Error {
    fn from(error: CertificateSubmissionError<Rpc>) -> Self {
        let detail = error.to_string();
        Self::SendCertificate { detail }
    }
}

impl From<CertificateRetrievalError> for Error {
    fn from(err: CertificateRetrievalError) -> Self {
        match err {
            CertificateRetrievalError::Storage(error) => Self::internal(error.to_string()),
            CertificateRetrievalError::NotFound { certificate_id } => {
                Self::ResourceNotFound(format!("Certificate({certificate_id})"))
            }
        }
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
