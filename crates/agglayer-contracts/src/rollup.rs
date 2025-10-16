use std::collections::HashMap;

use agglayer_primitives::Address;
use alloy::{
    eips::{BlockId, BlockNumberOrTag},
    primitives::{TxHash, U256},
    providers::Provider,
    rpc::types::Filter,
    signers::k256::elliptic_curve::ff::derive::bitvec::macros::internal::funty::Fundamental,
};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use tracing::{debug, error, trace};

use crate::{
    contracts::{PolygonRollupManager::RollupDataReturnV2, PolygonZkEvm},
    L1RpcClient, L1RpcError,
};

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
    async fn get_trusted_sequencer_address(
        &self,
        rollup_id: u32,
        proof_signers: HashMap<u32, Address>,
    ) -> Result<Address, L1RpcError>;

    async fn get_rollup_contract_address(&self, rollup_id: u32) -> Result<Address, L1RpcError>;
    async fn get_prev_pessimistic_root(
        &self,
        rollup_id: u32,
        before_tx: Option<TxHash>,
    ) -> Result<[u8; 32], L1RpcError>;

    async fn get_l1_info_root(&self, l1_leaf_count: u32) -> Result<[u8; 32], L1RpcError>;
    async fn get_verifier_type(&self, rollup_id: u32) -> Result<VerifierType, L1RpcError>;

    fn default_l1_info_tree_entry(&self) -> (u32, [u8; 32]);

    fn get_rollup_manager_address(&self) -> Address;

    fn get_event_filter_block_range(&self) -> u64;
}

#[async_trait::async_trait]
impl<RpcProvider> RollupContract for L1RpcClient<RpcProvider>
where
    RpcProvider: alloy::providers::Provider + Clone + 'static,
{
    /// Returns the first entry of the l1 info tree map in the L1.
    fn default_l1_info_tree_entry(&self) -> (u32, [u8; 32]) {
        self.default_l1_info_tree_entry
    }

    async fn get_l1_info_root(&self, l1_leaf_count: u32) -> Result<[u8; 32], L1RpcError> {
        // Check if we already have this l1_info_root cached
        {
            let cache = self
                .l1_info_roots
                .read()
                .map_err(|_| L1RpcError::CacheLockPoisoned)?;
            if let Some(&cached_root) = cache.get(&l1_leaf_count) {
                trace!(
                    "Retrieved cached L1 info root for leaf count {}: {}",
                    l1_leaf_count,
                    alloy::primitives::B256::from(cached_root)
                );
                return Ok(cached_root);
            }
        }

        use alloy::sol_types::SolEvent;

        use crate::contracts::PolygonZkEvmGlobalExitRootV2::UpdateL1InfoTreeV2;

        // Get `UpdateL1InfoTreeV2` event for the given leaf count from the latest block
        // To not hit the provider limit, we start from genesis and restrict search
        // to the self.event_filter_block_range blocks range.
        let mut events = Vec::new();
        let mut start_block = 0u64;
        let latest_network_block = self
            .rpc
            .get_block_number()
            .await
            .map_err(|e| {
                error!("Failed to fetch latest block number: {}", e);
                L1RpcError::UpdateL1InfoTreeV2EventFailure(e.to_string())
            })?
            .as_u64();
        while events.is_empty() && start_block <= latest_network_block {
            let end_block =
                (start_block + self.event_filter_block_range - 1).min(latest_network_block);
            let filter = Filter::new()
                .address(self.l1_info_tree)
                .event_signature(UpdateL1InfoTreeV2::SIGNATURE_HASH)
                .topic1(U256::from(l1_leaf_count))
                .from_block(BlockNumberOrTag::Number(start_block))
                .to_block(BlockNumberOrTag::Number(end_block));

            events = self.rpc.get_logs(&filter).await.map_err(|e| {
                error!("Failed to fetch UpdateL1InfoTreeV2EventFailure logs: {}", e);
                L1RpcError::UpdateL1InfoTreeV2EventFailure(e.to_string())
            })?;

            start_block += self.event_filter_block_range;
        }

        // Extract event details using alloy's event decoding
        let (l1_info_root, event_block_number, event_block_hash) = events
            .first()
            .and_then(|log| {
                // Use alloy's direct event decoding
                if let Ok(decoded_event) = UpdateL1InfoTreeV2::decode_log(&log.clone().into()) {
                    Some((
                        <[u8; 32]>::from(decoded_event.currentL1InfoRoot),
                        log.block_number?,
                        log.block_hash?,
                    ))
                } else {
                    None
                }
            })
            .ok_or(L1RpcError::UpdateL1InfoTreeV2EventNotFound)?;

        debug!(
            "Retrieved UpdateL1InfoTreeV2 event from block {}. L1 info tree leaf count: {}, root: \
             {}",
            event_block_number,
            l1_leaf_count,
            alloy::primitives::B256::from(l1_info_root) // Use alloy's B256 instead of H256
        );

        // Await for the related block to be finalized
        // NOTE: Cannot use block subscription because the provider is not websocket
        {
            let mut tick = tokio::time::interval(CHECK_BLOCK_FINALIZED_TICK_INTERVAL);
            let mut finalized_block_number = 0;

            _ = tokio::time::timeout(TIME_TO_FINALITY_ETHEREUM, async {
                loop {
                    tick.tick().await;

                    finalized_block_number = self
                        .rpc
                        .get_block(BlockId::Number(BlockNumberOrTag::Finalized))
                        .await
                        .ok()
                        .flatten()
                        .map(|block| block.header.number)
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
                            .map(|block| block.header.hash)
                            .ok_or(L1RpcError::BlockHashNotFound(event_block_number))?;

                        if retrieved_block_hash != event_block_hash {
                            error!(
                                "Reorg detected! Retrieved block hash ({:?}) does not match \
                                 expected event block hash ({:?}).",
                                retrieved_block_hash, event_block_hash
                            );
                            return Err(L1RpcError::ReorgDetected(event_block_number));
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

        // Cache the retrieved l1_info_root for future use
        {
            let mut cache = self
                .l1_info_roots
                .write()
                .map_err(|_| L1RpcError::CacheLockPoisoned)?;
            cache.insert(l1_leaf_count, l1_info_root);
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
                .rollupIDToRollupData(rollup_id)
                .call()
                .await
                .map_err(|_| L1RpcError::RollupDataRetrievalFailed)?;

            if rollup_data.rollupContract.is_zero() {
                return Err(L1RpcError::InvalidRollupContract(rollup_id));
            }

            PolygonZkEvm::new(rollup_data.rollupContract, self.rpc.clone())
                .trustedSequencer()
                .call()
                .await
                .map(Into::into)
                .map_err(|_| L1RpcError::TrustedSequencerRetrievalFailed)
        }
    }

    async fn get_rollup_contract_address(&self, rollup_id: u32) -> Result<Address, L1RpcError> {
        let rollup_data = self
            .inner
            .rollupIDToRollupData(rollup_id)
            .call()
            .await
            .map_err(|_| L1RpcError::RollupDataRetrievalFailed)?;

        if rollup_data.rollupContract.is_zero() {
            return Err(L1RpcError::InvalidRollupContract(rollup_id));
        }

        Ok(rollup_data.rollupContract.into())
    }
    async fn get_prev_pessimistic_root(
        &self,
        rollup_id: u32,
        before_tx_hash: Option<TxHash>,
    ) -> Result<[u8; 32], L1RpcError> {
        let at_block = if let Some(tx_hash) = before_tx_hash {
            let receipt = self
                .rpc
                .get_transaction_receipt(tx_hash)
                .await
                .map_err(|err| L1RpcError::UnableToFetchTransactionReceipt {
                    tx_hash: tx_hash.to_string(),
                    source: err.into(),
                })?
                .ok_or_else(|| L1RpcError::TransactionNotYetMined(tx_hash.to_string()))?;

            if receipt.status() {
                receipt
                    .block_number
                    .map(|block| {
                        let block = block.saturating_sub(1);
                        BlockId::number(block)
                    })
                    .unwrap_or_else(BlockId::latest)
            } else {
                return Err(L1RpcError::TransactionReceiptFailedOnL1(tx_hash));
            }
        } else {
            BlockId::latest()
        };

        let rollup_data: RollupDataReturnV2 = self
            .inner
            .rollupIDToRollupDataV2(rollup_id)
            .block(at_block)
            .call()
            .await
            .map_err(|_| L1RpcError::RollupDataRetrievalFailed)?;

        Ok(*rollup_data.lastPessimisticRoot)
    }

    async fn get_verifier_type(&self, rollup_id: u32) -> Result<VerifierType, L1RpcError> {
        let rollup_data: RollupDataReturnV2 = self
            .inner
            .rollupIDToRollupDataV2(rollup_id)
            .call()
            .await
            .map_err(|_| L1RpcError::RollupDataRetrievalFailed)?;

        Ok(VerifierType::from_u8(rollup_data.rollupVerifierType)
            .ok_or(L1RpcError::VerifierTypeRetrievalFailed)?)
    }

    fn get_rollup_manager_address(&self) -> Address {
        (*self.inner.address()).into()
    }

    fn get_event_filter_block_range(&self) -> u64 {
        self.event_filter_block_range
    }
}
