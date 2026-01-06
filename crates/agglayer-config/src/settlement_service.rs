use std::time::Duration;

use agglayer_primitives::U256;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::Multiplier;

/// Finality level for settlement transactions on Ethereum.
///
/// Defines when a transaction should be considered settled on the Ethereum
/// network, determining the security guarantees for the settlement operation.
///
/// # Example
///
/// ```toml
/// [settlement.pessimistic-proof-tx-config]
/// confirmations = 16
/// finality = "SafeBlock"
///
/// [settlement.validium-tx-config]
/// confirmations = 6
/// finality = "LatestBlock"  # Faster settlement for validium data
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub enum Finality {
    /// Transaction is considered settled immediately after the specified
    /// number of confirmation blocks.
    ///
    /// **Security**: Vulnerable to chain reorganizations beyond the
    /// confirmation count.
    LatestBlock,

    /// Transaction is considered settled when the containing block has been
    /// considered "safe."
    ///
    /// **Security**: Very strong. Reversing a safe block would require
    /// a significant portion of validators to be slashed.
    ///
    /// **Time**: Typically up to ~7 minutes on mainnet. Worst case scenario
    /// 12-13 minutes.
    #[default]
    SafeBlock,

    /// Transaction is considered settled only when the containing block has
    /// been fully finalized.
    ///
    /// **Time**: Typically between 7-13 minutes on mainnet. Worst case scenario
    /// ~19 minutes.
    FinalizedBlock,
}

/// Transaction retry policy for failed or pending settlement transactions.
///
/// Defines the strategy used when retrying failed or pending transactions.
///
/// Future versions may include additional policies such as:
/// - **Exponential**: Exponential backoff with increasing intervals
/// - **Jittered**: Random jitter to avoid thundering herd issues
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub struct TxRetryPolicy {
    /// Initial retry interval.
    #[serde(with = "crate::with::HumanDuration")]
    pub initial_interval: Duration,

    /// Interval multiplier for each subsequent retry.
    #[serde(default, skip_serializing_if = "crate::is_default")]
    pub interval_multiplier_factor: Multiplier,

    /// Maximum interval between retries.
    #[serde(default = "default_max_interval")]
    #[serde(with = "crate::with::HumanDuration")]
    pub max_interval: Duration,

    /// Jitter factor to add randomness to retry intervals.
    #[serde(default, skip_serializing_if = "crate::is_default")]
    #[serde(with = "crate::with::HumanDuration")]
    pub jitter: Duration,
}

/// The settlement transaction configuration.
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct SettlementTransactionConfig {
    /// Maximum number of retries for the transaction.
    /// Expected to be a big number.
    #[serde(default = "default_rpc_max_retries")]
    pub max_retries: usize,

    /// Retry policy for the transaction when there is a transient failure.
    #[serde(default = "default_transient_rpc_retry_policy")]
    pub retry_on_transient_failure: TxRetryPolicy,

    /// Retry policy for the transaction when it is not included on L1.
    #[serde(default = "default_non_inclusion_rpc_retry_policy")]
    pub retry_on_not_included_on_l1: TxRetryPolicy,

    /// Number of block confirmations required for
    /// the transaction to resolve a receipt.
    #[serde(default = "default_confirmations")]
    pub confirmations: usize,

    /// Finality level required for the transaction to be considered settled.
    #[serde(default)]
    pub finality: Finality,

    /// Gas limit multiplier factor for the transaction.
    /// The gas is calculated as follows:
    /// `gas = estimated_gas * gas_multiplier_factor`
    #[serde(default, skip_serializing_if = "crate::is_default")]
    pub gas_limit_multiplier_factor: Multiplier,

    /// Ceiling for the gas limit for the transaction.
    #[serde(default = "default_gas_limit_ceiling")]
    pub gas_limit_ceiling: U256,

    /// Gas price multiplier for the transaction.
    /// The gas price is calculated as follows:
    /// `gas_price = estimate_gas_price * gas_price_multiplier_factor`
    /// Used for both EIP1559 fee and priority fee.
    #[serde(default, skip_serializing_if = "crate::is_default")]
    pub gas_price_multiplier_factor: Multiplier,

    /// Minimum gas price floor (in wei) for the transaction.
    /// Can be specified with units: "1gwei", "0.1eth", "1000000000wei".
    /// Used for both EIP1559 fee and priority fee.
    #[serde(default, skip_serializing_if = "crate::is_default")]
    #[serde_as(as = "crate::with::EthAmount")]
    pub gas_price_floor: u128,

    /// Maximum gas price ceiling (in wei) for the transaction.
    /// Use for both EIP1559 `max_fee_per_gas` and `max_priority_fee_per_gas`.
    /// Can be specified with units: "100gwei", "0.01eth", "10000000000wei"
    #[serde(default = "default_gas_price_ceiling")]
    #[serde_as(as = "crate::with::EthAmount")]
    pub gas_price_ceiling: u128,
}

impl Default for SettlementTransactionConfig {
    fn default() -> Self {
        Self {
            max_retries: default_rpc_max_retries(),
            retry_on_transient_failure: default_transient_rpc_retry_policy(),
            retry_on_not_included_on_l1: default_non_inclusion_rpc_retry_policy(),
            confirmations: default_confirmations(),
            finality: Finality::default(),
            gas_limit_multiplier_factor: Multiplier::default(),
            gas_limit_ceiling: default_gas_limit_ceiling(),
            gas_price_multiplier_factor: Multiplier::default(),
            gas_price_floor: 0,
            gas_price_ceiling: default_gas_price_ceiling(),
        }
    }
}

/// The settlement service configuration.
///
/// Contains service-wide configuration options for the Agglayer settlement
/// service. This configuration is separate from transaction-specific settings
/// and focuses on overall service behavior and integration points.
///
/// This structure is designed to hold settlement service-specific values that
/// are not related to individual transactions.
///
/// # Example TOML Configuration
///
/// ```toml
/// [settlement.settlement-service-config]
/// # Currently no service-specific fields
/// # Future fields will be added here as the service evolves
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub struct SettlementServiceConfig {
    // TODO: settlement service will use L1 provider, agglayer contracts,
    // transaction signer etc. already configured on other places.
    // This structure should only have settlement service specific values.
    // Is there any? If not we can remove this struct, but let's keep it for
    // now to help parallelize work.
}

/// The Agglayer settlement configuration.
///
/// This configuration controls how the Agglayer settlement service interacts
/// with the L1 blockchain for settling certificates and validium transactions.
/// It provides separate transaction configurations for certificate settlements
/// and validium settlements, allowing fine-grained control over gas prices,
/// retries, and confirmation requirements.
///
/// # Configuration Structure
///
/// The settlement configuration is organized into three main components:
///
/// - **Certificate Transaction Config**: Controls settlement transactions for
///   certificates with pessimistic proofs (proofs of state transitions).
/// - **Validium Transaction Config**: Controls settlement transactions for
///   validium data (off-chain data availability).
/// - **Settlement Service Config**: General settlement service configuration.
///
/// # Example TOML Configuration
///
/// ```toml
/// [settlement]
/// [settlement.pessimistic-proof-tx-config]
/// max-retries = 1024
/// tx-retry-interval = "10s"
/// confirmations = 32
/// finality = "finalized"
/// gas-limit = 60000000
/// gas-multiplier-factor = 1.1
/// gas-price-multiplier-factor = 1.2
/// gas-price-ceiling = "100gwei"
///
/// [settlement.validium-tx-config]
/// max-retries = 512
/// tx-retry-interval = "5s"
/// confirmations = 16
/// finality = "justified"
/// gas-limit = 30000000
/// gas-multiplier-factor = 1.05
/// gas-price-floor = "1gwei"
/// gas-price-ceiling = "50gwei"
/// ```
///
/// # Gas Price Configuration
///
/// Gas prices can be specified with units for readability:
/// - `"1gwei"` = 1,000,000,000 wei
/// - `"0.1eth"` = 100,000,000,000,000,000 wei
/// - `"1000000000wei"` = 1,000,000,000 wei
///
/// The final gas price is calculated as:
/// ```text
/// gas_price = max(
///     floor,
///     min(ceiling, estimate_gas_price * gas_price_multiplier_factor)
/// )
/// ```
///
/// Each retry can increase the price by the multiplier factor.
///
/// # Security Considerations
///
/// - **Finality Level**: Choose the appropriate finality level based on
///   security requirements.
/// - **Gas Price Ceiling**: Acts as a safety mechanism to prevent excessive
///   transaction costs during network congestion.
/// - **Confirmations**: Higher confirmation counts increase security for
///   receipt retrieval but do not replace proper finality guarantees.
/// - **Max Retries**: Should be set high enough to handle temporary network
///   issues.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub struct SettlementConfig {
    /// Configuration for certificate settlement transactions.
    ///
    /// This controls how certificates (proofs of state transitions) are
    /// submitted to the L1 settlement layer.
    #[serde(default)]
    pub pessimistic_proof_tx_config: SettlementTransactionConfig,

    /// Configuration for validium settlement transactions.
    ///
    /// This controls how validium data (off-chain data availability proofs)
    /// are submitted to the L1 settlement layer. Validium transactions may
    /// have different gas and retry requirements than certificate transactions.
    #[serde(default)]
    pub validium_tx_config: SettlementTransactionConfig,

    /// General settlement service configuration.
    ///
    /// Contains service-wide settings that apply to the overall settlement
    /// service operation (beyond individual transaction parameters).
    #[serde(default)]
    pub settlement_service_config: SettlementServiceConfig,
}

const fn default_rpc_max_retries() -> usize {
    16 * 1024
}

const fn default_max_interval() -> Duration {
    Duration::from_secs(60 * 60 * 24 * 36525) // effectively infinite: ~100
                                              // years
}

const fn default_transient_rpc_retry_policy() -> TxRetryPolicy {
    TxRetryPolicy {
        initial_interval: Duration::from_secs(10),
        interval_multiplier_factor: Multiplier::from_u64_per_1000(1500),
        max_interval: Duration::from_secs(120),
        jitter: Duration::from_secs(1),
    }
}

const fn default_non_inclusion_rpc_retry_policy() -> TxRetryPolicy {
    TxRetryPolicy {
        initial_interval: Duration::from_secs(60),
        interval_multiplier_factor: Multiplier::from_u64_per_1000(2000),
        max_interval: Duration::from_secs(600),
        jitter: Duration::from_secs(10),
    }
}

/// Default number of confirmations required
/// for the transaction to resolve a receipt.
const fn default_confirmations() -> usize {
    32
}

fn default_gas_limit_ceiling() -> U256 {
    U256::from(60_000_000_u64)
}

/// Default gas price ceiling for the transaction.
const fn default_gas_price_ceiling() -> u128 {
    // 100 gwei
    100_000_000_000_u128
}
