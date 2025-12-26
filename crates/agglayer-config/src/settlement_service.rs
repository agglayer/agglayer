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
/// # Variants
///
/// - **Immediate**: Transaction is considered settled after the specified number
///   of `confirmations` blocks. Provides fastest settlement but lower security.
/// - **Justified**: Transaction is considered settled when the block containing
///   it has been justified by Ethereum's Casper FFG finality gadget.
/// - **Finalized**: Transaction is considered settled only when the block has
///   been finalized by Casper FFG, providing the strongest security guarantee.
///
/// # Example
///
/// ```toml
/// [settlement.certificate-tx-config]
/// confirmations = 16
/// finality = "justified"
///
/// [settlement.validium-tx-config]
/// confirmations = 6
/// finality = "immediate"  # Faster settlement for validium data
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Finality {
    /// Transaction is considered settled immediately after the specified
    /// number of confirmation blocks.
    ///
    /// **Security**: Vulnerable to chain reorganizations beyond the
    /// confirmation count.
    Immediate,

    /// Transaction is considered settled when the containing block has been
    /// justified by Casper FFG.
    ///
    /// **Security**: Very strong. Reversing a justified block would require
    /// a significant portion of validators to be slashed.
    ///
    /// **Time**: Typically up to ~7 minutes on mainnet. Worst case scenario 12-13 minutes.
    #[default]
    Justified,

    /// Transaction is considered settled only when the containing block has
    /// been finalized by Casper FFG.
    ///
    /// **Time**: Typically between 7-13 minutes on mainnet. Worst case scenario ~19 minutes.
    Finalized,
}

/// Transaction retry policy for failed or pending settlement transactions.
///
/// Defines the strategy used when retrying failed or pending transactions.
/// The retry policy works in conjunction with `tx_retry_interval` to determine
/// the timing between retry attempts.
///
/// # Variants
///
/// - **Linear**: Retries are attempted at fixed intervals defined by
///   `tx_retry_interval`. For example, with a 10-second interval, retries
///   occur at: 0s, 10s, 20s, 30s, etc.
///
/// # Future Extensions
///
/// Future versions may include additional policies such as:
/// - **Exponential**: Exponential backoff with increasing intervals
/// - **Jittered**: Random jitter to avoid thundering herd issues
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub enum TxRetryPolicy {
    /// Linear retry policy with fixed intervals between attempts.
    ///
    /// This is the default and recommended policy for most use cases.
    /// It provides predictable retry behavior and works well with the
    /// settlement service's transaction monitoring.
    #[default]
    Linear,
}

/// The settlement transaction configuration.
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub struct SettlementTransactionConfig {
    /// Maximum number of retries for the transaction.
    /// Expected to be a big number.
    #[serde(default = "default_rpc_max_retries")]
    pub max_retries: usize,

    /// Retry approach for the transaction.
    #[serde(default = "default_rpc_retry_policy")]
    pub tx_retry_policy: TxRetryPolicy,

    /// Retry interval.
    /// Used together with `tx_retry_policy`.
    #[serde(default = "default_tx_retry_interval")]
    #[serde(with = "crate::with::HumanDuration")]
    pub tx_retry_interval: Duration,

    /// Number of block confirmations required for
    /// the transaction to resolve a receipt.
    #[serde(default = "default_rpc_confirmations")]
    pub confirmations: usize,

    /// Finality level required for the transaction to be considered settled.
    #[serde(default)]
    pub finality: Finality,

    /// Gas multiplier factor for the transaction.
    /// The gas is calculated as follows:
    /// `gas = estimated_gas * gas_multiplier_factor`
    #[serde(default, skip_serializing_if = "crate::is_default")]
    pub gas_multiplier_factor: Multiplier,

    /// Gas limit for the transaction.
    #[serde(default = "default_gas_limit")]
    pub gas_limit: U256,

    /// Gas price multiplier for the transaction.
    /// The gas price is calculated as follows:
    /// `gas_price = estimate_gas_price * gas_price_multiplier_factor`
    #[serde(default, skip_serializing_if = "crate::is_default")]
    pub gas_price_multiplier_factor: Multiplier,

    /// Minimum gas price floor (in wei) for the transaction.
    /// Can be specified with units: "1gwei", "0.1eth", "1000000000wei".
    /// Use for both EIP1559 fee and priority fee.
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

/// The settlement service configuration.
///
/// Contains service-wide configuration options for the Agglayer settlement service.
/// This configuration is separate from transaction-specific settings and focuses
/// on overall service behavior and integration points.
///
/// # Future Configuration Options
///
/// This structure is designed to hold settlement service-specific values that
/// are not related to individual transactions. The service relies on other
/// configuration sources for:
///
/// - **L1 Provider**: RPC connection to the L1 blockchain
/// - **Agglayer Contracts**: Smart contract addresses and ABIs
/// - **Transaction Signer**: Private keys or KMS configuration for signing
///
/// Future additions to this configuration may include:
/// - Service health check intervals
/// - Monitoring and metrics endpoints
/// - Settlement batching strategies
/// - Emergency shutdown conditions
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
    // todo: settlement service will use L1 provider, agglayer contracts,
    // transaction signer etc. already configured on other places.
    // This structure should only have settlement service specific values.
}


/// The Agglayer settlement configuration.
///
/// This configuration controls how the Agglayer settlement service interacts with
/// the L1 blockchain for settling certificates and validium transactions. It provides
/// separate transaction configurations for certificate settlements and validium
/// settlements, allowing fine-grained control over gas prices, retries, and
/// confirmation requirements.
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
/// [settlement.certificate-tx-config]
/// max-retries = 1024
/// tx-retry-interval = "10s"
/// confirmations = 32
/// finality = "finalized"
/// gas-limit = 60000000
/// gas-price-ceiling = "100gwei"
///
/// [settlement.certificate-tx-config.gas-multiplier-factor]
/// numerator = 11
/// denominator = 10
///
/// [settlement.certificate-tx-config.gas-price-multiplier-factor]
/// numerator = 12
/// denominator = 10
///
/// [settlement.validium-tx-config]
/// max-retries = 512
/// tx-retry-interval = "5s"
/// confirmations = 16
/// finality = "justified"
/// gas-limit = 30000000
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
    certificate_tx_config: SettlementTransactionConfig,

    /// Configuration for validium settlement transactions.
    ///
    /// This controls how validium data (off-chain data availability proofs)
    /// are submitted to the L1 settlement layer. Validium transactions may
    /// have different gas and retry requirements than certificate transactions.
    #[serde(default)]
    validium_tx_config: SettlementTransactionConfig,

    /// General settlement service configuration.
    ///
    /// Contains service-wide settings that apply to the overall settlement
    /// service operation (beyond individual transaction parameters).
    #[serde(default)]
    settlement_service_config: SettlementServiceConfig,
}

/// Default number of retries for the transaction.
const fn default_rpc_max_retries() -> usize {
    1024
}

/// Default interval for the polling of the transaction.
const fn default_tx_retry_interval() -> Duration {
    Duration::from_secs(10)
}

const fn default_rpc_retry_policy() -> TxRetryPolicy {
    TxRetryPolicy::Linear
}

/// Default number of confirmations required
/// for the transaction to resolve a receipt.
const fn default_rpc_confirmations() -> usize {
    32
}

fn default_gas_limit() -> U256 {
    U256::from(60_000_000_u64)
}

/// Default gas price ceiling for the transaction.
const fn default_gas_price_ceiling() -> u128 {
    // 100 gwei
    100_000_000_000_u128
}
