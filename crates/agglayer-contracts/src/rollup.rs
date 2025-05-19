use std::collections::HashMap;

use ethers::{prelude::*, providers::Middleware};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use tracing::{debug, error};

use super::{
    polygon_rollup_manager::RollupIDToRollupDataReturn, polygon_zk_evm::PolygonZkEvm,
    polygon_zkevm_global_exit_root_v2::PolygonZkEVMGlobalExitRootV2Events,
};
use crate::{polygon_rollup_manager::RollupDataReturnV2, L1RpcClient, L1RpcError};

#[derive(Debug, FromPrimitive)]
pub enum VerifierType {
    StateTransition = 0,
    Pessimistic = 1,
    ALGateway = 2,
}
/// Polling tick interval used to check for one block to be finalized.
const CHECK_BLOCK_FINALIZED_TICK_INTERVAL: tokio::time::Duration =
    tokio::time::Duration::from_secs(10);

/// Conservative time for finality on Ethereum.
const TIME_TO_FINALITY_ETHEREUM: tokio::time::Duration = tokio::time::Duration::from_secs(30 * 60);

#[async_trait::async_trait]
pub trait RollupContract {
    type M: Middleware;
    async fn get_trusted_sequencer_address(
        &self,
        rollup_id: u32,
        proof_signers: HashMap<u32, Address>,
    ) -> Result<Address, L1RpcError>;

    async fn get_rollup_contract_address(&self, rollup_id: u32) -> Result<Address, L1RpcError>;
    async fn get_prev_pessimistic_root(&self, rollup_id: u32) -> Result<[u8; 32], L1RpcError>;

    async fn get_l1_info_root(&self, l1_leaf_count: u32) -> Result<[u8; 32], L1RpcError>;
    async fn get_verifier_type(&self, rollup_id: u32) -> Result<VerifierType, L1RpcError>;

    fn default_l1_info_tree_entry(&self) -> (u32, [u8; 32]);
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
    ) -> Result<Address, L1RpcError> {
        if let Some(addr) = proof_signers.get(&rollup_id) {
            Ok(*addr)
        } else {
            let rollup_data = self
                .inner
                .rollup_id_to_rollup_data(rollup_id)
                .await
                .map_err(|_| L1RpcError::RollupDataRetrievalFailed)?;

            let rollup_metadata = RollupIDToRollupDataReturn { rollup_data };
            PolygonZkEvm::new(
                rollup_metadata.rollup_data.rollup_contract,
                self.inner.client().clone(),
            )
            .trusted_sequencer()
            .await
            .map_err(|_| L1RpcError::TrustedSequencerRetrievalFailed)
        }
    }

    async fn get_rollup_contract_address(&self, rollup_id: u32) -> Result<Address, L1RpcError> {
        let rollup_data = self
            .inner
            .rollup_id_to_rollup_data(rollup_id)
            .await
            .map_err(|_| L1RpcError::RollupDataRetrievalFailed)?;

        Ok(rollup_data.rollup_contract)
    }
    async fn get_prev_pessimistic_root(&self, rollup_id: u32) -> Result<[u8; 32], L1RpcError> {
        let rollup_data: RollupDataReturnV2 = self
            .inner
            .rollup_id_to_rollup_data_v2(rollup_id)
            .await
            .map_err(|_| L1RpcError::RollupDataRetrievalFailed)?;

        Ok(rollup_data.last_pessimistic_root)
    }

    async fn get_verifier_type(&self, rollup_id: u32) -> Result<VerifierType, L1RpcError> {
        let rollup_data: RollupDataReturnV2 = self
            .inner
            .rollup_id_to_rollup_data_v2(rollup_id)
            .await
            .map_err(|_| L1RpcError::RollupDataRetrievalFailed)?;

        Ok(VerifierType::from_u8(rollup_data.rollup_verifier_type)
            .ok_or(L1RpcError::VerifierTypeRetrievalFailed)?)
    }
}
