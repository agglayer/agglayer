//! Agglayer smart-contract bindings.

use std::sync::Arc;

use ethers::prelude::*;
use ethers::providers::Middleware;
use polygon_zkevm_global_exit_root_v2::PolygonZkEVMGlobalExitRootV2Events;
use tracing::{debug, error};

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
    /// L1 info tree entry used for certificates without imported bridge exits.
    default_l1_info_tree_entry: (u32, [u8; 32]),
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
        default_l1_info_tree_entry: (u32, [u8; 32]),
    ) -> Self {
        Self {
            rpc,
            inner,
            l1_info_tree,
            default_l1_info_tree_entry,
        }
    }

    pub async fn try_new(
        rpc: Arc<RpcProvider>,
        inner: polygon_rollup_manager::PolygonRollupManager<RpcProvider>,
        l1_info_tree: polygon_zkevm_global_exit_root_v2::PolygonZkEVMGlobalExitRootV2<RpcProvider>,
    ) -> Result<Self, L1RpcInitializationError> {
        let default_l1_info_tree_entry = {
            let filter = Filter::new()
                .address(l1_info_tree.address())
                .event("InitL1InfoRootMap(uint32,bytes32)")
                .from_block(BlockNumber::Earliest);

            let events = l1_info_tree.client().get_logs(&filter).await.map_err(|e| {
                L1RpcInitializationError::InitL1InfoRootMapEventNotFound(e.to_string())
            })?;

            // Get the first l1 info tree leaf from the init event
            let (l1_leaf_count, l1_info_root) = match events
                .first()
                .cloned()
                .map(|log| PolygonZkEVMGlobalExitRootV2Events::decode_log(&log.into()))
                .ok_or(L1RpcInitializationError::InitL1InfoRootMapEventNotFound(
                    String::from("Event InitL1InfoRootMap not found"),
                ))? {
                Ok(PolygonZkEVMGlobalExitRootV2Events::InitL1InfoRootMapFilter(event)) => {
                    (event.leaf_count, event.current_l1_info_root)
                }
                _ => {
                    return Err(L1RpcInitializationError::InitL1InfoRootMapEventNotFound(
                        String::from("Event InitL1InfoRootMap not found"),
                    ))
                }
            };

            // Check that fetched l1 info root is non-zero
            if l1_info_root == [0u8; 32] {
                return Err(L1RpcInitializationError::InvalidL1InfoRootFromEvent(
                    l1_leaf_count,
                ));
            }

            debug!(
                "Retrieved the default L1 Info Tree entry. leaf_count: {}, root: {}",
                l1_leaf_count,
                H256::from_slice(l1_info_root.as_slice())
            );

            // Use this entry as default
            (l1_leaf_count, l1_info_root)
        };

        Ok(Self::new(
            rpc,
            inner,
            l1_info_tree,
            default_l1_info_tree_entry,
        ))
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use polygon_rollup_manager::PolygonRollupManager;
    use polygon_zkevm_global_exit_root_v2::PolygonZkEVMGlobalExitRootV2;

    use super::*;
    use crate::rollup::RollupContract;

    #[tokio::test]
    #[ignore = "reaches external endpoint"]
    async fn test_fetch_proper_default_l1_leaf_count() {
        let rpc = Arc::new(
            Provider::<Http>::try_from("https://sepolia.gateway.tenderly.co/adEEbh8f3HykepCfd151V")
                .unwrap(),
        );

        // Cardona contracts
        let rollup_manager_contract: H160 = "0x32d33D5137a7cFFb54c5Bf8371172bcEc5f310ff" // bali: 0xe2ef6215adc132df6913c8dd16487abf118d1764
            .parse()
            .unwrap();

        let ger_contract: H160 = "0xAd1490c248c5d3CbAE399Fd529b79B42984277DF" // bali: 0x2968d6d736178f8fe7393cc33c87f29d9c287e78
            .parse()
            .unwrap();

        let l1_rpc = Arc::new(
            L1RpcClient::try_new(
                rpc.clone(),
                PolygonRollupManager::new(rollup_manager_contract, rpc.clone()),
                PolygonZkEVMGlobalExitRootV2::new(ger_contract, rpc.clone()),
            )
            .await
            .unwrap(),
        );

        let (default_leaf_count, _default_l1_info_root) = l1_rpc.default_l1_info_tree_entry;
        let expected_leaf_count = 48445; // bali: 335

        assert_eq!(
            default_leaf_count, expected_leaf_count,
            "default: {}, expected: {}",
            default_leaf_count, expected_leaf_count,
        );

        // check that the awaiting for finalization is done as expected
        let latest_l1_leaf = 73587;
        let _l1_info_root = l1_rpc.get_l1_info_root(latest_l1_leaf).await.unwrap();
    }
}
