//! Agglayer smart-contract bindings.

use std::sync::Arc;

use ethers::prelude::*;
use ethers::providers::Middleware;
use tracing::error;

#[rustfmt::skip]
#[allow(warnings)]
#[path = "contracts/aggchain_base.rs"]
pub mod aggchain_base;

#[rustfmt::skip]
#[allow(warnings)]
#[path = "contracts/agglayer_gateway.rs"]
pub mod agglayer_gateway;


#[rustfmt::skip]
#[allow(warnings)]
#[path = "contracts/polygon_rollup_manager.rs"]
pub mod polygon_rollup_manager;

#[rustfmt::skip]
#[allow(warnings)]
#[path = "contracts/polygon_zk_evm.rs"]
pub mod polygon_zk_evm;

#[rustfmt::skip]
#[allow(warnings)]
#[path = "contracts/polygon_zkevm_global_exit_root_v2.rs"]
pub mod polygon_zkevm_global_exit_root_v2;

pub mod aggchain;
pub mod rollup;
pub mod settler;

pub use aggchain::AggchainContract;
pub use rollup::RollupContract;
pub use settler::Settler;

#[async_trait::async_trait]
pub trait L1TransactionFetcher {
    async fn fetch_transaction_receipt(
        &self,
        tx_hash: H256,
    ) -> Result<TransactionReceipt, L1RpcError>;
}

pub struct L1RpcClient<RpcProvider> {
    rpc: Arc<RpcProvider>,
    inner: polygon_rollup_manager::PolygonRollupManager<RpcProvider>,
    l1_info_tree: polygon_zkevm_global_exit_root_v2::PolygonZkEVMGlobalExitRootV2<RpcProvider>,
}

#[derive(thiserror::Error, Debug)]
pub enum L1RpcInitializationError {
    #[error("Unable to get the InitL1InfoRootMap: {0}")]
    InitL1InfoRootMapEventNotFound(String),
    #[error("Event InitL1InfoRootMap returned null value for L1 info root, leaf count: {0}")]
    InvalidL1InfoRootFromEvent(u32),
}

#[derive(thiserror::Error, Debug)]
pub enum L1RpcError {
    #[error("Failed to get the `UpdateL1InfoTreeV2` events: {0}")]
    UpdateL1InfoTreeV2EventFailure(String),
    #[error("Unable to find `UpdateL1InfoTreeV2` events")]
    UpdateL1InfoTreeV2EventNotFound,
    #[error("Unable to fetch the latest finalized block")]
    LatestFinalizedBlockNotFound,
    #[error("Timeout exceeded while waiting for block {0} to be finalized.")]
    FinalizationTimeoutExceeded(u64),
    #[error("L1 Reorg detected for block number {0}")]
    ReorgDetected(u64),
    #[error("Cannot get the block hash for the block number {0}")]
    BlockHashNotFound(u64),
    #[error("Unable to fetch transaction receipt for {0}")]
    UnableToFetchTransactionReceipt(String),
    #[error("No transaction receipt found for {0}")]
    TransactionReceiptNotFound(String),
    #[error("Failed to fetch aggchain vkey")]
    AggchainVkeyFetchFailed,
    #[error("Failed to retrieve trusted sequencer")]
    TrustedSequencerRetrievalFailed,
    #[error("Failed to retrieve rollup data")]
    RollupDataRetrievalFailed,
    #[error("Unable to get transaction")]
    UnableToGetTransaction {
        #[source]
        source: Box<anyhow::Error>,
    },
    #[error("Unable to parse aggchain vkey")]
    UnableToParseAggchainVkey,
    #[error("Unable to retrieve verifier type")]
    VerifierTypeRetrievalFailed,
}

impl<RpcProvider> L1RpcClient<RpcProvider>
where
    RpcProvider: Middleware + 'static,
{
    pub fn new(
        rpc: Arc<RpcProvider>,
        inner: polygon_rollup_manager::PolygonRollupManager<RpcProvider>,
        l1_info_tree: polygon_zkevm_global_exit_root_v2::PolygonZkEVMGlobalExitRootV2<RpcProvider>,
    ) -> Self {
        Self {
            rpc,
            inner,
            l1_info_tree,
        }
    }

    pub async fn try_new(
        rpc: Arc<RpcProvider>,
        inner: polygon_rollup_manager::PolygonRollupManager<RpcProvider>,
        l1_info_tree: polygon_zkevm_global_exit_root_v2::PolygonZkEVMGlobalExitRootV2<RpcProvider>,
    ) -> Result<Self, L1RpcInitializationError> {
        Ok(Self::new(rpc, inner, l1_info_tree))
    }
}

#[async_trait::async_trait]
impl<RpcProvider> L1TransactionFetcher for L1RpcClient<RpcProvider>
where
    RpcProvider: Middleware + 'static,
{
    async fn fetch_transaction_receipt(
        &self,
        tx_hash: H256,
    ) -> Result<TransactionReceipt, L1RpcError> {
        self.rpc
            .get_transaction_receipt(tx_hash)
            .await
            .map_err(|_| L1RpcError::UnableToFetchTransactionReceipt(tx_hash.to_string()))?
            .ok_or_else(|| L1RpcError::TransactionReceiptNotFound(tx_hash.to_string()))
    }
}
