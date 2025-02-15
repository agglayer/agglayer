//! Agglayer smart-contract bindings.

use std::{collections::HashMap, sync::Arc};

use ethers::prelude::*;
use ethers::providers::Middleware;
use ethers_contract::{ContractCall, ContractError};
use polygon_rollup_manager::{PolygonRollupManagerErrors, RollupIDToRollupDataReturn};
use tracing::{debug, error};

#[rustfmt::skip]
#[allow(warnings)]
pub mod polygon_rollup_manager;

#[rustfmt::skip]
#[allow(warnings)]
pub mod polygon_zk_evm;

#[rustfmt::skip]
#[allow(warnings)]
pub mod polygon_zkevm_global_exit_root_v2;

pub mod settler;

use polygon_zk_evm::PolygonZkEvm;
use polygon_zkevm_global_exit_root_v2::PolygonZkEVMGlobalExitRootV2Events;
pub use settler::Settler;

#[async_trait::async_trait]
pub trait RollupContract {
    type M: Middleware;
    async fn get_trusted_sequencer_address(
        &self,
        rollup_id: u32,
        proof_signers: HashMap<u32, Address>,
    ) -> Result<Address, ()>;

    async fn get_l1_info_root(&self, l1_leaf_count: u32) -> Result<[u8; 32], L1RpcError>;

    fn default_l1_info_tree_entry(&self) -> (u32, [u8; 32]);
}

/// Polling tick interval used to check for one block to be finalized.
const CHECK_BLOCK_FINALIZED_TICK_INTERVAL: tokio::time::Duration =
    tokio::time::Duration::from_secs(10);

/// Conservative time for finality on Ethereum.
const TIME_TO_FINALITY_ETHEREUM: tokio::time::Duration = tokio::time::Duration::from_secs(30 * 60);

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
}

impl<RpcProvider> L1RpcClient<RpcProvider>
where
    RpcProvider: Middleware + 'static,
{
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

        Ok(Self {
            rpc,
            inner,
            l1_info_tree,
            default_l1_info_tree_entry,
        })
    }
}

#[async_trait::async_trait]
impl<RpcProvider> RollupContract for L1RpcClient<RpcProvider>
where
    RpcProvider: Middleware + 'static,
{
    type M = RpcProvider;

    /// Returns the first entry of the l1 info tree map in the L1.
    fn default_l1_info_tree_entry(&self) -> (u32, [u8; 32]) {
        self.default_l1_info_tree_entry
    }

    async fn get_l1_info_root(&self, l1_leaf_count: u32) -> Result<[u8; 32], L1RpcError> {
        // Get `UpdateL1InfoTreeV2` event for the given leaf count from the latest block
        let filter = Filter::new()
            .address(self.l1_info_tree.address())
            .event("UpdateL1InfoTreeV2(bytes32,uint32,uint256,uint64)")
            .topic1(U256::from_big_endian(&l1_leaf_count.to_be_bytes()))
            .from_block(BlockNumber::Earliest);

        let events = self
            .l1_info_tree
            .client()
            .get_logs(&filter)
            .await
            .map_err(|e| L1RpcError::UpdateL1InfoTreeV2EventFailure(e.to_string()))?;

        // Extract event details
        let (l1_info_root, event_block_number, event_block_hash) = events
            .first()
            .and_then(|log| {
                match PolygonZkEVMGlobalExitRootV2Events::decode_log(&log.clone().into()).ok()? {
                    PolygonZkEVMGlobalExitRootV2Events::UpdateL1InfoTreeV2Filter(event) => Some((
                        event.current_l1_info_root,
                        log.block_number?,
                        log.block_hash?,
                    )),
                    _ => None,
                }
            })
            .ok_or(L1RpcError::UpdateL1InfoTreeV2EventNotFound)?;

        debug!(
            "Retrieved UpdateL1InfoTreeV2 event from block {}. L1 info tree leaf count: {}, root: \
             {}",
            event_block_number,
            l1_leaf_count,
            H256::from_slice(l1_info_root.as_slice())
        );

        // Await for the related block to be finalized
        // NOTE: Cannot use block subscription because the provider is not websocket
        {
            let mut tick = tokio::time::interval(CHECK_BLOCK_FINALIZED_TICK_INTERVAL);
            let mut finalized_block_number: U64 = 0.into();

            _ = tokio::time::timeout(TIME_TO_FINALITY_ETHEREUM, async {
                loop {
                    tick.tick().await;

                    finalized_block_number = self
                        .rpc
                        .get_block(BlockNumber::Finalized)
                        .await
                        .ok()
                        .flatten()
                        .and_then(|block| block.number)
                        .ok_or(L1RpcError::LatestFinalizedBlockNotFound)?;

                    debug!(
                        "Awaiting L1 info tree leaf count ({}) set at block {} to be finalized. \
                         Latest finalized block: {}",
                        l1_leaf_count, event_block_number, finalized_block_number,
                    );

                    // Check whether the block number containing the event is now finalized.
                    if finalized_block_number >= event_block_number {
                        // Verify that the hash of the block containing
                        // the event did not change due to potential reorg
                        let retrieved_block_hash = self
                            .rpc
                            .get_block(BlockId::Number(event_block_number.into()))
                            .await
                            .ok()
                            .flatten()
                            .and_then(|block| block.hash)
                            .ok_or(L1RpcError::BlockHashNotFound(event_block_number.as_u64()))?;

                        if retrieved_block_hash != event_block_hash {
                            error!(
                                "Reorg detected! Retrieved block hash ({:?}) does not match \
                                 expected event block hash ({:?}).",
                                retrieved_block_hash, event_block_hash
                            );
                            return Err(L1RpcError::ReorgDetected(event_block_number.as_u64()));
                        }

                        break;
                    }
                }

                Ok(())
            })
            .await
            .map_err(|_| {
                error!(
                    "Timeout occurred while waiting for block {} to be finalized. Latest \
                     finalized block: {}",
                    event_block_number, finalized_block_number
                );
                L1RpcError::FinalizationTimeoutExceeded(event_block_number.as_u64())
            })??;
        }

        Ok(l1_info_root)
    }

    async fn get_trusted_sequencer_address(
        &self,
        rollup_id: u32,
        proof_signers: HashMap<u32, Address>,
    ) -> Result<Address, ()> {
        if let Some(addr) = proof_signers.get(&rollup_id) {
            Ok(*addr)
        } else {
            let rollup_data = self
                .inner
                .rollup_id_to_rollup_data(rollup_id)
                .await
                .map_err(|_| ())?;

            let rollup_metadata = RollupIDToRollupDataReturn { rollup_data };
            PolygonZkEvm::new(
                rollup_metadata.rollup_data.rollup_contract,
                self.inner.client().clone(),
            )
            .trusted_sequencer()
            .await
            .map_err(|_| ())
        }
    }
}

#[async_trait::async_trait]
impl<RpcProvider> Settler for L1RpcClient<RpcProvider>
where
    RpcProvider: Middleware + 'static,
{
    type M = RpcProvider;

    async fn transaction_exists(&self, tx_hash: H256) -> Result<bool, String> {
        self.rpc
            .get_transaction(tx_hash)
            .await
            .map_err(|e| e.to_string())
            .map(|v| v.is_some())
    }

    fn build_pending_transaction(
        &self,
        tx_hash: H256,
    ) -> PendingTransaction<'_, <Self::M as Middleware>::Provider> {
        PendingTransaction::new(tx_hash, self.rpc.as_ref().provider())
    }

    fn decode_contract_revert(error: &ContractError<Self::M>) -> Option<String> {
        error
            .decode_contract_revert::<PolygonRollupManagerErrors>()
            .map(|err| format!("{:?}", err))
    }

    fn build_verify_pessimistic_trusted_aggregator_call(
        &self,
        rollup_id: u32,
        l_1_info_tree_leaf_count: u32,
        new_local_exit_root: [u8; 32],
        new_pessimistic_root: [u8; 32],
        proof: ::ethers::core::types::Bytes,
    ) -> ContractCall<Self::M, ()> {
        self.inner.verify_pessimistic_trusted_aggregator(
            rollup_id,
            l_1_info_tree_leaf_count,
            new_local_exit_root,
            new_pessimistic_root,
            proof,
        )
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use polygon_rollup_manager::PolygonRollupManager;
    use polygon_zkevm_global_exit_root_v2::PolygonZkEVMGlobalExitRootV2;

    use super::*;

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
