//! Support for structured errors in RPC.

use ethers::{middleware::Middleware, types::H256};
use jsonrpsee::types::error::ErrorObjectOwned;
use serde::Serialize;

use crate::{
    kernel::{CheckTxStatusError, SignatureVerificationError, ZkevmNodeVerificationError},
    rate_limiting,
};

/// RPC error codes.
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

    // Note we skip -10006 here which is reserved for certificate submission.

    /// Transaction settlement has been rate limited.
    pub const RATE_LIMITED: i32 = -10007;
}

#[derive(PartialEq, Eq, Serialize, Debug, Clone, thiserror::Error)]
#[serde(rename_all = "kebab-case")]
pub enum ValidationError {
    #[error("Dry run failed")]
    DryRun { detail: String },

    #[error("State root verification failed")]
    RootVerification { detail: String },
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

impl StatusError {
    pub(crate) fn tx_not_found(hash: H256) -> Self {
        Self::TxNotFound { hash }
    }

    pub(crate) fn tx_status<R: Middleware>(err: CheckTxStatusError<R>) -> Self {
        let detail = err.to_string();
        Self::TxStatus { detail }
    }

    pub(crate) fn l1_block<R: Middleware>(err: CheckTxStatusError<R>) -> Self {
        let detail = err.to_string();
        Self::L1Block { detail }
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

    #[error("Rate limited")]
    #[serde(rename_all = "kebab-case")]
    RateLimited {
        detail: String,
        error: rate_limiting::Error,
    },
}

impl Error {
    pub(crate) fn rollup_not_registered(rollup_id: u32) -> Self {
        Self::RollupNotRegistered { rollup_id }
    }

    pub(crate) fn signature_mismatch<R: Middleware>(err: SignatureVerificationError<R>) -> Self {
        let detail = err.to_string();
        Self::SignatureMismatch { detail }
    }

    pub(crate) fn dry_run(detail: String) -> Self {
        ValidationError::DryRun { detail }.into()
    }

    pub(crate) fn root_verification(err: ZkevmNodeVerificationError) -> Self {
        let detail = err.to_string();
        ValidationError::RootVerification { detail }.into()
    }

    pub(crate) fn settlement<R: Middleware>(err: crate::kernel::SettlementError<R>) -> Self {
        err.into()
    }

    /// Get the jsonrpc error code for this error.
    pub fn code(&self) -> i32 {
        match self {
            Self::RollupNotRegistered { .. } => code::ROLLUP_NOT_REGISTERED,
            Self::SignatureMismatch { .. } => code::SIGNATURE_MISMATCH,
            Self::Validation(_) => code::VALIDATION_FAILURE,
            Self::Settlement(_) => code::SETTLEMENT_ERROR,
            Self::Status(_) => code::STATUS_ERROR,
            Self::RateLimited { .. } => code::RATE_LIMITED,
        }
    }
}

impl<R: Middleware> From<crate::kernel::SettlementError<R>> for Error {
    fn from(err: crate::kernel::SettlementError<R>) -> Self {
        use crate::kernel::SettlementError as E;
        match err {
            E::NoReceipt => SettlementError::NoReceipt.into(),
            E::ProviderError(e) => SettlementError::IoError(e.to_string()).into(),
            E::ContractError(e) => {
                let detail = e.to_string();
                SettlementError::Contract { detail }.into()
            }
            E::RateLimited(e) => e.into(),
        }
    }
}

impl From<rate_limiting::Error> for Error {
    fn from(error: rate_limiting::Error) -> Self {
        let detail = error.to_string();
        Self::RateLimited { detail, error }
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
