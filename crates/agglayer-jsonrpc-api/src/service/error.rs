//! Error types for the top-level Agglayer service.

use agglayer_contracts::contracts::{
    PolygonRollupManager::PolygonRollupManagerErrors, PolygonZkEvm::PolygonZkEvmErrors,
};
use agglayer_rate_limiting::RateLimited as RateLimitedError;
use agglayer_rpc::error::SignatureVerificationError;
pub use agglayer_types::Digest;
use alloy::{contract::Error as ContractError, primitives::B256};

pub use crate::kernel::{CheckTxStatusError, SettlementError, ZkevmNodeVerificationError};

#[derive(Debug, thiserror::Error)]
pub enum CertificateRetrievalError {
    #[error("Data for certificate {certificate_id} not found")]
    NotFound { certificate_id: Digest },
}

#[derive(Debug, thiserror::Error)]
pub enum TxStatusError {
    #[error("Status retrieval error: {0}")]
    StatusCheck(CheckTxStatusError),

    #[error("Failed to get L1 block: {0}")]
    L1BlockRetrieval(CheckTxStatusError),

    #[error("Transaction {hash} not found")]
    TxNotFound { hash: B256 },
}

#[derive(Debug, thiserror::Error)]
pub enum SendTxError {
    #[error("Rate limited: {0}")]
    RateLimited(#[from] RateLimitedError),

    #[error(transparent)]
    SignatureError(#[from] SignatureVerificationError),

    #[error("Rollup {rollup_id} not registered")]
    RollupNotRegistered { rollup_id: u32 },

    #[error("Zkevm error during dry run")]
    DryRunZkEvm(PolygonZkEvmErrors),

    #[error("Rollup manager error during dry run")]
    DryRunRollupManager(PolygonRollupManagerErrors),

    #[error("Error during dry run: {0}")]
    DryRunOther(ContractError),

    #[error("Failed to verify local exit root or state root: {0}")]
    RootVerification(ZkevmNodeVerificationError),

    #[error("Settlement failed: {0}")]
    Settlement(SettlementError),
}

impl SendTxError {
    /// Decode the dry run contract errors.
    pub fn dry_run(err: ContractError) -> Self {
        // Note: In alloy, contract error decoding is handled differently
        // This is a simplified version and may need adjustment based on actual contract
        // error handling
        Self::DryRunOther(err)
    }
}

impl From<SettlementError> for SendTxError {
    fn from(err: SettlementError) -> Self {
        match err {
            SettlementError::RateLimited(e) => e.into(),
            e => Self::Settlement(e),
        }
    }
}
