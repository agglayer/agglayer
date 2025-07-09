//! Agglayer smart-contract bindings.

use std::sync::Arc;

use alloy::{
    eips::BlockNumberOrTag,
    primitives::{Address, FixedBytes, B256},
    providers::Provider,
    rpc::types::{Filter, TransactionReceipt},
};
use tracing::{debug, error};

pub mod aggchain;
pub mod contracts;
pub mod rollup;
pub mod settler;

pub use aggchain::AggchainContract;
pub use rollup::RollupContract;
pub use settler::Settler;

#[async_trait::async_trait]
pub trait L1TransactionFetcher {
    type Provider: Provider;

    /// Fetches the transaction receipt for a given transaction hash.
    async fn fetch_transaction_receipt(
        &self,
        tx_hash: B256,
    ) -> Result<TransactionReceipt, L1RpcError>;

    /// Returns the provider for direct access to watch transactions
    fn get_provider(&self) -> &Self::Provider;
}

pub struct L1RpcClient<RpcProvider> {
    /// RPC provider to interact with the L1 blockchain.
    rpc: Arc<RpcProvider>,
    /// Inner client for interacting with the Polygon Rollup Manager contract.
    inner: contracts::PolygonRollupManagerRpcClient<RpcProvider>,
    /// Address of the PolygonZkEVMGlobalExitRootV2 contract
    l1_info_tree: Address,
    /// L1 info tree entry used for certificates without imported bridge exits.
    default_l1_info_tree_entry: (u32, [u8; 32]),
    /// Gas multiplier factor for transactions.
    gas_multiplier_factor: u32,
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
    #[error("Unable to retrieve the aggchain hash")]
    AggchainHashFetchFailed,
}

impl<RpcProvider> L1RpcClient<RpcProvider>
where
    RpcProvider: alloy::providers::Provider + Clone + 'static,
{
    pub fn new(
        rpc: Arc<RpcProvider>,
        inner: contracts::PolygonRollupManagerRpcClient<RpcProvider>,
        l1_info_tree: Address,
        default_l1_info_tree_entry: (u32, [u8; 32]),
        gas_multiplier_factor: u32,
    ) -> Self {
        Self {
            rpc,
            inner,
            l1_info_tree,
            default_l1_info_tree_entry,
            gas_multiplier_factor,
        }
    }

    pub async fn try_new(
        rpc: Arc<RpcProvider>,
        inner: contracts::PolygonRollupManagerRpcClient<RpcProvider>,
        l1_info_tree: Address,
        gas_multiplier_factor: u32,
    ) -> Result<Self, L1RpcInitializationError>
    where
        RpcProvider: alloy::providers::Provider + Clone + 'static,
    {
        use alloy::sol_types::SolEvent;

        use crate::contracts::PolygonZkEvmGlobalExitRootV2::InitL1InfoRootMap;

        let default_l1_info_tree_entry = {
            // Create filter for InitL1InfoRootMap events
            let filter = Filter::new()
                .address(l1_info_tree)
                .event_signature(InitL1InfoRootMap::SIGNATURE_HASH)
                .from_block(BlockNumberOrTag::Earliest);

            // Get logs from the contract
            let logs = rpc.get_logs(&filter).await.map_err(|e| {
                L1RpcInitializationError::InitL1InfoRootMapEventNotFound(e.to_string())
            })?;

            // Get the first log and decode it
            let first_log =
                logs.first()
                    .ok_or(L1RpcInitializationError::InitL1InfoRootMapEventNotFound(
                        String::from("Event InitL1InfoRootMap not found"),
                    ))?;

            // Decode the log using alloy's generated event type
            let decoded_event =
                InitL1InfoRootMap::decode_log(&first_log.clone().into()).map_err(|_| {
                    L1RpcInitializationError::InitL1InfoRootMapEventNotFound(String::from(
                        "Failed to decode InitL1InfoRootMap event",
                    ))
                })?;

            let l1_leaf_count = decoded_event.leafCount;
            let l1_info_root: [u8; 32] = decoded_event.currentL1InfoRoot.into();

            // Check that fetched l1 info root is non-zero
            if l1_info_root == [0u8; 32] {
                return Err(L1RpcInitializationError::InvalidL1InfoRootFromEvent(
                    l1_leaf_count,
                ));
            }

            debug!(
                "Retrieved the default L1 Info Tree entry. leaf_count: {}, root: {}",
                l1_leaf_count,
                FixedBytes::<32>::from(l1_info_root)
            );

            // Use this entry as default
            (l1_leaf_count, l1_info_root)
        };

        Ok(Self::new(
            rpc,
            inner,
            l1_info_tree,
            default_l1_info_tree_entry,
            gas_multiplier_factor,
        ))
    }
}

#[async_trait::async_trait]
impl<RpcProvider> L1TransactionFetcher for L1RpcClient<RpcProvider>
where
    RpcProvider: alloy::providers::Provider + Clone + 'static,
{
    type Provider = RpcProvider;

    async fn fetch_transaction_receipt(
        &self,
        tx_hash: B256,
    ) -> Result<TransactionReceipt, L1RpcError> {
        self.rpc
            .get_transaction_receipt(tx_hash)
            .await
            .map_err(|_| L1RpcError::UnableToFetchTransactionReceipt(tx_hash.to_string()))?
            .ok_or_else(|| L1RpcError::TransactionReceiptNotFound(tx_hash.to_string()))
    }

    fn get_provider(&self) -> &Self::Provider {
        self.rpc.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use std::{str::FromStr, sync::Arc};

    use prover_alloy::build_alloy_fill_provider;
    use url::Url;

    use super::*;
    use crate::rollup::RollupContract;

    #[tokio::test]
    #[ignore = "reaches external endpoint"]
    async fn test_fetch_proper_default_l1_leaf_count() {
        let rpc = build_alloy_fill_provider(
            &Url::from_str("https://sepolia.gateway.tenderly.co/adEEbh8f3HykepCfd151V").unwrap(),
            prover_alloy::DEFAULT_HTTP_RPC_NODE_INITIAL_BACKOFF_MS,
            prover_alloy::DEFAULT_HTTP_RPC_NODE_BACKOFF_MAX_RETRIES,
        )
        .expect("valid alloy provider");

        // Cardona contracts
        let rollup_manager_contract: agglayer_primitives::Address =
            "0x32d33D5137a7cFFb54c5Bf8371172bcEc5f310ff" // bali: 0xe2ef6215adc132df6913c8dd16487abf118d1764
                .parse()
                .unwrap();

        let ger_contract: agglayer_primitives::Address =
            "0xAd1490c248c5d3CbAE399Fd529b79B42984277DF" // bali: 0x2968d6d736178f8fe7393cc33c87f29d9c287e78
                .parse()
                .unwrap();

        let l1_rpc = Arc::new(
            L1RpcClient::try_new(
                Arc::new(rpc.clone()),
                contracts::PolygonRollupManager::new(rollup_manager_contract.into(), rpc),
                ger_contract.into(),
                100,
            )
            .await
            .unwrap(),
        );

        let (default_leaf_count, _default_l1_info_root) = l1_rpc.default_l1_info_tree_entry;
        let expected_leaf_count = 48445; // bali: 335

        assert_eq!(
            default_leaf_count, expected_leaf_count,
            "default: {default_leaf_count}, expected: {expected_leaf_count}"
        );

        // check that the awaiting finalization is done as expected
        let latest_l1_leaf = 73587;
        let _l1_info_root = l1_rpc.get_l1_info_root(latest_l1_leaf).await.unwrap();
        println!(
            "L1 info root for leaf count {latest_l1_leaf} is: {}",
            FixedBytes::<32>::from(_l1_info_root)
        );
    }
}
