//! Error types for the top-level Agglayer service.

use agglayer_contracts::{
    polygon_rollup_manager::PolygonRollupManagerErrors, polygon_zk_evm::PolygonZkEvmErrors,
};
use agglayer_rate_limiting::resource::SendTxRateLimited as SendTxRateLimited;
use agglayer_rpc::error::SignatureVerificationError;
pub use agglayer_types::Digest;
use ethers::{contract::ContractError, providers::Middleware, types::H256};

pub use crate::kernel::{CheckTxStatusError, SettlementError, ZkevmNodeVerificationError};

#[derive(Debug, thiserror::Error)]
pub enum CertificateRetrievalError {
    #[error("Data for certificate {certificate_id} not found")]
    NotFound { certificate_id: Digest },
}

#[derive(Debug, thiserror::Error)]
pub enum TxStatusError<Rpc: 'static + Middleware> {
    #[error("Status retrieval error: {0}")]
    StatusCheck(CheckTxStatusError<Rpc>),

    #[error("Failed to get L1 block: {0}")]
    L1BlockRetrieval(CheckTxStatusError<Rpc>),

    #[error("Transaction {hash} not found")]
    TxNotFound { hash: H256 },
}

#[derive(Debug, thiserror::Error)]
pub enum SendTxError<Rpc: 'static + Middleware> {
    #[error(transparent)]
    RateLimited(#[from] SendTxRateLimited),

    #[error(transparent)]
    SignatureError(#[from] SignatureVerificationError<Rpc>),

    #[error("Rollup {rollup_id} not registered")]
    RollupNotRegistered { rollup_id: u32 },

    #[error("Zkevm error during dry run: {0:?}")]
    DryRunZkEvm(PolygonZkEvmErrors),

    #[error("Rollup manager error during dry run: {0:?}")]
    DryRunRollupManager(PolygonRollupManagerErrors),

    #[error("Error during dry run: {0}")]
    DryRunOther(ContractError<Rpc>),

    #[error("Failed to verify local exit root or state root: {0}")]
    RootVerification(ZkevmNodeVerificationError),

    #[error("Settlement failed: {0}")]
    Settlement(SettlementError<Rpc>),
}

impl<Rpc: 'static + Middleware> SendTxError<Rpc> {
    /// Decode the dry run contract errors.
    pub fn dry_run(err: &ContractError<Rpc>) -> Self {
        err.decode_contract_revert::<PolygonZkEvmErrors>()
            .map(Self::DryRunZkEvm)
            .or_else(|| {
                err.decode_contract_revert::<PolygonRollupManagerErrors>()
                    .map(Self::DryRunRollupManager)
            })
            .unwrap_or_else(|| Self::dry_run(err))
    }
}

impl<Rpc: Middleware> From<SettlementError<Rpc>> for SendTxError<Rpc> {
    fn from(err: SettlementError<Rpc>) -> Self {
        match err {
            SettlementError::RateLimited(e) => e.into(),
            e => Self::Settlement(e),
        }
    }
}
