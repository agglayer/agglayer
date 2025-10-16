//! Agglayer smart-contract bindings.

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use agglayer_primitives::U256;
use alloy::{
    eips::BlockNumberOrTag,
    primitives::{Address, FixedBytes, TxHash, B256},
    providers::Provider,
    rpc::types::{Filter, TransactionReceipt},
    signers::k256::elliptic_curve::ff::derive::bitvec::macros::internal::funty::Fundamental,
};
use tracing::{debug, error};

pub mod aggchain;
pub mod contracts;
pub mod rollup;
pub mod settler;

pub use aggchain::AggchainContract;
pub use rollup::RollupContract;
pub use settler::Settler;

/// Gas price parameters for L1 transactions.
#[derive(Debug, Clone)]
pub struct GasPriceParams {
    /// Gas price multiplier for transactions (scaled by 1000).
    pub multiplier_per_1000: u64,
    /// Minimum gas price floor (in wei) for transactions.
    pub floor: u128,
    /// Maximum gas price ceiling (in wei) for transactions.
    pub ceiling: u128,
}

impl Default for GasPriceParams {
    fn default() -> Self {
        GasPriceParams {
            multiplier_per_1000: 1000, // 1.0 scaled by 1000
            floor: 0,
            ceiling: u128::MAX,
        }
    }
}

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
    /// Gas price parameters for transactions.
    gas_price_params: GasPriceParams,
    /// Cached UpdateL1InfoTreeV2 first l1_info_root for each leaf count.
    /// Map<leaf_count, l1_info_root>
    l1_info_roots: Arc<RwLock<HashMap<u32, [u8; 32]>>>,
    /// Number of blocks to query when filtering for events.
    /// This is to avoid hitting provider limits when querying large block
    /// ranges or errors like "query returned more than 10000 results".
    event_filter_block_range: u64,
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
    #[error("Unable to fetch transaction receipt for {tx_hash}: {source}")]
    UnableToFetchTransactionReceipt {
        tx_hash: String,
        #[source]
        source: eyre::Error,
    },
    // WARNING: following error message is used in checks, do not change without updating the
    // checks
    #[error("No transaction receipt found for tx {0}, not yet mined")]
    TransactionNotYetMined(String),
    #[error("Failed to fetch aggchain vkey")]
    AggchainVkeyFetchFailed,
    #[error("Failed to retrieve trusted sequencer")]
    TrustedSequencerRetrievalFailed,
    #[error("Failed to retrieve rollup data")]
    RollupDataRetrievalFailed,
    #[error("Unable to get transaction")]
    UnableToGetTransaction {
        tx_hash: String,
        #[source]
        source: eyre::Error,
    },
    #[error("Unable to parse aggchain vkey")]
    UnableToParseAggchainVkey,
    #[error("Unable to retrieve verifier type")]
    VerifierTypeRetrievalFailed,
    #[error("Unable to retrieve the aggchain hash")]
    AggchainHashFetchFailed,
    #[error("The rollup contract is either invalid or not set for the specified rollup id {0}")]
    InvalidRollupContract(u32),
    #[error("Unable to fetch the multisig signers: {0}")]
    MultisigSignersFetchFailed(#[source] alloy::contract::Error),
    #[error("Unable to fetch the multisig threshold: {0}")]
    MultisigThresholdFetchFailed(#[source] alloy::contract::Error),
    #[error("Threshold value is too large to fit in usize. fetched value: {fetched}")]
    ThresholdTypeOverflow { fetched: U256 },
    #[error("Transaction receipt for tx {0} failed on L1")]
    TransactionReceiptFailedOnL1(TxHash),
    #[error("Failed to get the events: {0}")]
    FailedToQueryEvents(String),
    #[error("L1 info roots cache lock poisoned")]
    CacheLockPoisoned,
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
        gas_price_params: GasPriceParams,
        event_filter_block_range: u64,
    ) -> Self {
        Self {
            rpc,
            inner,
            l1_info_tree,
            default_l1_info_tree_entry,
            gas_multiplier_factor,
            gas_price_params,
            l1_info_roots: Arc::new(RwLock::new(HashMap::new())),
            event_filter_block_range,
        }
    }

    pub async fn try_new(
        rpc: Arc<RpcProvider>,
        inner: contracts::PolygonRollupManagerRpcClient<RpcProvider>,
        l1_info_tree: Address,
        gas_multiplier_factor: u32,
        gas_price_params: GasPriceParams,
        event_filter_block_range: u64,
    ) -> Result<Self, L1RpcInitializationError>
    where
        RpcProvider: alloy::providers::Provider + Clone + 'static,
    {
        use alloy::sol_types::SolEvent;

        use crate::contracts::PolygonZkEvmGlobalExitRootV2::InitL1InfoRootMap;

        let default_l1_info_tree_entry = {
            // To not hit the provider limit, we start from genesis and restrict search
            // to the self. blocks range.
            let mut events = Vec::new();
            let mut start_block = 0u64;
            let latest_network_block = rpc
                .get_block_number()
                .await
                .map_err(|e| {
                    error!("Failed to get the latest block number: {e:?}");
                    L1RpcInitializationError::InitL1InfoRootMapEventNotFound(e.to_string())
                })?
                .as_u64();
            while events.is_empty() && start_block <= latest_network_block {
                let end_block =
                    (start_block + event_filter_block_range - 1).min(latest_network_block);
                let filter = Filter::new()
                    .address(l1_info_tree)
                    .event_signature(InitL1InfoRootMap::SIGNATURE_HASH)
                    .from_block(BlockNumberOrTag::Number(start_block))
                    .to_block(BlockNumberOrTag::Number(end_block));

                // Get logs from the contract
                events = rpc.get_logs(&filter).await.map_err(|err| {
                    error!("Failed to get InitL1InfoRootMap events: {err:?}");
                    L1RpcInitializationError::InitL1InfoRootMapEventNotFound(err.to_string())
                })?;
                start_block += event_filter_block_range;
            }

            // Get the first log and decode it
            let first_log =
                events
                    .first()
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
            gas_price_params,
            event_filter_block_range,
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
            .map_err(|err| L1RpcError::UnableToFetchTransactionReceipt {
                tx_hash: tx_hash.to_string(),
                source: err.into(),
            })?
            .ok_or_else(|| L1RpcError::TransactionNotYetMined(tx_hash.to_string()))
    }

    fn get_provider(&self) -> &Self::Provider {
        self.rpc.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use prover_alloy::build_alloy_fill_provider;
    use url::Url;

    use super::*;
    use crate::rollup::RollupContract;

    struct ContractSetup {
        rollup_manager: Address,
        ger_contract: Address,
    }

    impl ContractSetup {
        pub fn new() -> Self {
            Self {
                rollup_manager: "0x32d33D5137a7cFFb54c5Bf8371172bcEc5f310ff" // bali: 0xe2ef6215adc132df6913c8dd16487abf118d1764
                    .parse()
                    .unwrap(),
                ger_contract: "0xAd1490c248c5d3CbAE399Fd529b79B42984277DF" // bali: 0x2968d6d736178f8fe7393cc33c87f29d9c287e78
                    .parse()
                    .unwrap(),
            }
        }
    }

    #[tokio::test]
    #[ignore = "reaches external endpoint"]
    async fn test_fetch_proper_default_l1_leaf_count() {
        let rpc_url = std::env::var("L1_RPC_ENDPOINT")
            .expect("L1_RPC_ENDPOINT must be defined")
            .parse::<Url>()
            .expect("Invalid URL format");

        let rpc = build_alloy_fill_provider(
            &rpc_url,
            prover_alloy::DEFAULT_HTTP_RPC_NODE_INITIAL_BACKOFF_MS,
            prover_alloy::DEFAULT_HTTP_RPC_NODE_BACKOFF_MAX_RETRIES,
        )
        .expect("valid alloy provider");

        let contracts = ContractSetup::new();
        let l1_rpc = Arc::new(
            L1RpcClient::try_new(
                Arc::new(rpc.clone()),
                contracts::PolygonRollupManager::new(contracts.rollup_manager, rpc),
                contracts.ger_contract,
                100,
                GasPriceParams::default(),
                10000,
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

    #[tokio::test]
    #[ignore = "reaches external endpoint"]
    async fn test_fetch_multisig_context() {
        let rpc_url = std::env::var("L1_RPC_ENDPOINT")
            .expect("L1_RPC_ENDPOINT must be defined")
            .parse::<Url>()
            .expect("Invalid URL format");

        let rpc = build_alloy_fill_provider(
            &rpc_url,
            prover_alloy::DEFAULT_HTTP_RPC_NODE_INITIAL_BACKOFF_MS,
            prover_alloy::DEFAULT_HTTP_RPC_NODE_BACKOFF_MAX_RETRIES,
        )
        .expect("valid alloy provider");

        let contracts = ContractSetup::new();
        let l1_rpc = Arc::new(
            L1RpcClient::try_new(
                Arc::new(rpc.clone()),
                contracts::PolygonRollupManager::new(contracts.rollup_manager, rpc),
                contracts.ger_contract,
                100,
                GasPriceParams::default(),
                10000,
            )
            .await
            .unwrap(),
        );

        let rollup_contract_address: agglayer_primitives::Address =
            "0x5D884D7808DF483CB575Df8B4C480a1880462B74"
                .parse()
                .unwrap();

        let (signers, threshold) = l1_rpc
            .get_multisig_context(rollup_contract_address)
            .await
            .unwrap();

        let expected_signers: Vec<agglayer_primitives::Address> =
            vec!["0x0cae25c8623761783fe4ce241c9b428126a7612a"
                .parse()
                .unwrap()];
        let expected_threshold = 1;

        assert_eq!(signers, expected_signers);
        assert_eq!(threshold, expected_threshold);
    }
}
