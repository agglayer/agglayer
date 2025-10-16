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
        tx_hash: B256,
    ) -> Result<TransactionReceipt, L1RpcError>;

    /// Returns the provider for direct access to watch transactions
    fn get_provider(&self) -> &Self::Provider;

    /// Finds the block number where a contract was deployed.
    /// This is used to optimize event queries by starting from the deployment
    /// block.
    async fn find_contract_deployment_block_number(
        &self,
        address: Address,
    ) -> Result<Option<u64>, L1RpcError>;
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
    /// PolygonZkEVMGlobalExitRootV2 contract deployment block number.
    global_exit_root_manager_contract_block: u64,
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

/// Finds the block number where a contract was deployed using binary search.
/// This is used to optimize event queries by starting from the deployment
/// block.
async fn find_contract_deployment_block_number<RpcProvider>(
    rpc: &RpcProvider,
    address: Address,
) -> Result<Option<u64>, L1RpcError>
where
    RpcProvider: alloy::providers::Provider,
{
    // Get the latest block number as the upper bound for search
    let latest_block = rpc.get_block_number().await.map_err(|error| {
        error!(?error, "Failed to get the latest block number");
        L1RpcError::FailedToQueryEvents(format!("Failed to get latest block number: {error}"))
    })?;

    debug!("Finding contract deployment block for address: {address}, latest_block {latest_block}");

    let mut hi = latest_block;
    let mut lo = 0u64;

    // Quick exit: if no code at latest block, contract was never deployed
    // (Note: this will treat self-destructed contracts as "not found")
    let latest_code = rpc.get_code_at(address).number(hi).await.map_err(|error| {
        error!(?error, "Failed to get code at latest block");
        L1RpcError::FailedToQueryEvents(format!("Failed to get code at latest block: {error}"))
    })?;

    if latest_code.is_empty() {
        debug!("No code found at latest block for address: {}", address);
        return Ok(None);
    }

    debug!(
        "Contract has code at latest block, starting binary search from block 0 to {}",
        hi
    );

    // Binary search for the first block where code is present
    let mut answer: Option<u64> = None;
    let mut iterations = 0;
    const MAX_ITERATIONS: u32 = 32; // Safety limit to prevent infinite loops

    while lo <= hi && iterations < MAX_ITERATIONS {
        iterations += 1;
        let mid = lo + ((hi - lo) / 2);

        debug!(
            "Binary search iteration {}: checking block {} (range: {} to {})",
            iterations, mid, lo, hi
        );

        // Query bytecode at specific block height
        let code_at_mid = rpc
            .get_code_at(address)
            .number(mid)
            .await
            .map_err(|error| {
                error!(?error, "Failed to get code at block {mid}");
                L1RpcError::FailedToQueryEvents(format!(
                    "Failed to get code at block {mid}: {error}"
                ))
            })?;

        if code_at_mid.is_empty() {
            // Not deployed yet at `mid` -> search higher
            lo = mid.saturating_add(1);
        } else {
            // Code exists at `mid` -> remember and search lower
            answer = Some(mid);
            if mid == 0 {
                break;
            }
            hi = mid - 1;
        }
    }

    if iterations >= MAX_ITERATIONS {
        error!(
            "Binary search exceeded maximum iterations ({}), returning partial result",
            MAX_ITERATIONS
        );
    }

    answer.as_ref().map_or_else(
        || debug!(?address, "Contract deployment block not found"),
        |block| debug!("Contract deployment found at block: {block}"),
    );
    Ok(answer)
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
        global_exit_root_manager_contract_block: u64,
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
            global_exit_root_manager_contract_block,
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

        // Find the deployment block of the global exit root manager to optimize
        // InitL1InfoRootMap event search.
        let global_exit_root_manager_contract_block =
            find_contract_deployment_block_number(&*rpc, global_exit_root_manager_contract)
                .await
                .map_err(|error| {
                    error!(
                        ?error,
                        "Failed to find contract {} deployment block",
                        global_exit_root_manager_contract
                    );
                    L1RpcInitializationError::InitL1InfoRootMapEventNotFound(error.to_string())
                })?
                .unwrap_or_default();

        let default_l1_info_tree_entry = {
            // Start search from deployment block or genesis if not found
            let mut start_block = global_exit_root_manager_contract_block;
            debug!(
                "Starting InitL1InfoRootMap event search from block: {start_block}, contract \
                 address: {global_exit_root_manager_contract}"
            );

            let mut events = Vec::new();
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
                    .address(global_exit_root_manager_contract)
                    .event_signature(InitL1InfoRootMap::SIGNATURE_HASH)
                    .from_block(BlockNumberOrTag::Number(start_block))
                    .to_block(BlockNumberOrTag::Number(end_block));

                // Get logs from the contract
                events = rpc.get_logs(&filter).await.map_err(|error| {
                    error!(?error, "Failed to get InitL1InfoRootMap events");
                    L1RpcInitializationError::InitL1InfoRootMapEventNotFound(error.to_string())
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
            global_exit_root_manager_contract_block,
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

    async fn find_contract_deployment_block_number(
        &self,
        address: Address,
    ) -> Result<Option<u64>, L1RpcError> {
        find_contract_deployment_block_number(&*self.rpc, address).await
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

    #[test_log::test(tokio::test)]
    #[ignore = "reaches external endpoint"]
    async fn test_fetch_proper_default_l1_global_exit_root_manager_sepolia() {
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

        let contracts = crate::tests::ContractSetup::new();
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

    #[rstest::rstest]
    #[case(
        "0xE2EF6215aDc132Df6913C8DD16487aBF118d1764",
        Some(4794475),
        "Contract 1"
    )]
    #[case(
        "0x1348947e282138d8f377b467F7D9c2EB0F335d1f",
        Some(4794471),
        "Contract 2"
    )]
    #[case(
        "0x2968D6d736178f8FE7393CC33C87f29D9C287e78",
        Some(4794473),
        "Contract 3"
    )]
    #[case(
        "0x528e26b25a34a4A5d0dbDa1d57D318153d2ED582",
        Some(4789186),
        "Contract 4"
    )]
    #[case(
        "0xfd8ACe213595faC05d45714e8e2a63Df267E3545",
        Some(4789191),
        "Contract 5"
    )]
    #[case("0x0000000000000000000000000000000000000001", None, "Non existing")]
    #[case("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045", None, "Vitalik")]
    #[case("0x3e622317f8C93f7328350cF0B56d9eD4C620C5d6", Some(3218325), "DAI")]
    #[case(
        "0xB26B2De65D07eBB5E54C7F6282424D3be670E1f0",
        Some(3279404),
        "Uniswap V2"
    )]
    #[case("0x9A05509488486D2DC25BFb875304ff378d76Fab3", Some(4599573), "CreateX")]
    #[case("0xEB590e5A96CD0E943A0899412E4fB06e0B362a7f", Some(4898155), "Weth")]
    #[case(
        "0x18Ea3C01215880a282D50eB398ddfDB3937E5A5b",
        Some(9304458),
        "DailyCheckin"
    )]
    #[test_log::test(tokio::test)]
    #[ignore = "reaches external endpoint"]
    async fn test_find_contract_deployment_block_number_sepolia(
        #[case] address: &str,
        #[case] expected_block: Option<u64>,
        #[case] name: &str,
    ) {
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

        // Create a minimal L1RpcClient for testing
        let dummy_rollup_manager: Address = "0x0000000000000000000000000000000000000000"
            .parse()
            .unwrap();
        let dummy_ger_contract: Address = "0x0000000000000000000000000000000000000000"
            .parse()
            .unwrap();

        let l1_rpc = L1RpcClient::new(
            Arc::new(rpc.clone()),
            contracts::PolygonRollupManager::new(dummy_rollup_manager, rpc),
            dummy_ger_contract,
            (0, [0u8; 32]),
            100,
            GasPriceParams::default(),
            10000,
            0,
        );

        let contract_address: Address = address.parse().expect("Invalid contract address");

        let deployment_block = l1_rpc
            .find_contract_deployment_block_number(contract_address)
            .await
            .unwrap_or_else(|_| panic!("Failed to check contract {name} at {address}"));

        assert_eq!(
            deployment_block, expected_block,
            "Contract {name} at {address} should have deployment block {expected_block:?}"
        );
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

        // Use the specified contract addresses for Bali testnet
        let rollup_manager_address: Address = "0xE2EF6215aDc132Df6913C8DD16487aBF118d1764"
            .parse()
            .expect("Invalid rollup manager address");
        let global_exit_root_manager_address: Address =
            "0x2968D6d736178f8FE7393CC33C87f29D9C287e78"
                .parse()
                .expect("Invalid PolygonZkEVMGlobalExitRootV2 address");

        // Create L1RpcClient with default config for other parameters for Bali testnet
        // InitL1InfoRootMap event is on block 6487027
        let l1_rpc = L1RpcClient::try_new(
            Arc::new(rpc.clone()),
            contracts::PolygonRollupManager::new(rollup_manager_address, rpc),
            global_exit_root_manager_address,
            100, // default gas_multiplier_factor
            GasPriceParams::default(),
            10000, // default event_filter_block_range
        )
        .await
        .expect("Failed to create L1RpcClient");

        // Test get_l1_info_root for specific leaf counts from log output
        let leaf_counts = vec![
            68, 71, 74, 79, 82, 85, 88, 90, 95, 98, 101, 104, 107, 112, 115, 118, 121, 123, 128,
            131, 134, 137,
        ];

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
}
