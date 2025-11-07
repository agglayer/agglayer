//! Agglayer smart-contract bindings.

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use agglayer_primitives::U256;
use agglayer_types::SettlementTxHash;
use alloy::{
    eips::{eip1559::Eip1559Estimation, BlockNumberOrTag},
    primitives::{Address, FixedBytes, TxHash},
    providers::Provider,
    rpc::types::{Filter, TransactionReceipt},
};
use tracing::{debug, error, info};

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
        tx_hash: SettlementTxHash,
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
    global_exit_root_manager_contract: Address,
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
    #[error("Failed to get the `UpdateL1InfoTreeV2` events")]
    UpdateL1InfoTreeV2EventFailure(#[source] eyre::Error),

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
        tx_hash: SettlementTxHash,
        #[source]
        source: eyre::Error,
    },

    #[error("Transaction receipt not found for tx {0}, it is most likely not yet mined")]
    TransactionNotYetMined(SettlementTxHash),

    #[error("Failed to fetch aggchain vkey")]
    AggchainVkeyFetchFailed,

    #[error("Failed to retrieve trusted sequencer")]
    TrustedSequencerRetrievalFailed,

    #[error("Failed to retrieve rollup data")]
    RollupDataRetrievalFailed,

    #[error("Unable to get transaction {tx_hash}")]
    UnableToGetTransaction {
        tx_hash: SettlementTxHash,
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

    #[error("Failed to get the events")]
    FailedToQueryEvents(#[source] eyre::Error),

    #[error("L1 info roots cache lock poisoned")]
    CacheLockPoisoned,
}

impl<RpcProvider> L1RpcClient<RpcProvider>
where
    RpcProvider: alloy::providers::Provider + Clone + 'static,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        rpc: Arc<RpcProvider>,
        inner: contracts::PolygonRollupManagerRpcClient<RpcProvider>,
        global_exit_root_manager_contract: Address,
        default_l1_info_tree_entry: (u32, [u8; 32]),
        gas_multiplier_factor: u32,
        gas_price_params: GasPriceParams,
        event_filter_block_range: u64,
    ) -> Self {
        Self {
            rpc,
            inner,
            global_exit_root_manager_contract,
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
        global_exit_root_manager_contract: Address,
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
            // Start search from genesis. Contracts have very few `InitL1InfoRootMap`
            // events, so should not hit the provider limits.
            debug!(
                "Querying InitL1InfoRootMap event search contract address: \
                 {global_exit_root_manager_contract}"
            );

            let filter = Filter::new()
                .address(global_exit_root_manager_contract)
                .event_signature(InitL1InfoRootMap::SIGNATURE_HASH)
                .from_block(BlockNumberOrTag::Earliest);

            // Get logs from the contract
            let events = rpc.get_logs(&filter).await.map_err(|error| {
                error!(?error, "Failed to get InitL1InfoRootMap events");
                L1RpcInitializationError::InitL1InfoRootMapEventNotFound(error.to_string())
            })?;

            // Get the first log and decode it
            let first_log =
                events
                    .first()
                    .ok_or(L1RpcInitializationError::InitL1InfoRootMapEventNotFound(
                        String::from("Event InitL1InfoRootMap not found"),
                    ))?;

            info!(
                "Found InitL1InfoRootMap event on block {:?}",
                first_log.block_number
            );

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
            global_exit_root_manager_contract,
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
        tx_hash: SettlementTxHash,
    ) -> Result<TransactionReceipt, L1RpcError> {
        self.rpc
            .get_transaction_receipt(tx_hash.into())
            .await
            .map_err(|err| L1RpcError::UnableToFetchTransactionReceipt {
                tx_hash,
                source: err.into(),
            })?
            .ok_or_else(|| L1RpcError::TransactionNotYetMined(tx_hash))
    }

    fn get_provider(&self) -> &Self::Provider {
        self.rpc.as_ref()
    }
}

pub fn adjust_gas_estimate(
    estimate: &Eip1559Estimation,
    params: &GasPriceParams,
) -> Eip1559Estimation {
    let GasPriceParams {
        floor,
        ceiling,
        multiplier_per_1000,
    } = params;

    // Apply gas price multiplier and floor/ceiling constraints
    let adjust = |fee: u128| -> u128 {
        // Multiply by multiplier_per_1000 and divide by 1000
        fee.saturating_mul(*multiplier_per_1000 as u128) / 1000
    };

    let mut max_fee_per_gas = adjust(estimate.max_fee_per_gas).max(*floor);
    if max_fee_per_gas > *ceiling {
        tracing::warn!(
            max_fee_per_gas_estimated = estimate.max_fee_per_gas,
            max_fee_per_gas_adjusted = max_fee_per_gas,
            max_fee_per_gas_ceiling = ceiling,
            "Exceeded configured gas ceiling, clamping",
        );
        max_fee_per_gas = *ceiling;
    }

    let max_priority_fee_per_gas = adjust(estimate.max_priority_fee_per_gas).min(*ceiling);

    let adjusted = Eip1559Estimation {
        max_fee_per_gas,
        max_priority_fee_per_gas,
    };

    if &adjusted != estimate {
        debug!(
            estimate=?estimate,
            adjusted=?adjusted,
            "Applied gas price adjustment."
        );
    }

    adjusted
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use alloy::eips::eip1559::Eip1559Estimation;
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
                rollup_manager: "0xE2EF6215aDc132Df6913C8DD16487aBF118d1764" // Bali
                    .parse()
                    .unwrap(),
                ger_contract: "0x2968D6d736178f8FE7393CC33C87f29D9C287e78" // Bali
                    .parse()
                    .unwrap(),
            }
        }
    }

    #[test_log::test(tokio::test)]
    #[ignore = "reaches external endpoint"]
    async fn test_get_l1_info_root_for_leaf_counts() {
        use url::Url;

        // Use L1_RPC_ENDPOINT environment variable (should be set to Sepolia endpoint)
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

        tracing::info!("Testing get_l1_info_root for leaf counts for Bali testnet");

        // Create L1RpcClient with default config for other parameters for Bali testnet
        // InitL1InfoRootMap event is on block 6487027
        let contracts = ContractSetup::new();
        let l1_rpc = L1RpcClient::try_new(
            Arc::new(rpc.clone()),
            contracts::PolygonRollupManager::new(contracts.rollup_manager, rpc),
            contracts.ger_contract,
            100, // default gas_multiplier_factor
            GasPriceParams::default(),
            10000, // default event_filter_block_range
        )
        .await
        .expect("Failed to create L1RpcClient");

        // Test get_l1_info_root for specific leaf counts from log output
        let leaf_counts = vec![10000, 32131, 50000, 62322, 84213, 200000];

        tracing::info!(
            "Testing {} leaf counts: {:?}",
            leaf_counts.len(),
            leaf_counts
        );

        for leaf_count in leaf_counts {
            tracing::debug!("Testing leaf count: {}", leaf_count);

            match l1_rpc.get_l1_info_root(leaf_count).await {
                Ok(l1_info_root) => {
                    tracing::info!(
                        "Leaf count found {}: L1 info root = {}",
                        leaf_count,
                        FixedBytes::<32>::from(l1_info_root)
                    );
                    // Verify that the root is not all zeros (which would indicate an invalid
                    // result)
                    assert_ne!(
                        l1_info_root, [0u8; 32],
                        "L1 info root should not be all zeros for leaf count {leaf_count}",
                    );
                }
                Err(error) => {
                    tracing::warn!(
                        "Failed to get L1 info root for leaf count {leaf_count}: {error}",
                    );
                    // For this test, we expect some leaf counts might not exist
                    // yet, so we don't fail the test but
                    // just continue
                }
            }
        }

        tracing::info!("Completed testing get_l1_info_root for all leaf counts");
    }

    #[test_log::test(tokio::test)]
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

    #[test_log::test(tokio::test)]
    #[ignore = "reaches external endpoint"]
    async fn test_create_l1_rpc_client_for_bali_testnet() {
        use url::Url;

        // Use L1_RPC_ENDPOINT environment variable (should be set to Sepolia endpoint)
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

        tracing::info!("Test fetching of the InitL1InfoRootMap for Bali testnet");

        // Create L1RpcClient with default config for other parameters for Bali testnet
        // InitL1InfoRootMap event is on block 6487027
        let contracts = ContractSetup::new();
        let _l1_rpc = L1RpcClient::try_new(
            Arc::new(rpc.clone()),
            contracts::PolygonRollupManager::new(contracts.rollup_manager, rpc),
            contracts.ger_contract,
            100, // default gas_multiplier_factor
            GasPriceParams::default(),
            10000, // default event_filter_block_range
        )
        .await
        .expect("Failed to create L1RpcClient");
    }

    #[rstest::rstest]
    fn test_adjust_gas_estimate_respects_floor_and_ceiling(
        #[values(500, 1000, 1500, 2000)] multiplier_per_1000: u64,
        #[values(10_000_000, 50_000_000)] floor: u128,
        #[values(100_000_000, 200_000_000)] ceiling: u128,
        #[values(10_000_000, 100_000_000, 200_000_000)] max_fee_per_gas: u128,
        #[values(5_000_000, 50_000_000, 100_000_000)] max_priority_fee_per_gas: u128,
    ) {
        let estimate = Eip1559Estimation {
            max_fee_per_gas,
            max_priority_fee_per_gas,
        };
        let params = GasPriceParams {
            multiplier_per_1000,
            floor,
            ceiling,
        };

        let adjusted = adjust_gas_estimate(&estimate, &params);

        let acceptable_fee = floor..=ceiling;
        assert!(
            acceptable_fee.contains(&adjusted.max_fee_per_gas),
            "max_fee_per_gas {} is out of range {acceptable_fee:?}",
            adjusted.max_fee_per_gas,
        );

        let acceptable_priority_fee = 0..=ceiling;
        assert!(
            acceptable_priority_fee.contains(&adjusted.max_priority_fee_per_gas),
            "max_priority_fee_per_gas {} out of range {acceptable_priority_fee:?}",
            adjusted.max_priority_fee_per_gas,
        );

        // Some extra tests for scaling factor = 1.0
        if multiplier_per_1000 == 1000 {
            let acceptable_fee = [floor, max_fee_per_gas, ceiling];
            assert!(acceptable_fee.contains(&adjusted.max_fee_per_gas));

            let acceptable_priority_fee = [max_priority_fee_per_gas, ceiling];
            assert!(acceptable_priority_fee.contains(&adjusted.max_priority_fee_per_gas));
        }
    }
}
