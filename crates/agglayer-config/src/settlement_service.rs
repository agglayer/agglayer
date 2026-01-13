use std::time::Duration;

use agglayer_primitives::U256;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, serde_conv};

use crate::{with::HumanDuration, Multiplier};

/// Policy for considering the transactions as settled on Ethereum.
///
/// Defines when a transaction should be considered settled on the Ethereum
/// network, determining the security guarantees for the settlement operation.
///
/// This matches the `latest`, `safe`, and `finalized` block concepts from
/// Ethereum clients, exposed over the JSON-RPC API.
#[derive(Serialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum SettlementPolicy {
    /// Transaction is considered settled immediately after the specified
    /// number of confirmation blocks.
    ///
    /// **Security**: Vulnerable to chain reorganizations beyond the
    /// confirmation count.
    ///
    /// Can be configured as:
    /// - `"latest-block/N"` where N is the number of confirmations
    /// - `{ latest-block = { confirmations = N } }`
    LatestBlock {
        /// Number of block confirmations required for the transaction
        /// to be considered settled.
        #[serde(default = "default_latest_block_confirmations")]
        confirmations: usize,
    },

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

impl SettlementPolicy {
    /// Returns the number of confirmations required for the policy.
    ///
    /// - For `LatestBlock`, returns the configured confirmation count.
    /// - For `SafeBlock` and `FinalizedBlock`, returns `None` as these policies
    ///   rely on Ethereum's built-in finality mechanisms.
    pub fn confirmations(&self) -> Option<usize> {
        match self {
            SettlementPolicy::LatestBlock { confirmations } => Some(*confirmations),
            SettlementPolicy::SafeBlock | SettlementPolicy::FinalizedBlock => None,
        }
    }
}

impl<'de> Deserialize<'de> for SettlementPolicy {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use std::fmt;

        use serde::de::{self, MapAccess, Visitor};

        struct SettlementPolicyVisitor;

        impl<'de> Visitor<'de> for SettlementPolicyVisitor {
            type Value = SettlementPolicy;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str(
                    "a settlement policy: \"safe-block\", \"finalized-block\", \
                     \"latest-block/N\", or a table with policy details",
                )
            }

            fn visit_str<E>(self, value: &str) -> Result<SettlementPolicy, E>
            where
                E: de::Error,
            {
                match value {
                    "safe-block" => Ok(SettlementPolicy::SafeBlock),
                    "finalized-block" => Ok(SettlementPolicy::FinalizedBlock),
                    s if s.starts_with("latest-block/") => {
                        let confirmations_str = s.strip_prefix("latest-block/").unwrap();
                        let confirmations = confirmations_str.parse::<usize>().map_err(|_| {
                            E::custom(format!(
                                "invalid confirmations value '{}' in 'latest-block/N'",
                                confirmations_str
                            ))
                        })?;
                        Ok(SettlementPolicy::LatestBlock { confirmations })
                    }
                    "latest-block" => {
                        // Support "latest-block" without confirmations, use default
                        Ok(SettlementPolicy::LatestBlock {
                            confirmations: default_latest_block_confirmations(),
                        })
                    }
                    _ => Err(E::custom(format!(
                        "unknown settlement policy '{}', expected 'safe-block', \
                         'finalized-block', 'latest-block', or 'latest-block/N'",
                        value
                    ))),
                }
            }

            fn visit_map<M>(self, mut map: M) -> Result<SettlementPolicy, M::Error>
            where
                M: MapAccess<'de>,
            {
                #[derive(Deserialize)]
                #[serde(rename_all = "kebab-case")]
                struct LatestBlockConfig {
                    #[serde(default = "default_latest_block_confirmations")]
                    confirmations: usize,
                }

                let key: String = map
                    .next_key()?
                    .ok_or_else(|| de::Error::custom("expected a policy key"))?;

                match key.as_str() {
                    "safe-block" => {
                        // Consume the value (should be empty or unit)
                        let _: Option<()> = map.next_value().ok();
                        Ok(SettlementPolicy::SafeBlock)
                    }
                    "finalized-block" => {
                        // Consume the value (should be empty or unit)
                        let _: Option<()> = map.next_value().ok();
                        Ok(SettlementPolicy::FinalizedBlock)
                    }
                    "latest-block" => {
                        let config: LatestBlockConfig = map.next_value()?;
                        Ok(SettlementPolicy::LatestBlock {
                            confirmations: config.confirmations,
                        })
                    }
                    _ => Err(de::Error::custom(format!(
                        "unknown settlement policy '{}', expected 'safe-block', \
                         'finalized-block', or 'latest-block'",
                        key
                    ))),
                }
            }
        }

        deserializer.deserialize_any(SettlementPolicyVisitor)
    }
}

/// Transaction retry policy.
#[serde_as]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
struct ConfigTxRetryPolicy {
    /// Initial retry interval.
    #[serde(default, skip_serializing_if = "crate::is_default")]
    #[serde_as(as = "Option<HumanDuration>")]
    initial_interval: Option<Duration>,

    /// Interval multiplier for each subsequent retry.
    #[serde(default, skip_serializing_if = "crate::is_default")]
    interval_multiplier_factor: Option<Multiplier>,

    /// Maximum interval between retries.
    #[serde(default, skip_serializing_if = "crate::is_default")]
    #[serde_as(as = "Option<HumanDuration>")]
    max_interval: Option<Duration>,

    /// Jitter factor to add randomness to retry intervals.
    #[serde(default, skip_serializing_if = "crate::is_default")]
    #[serde_as(as = "Option<HumanDuration>")]
    jitter: Option<Duration>,
}

impl From<&TxRetryPolicy> for ConfigTxRetryPolicy {
    fn from(value: &TxRetryPolicy) -> Self {
        ConfigTxRetryPolicy {
            initial_interval: Some(value.initial_interval),
            interval_multiplier_factor: Some(value.interval_multiplier_factor),
            max_interval: Some(value.max_interval),
            jitter: Some(value.jitter),
        }
    }
}

serde_conv!(pub TransientRetryPolicy, TxRetryPolicy, TransientRetryPolicyImpl::new, TransientRetryPolicyImpl::get);

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct TransientRetryPolicyImpl(ConfigTxRetryPolicy);

impl TransientRetryPolicyImpl {
    fn new(value: &TxRetryPolicy) -> Self {
        TransientRetryPolicyImpl(ConfigTxRetryPolicy::from(value))
    }

    pub fn get(self) -> Result<TxRetryPolicy, std::convert::Infallible> {
        Ok(TxRetryPolicy {
            initial_interval: self.0.initial_interval.unwrap_or(Duration::from_secs(10)),
            interval_multiplier_factor: self
                .0
                .interval_multiplier_factor
                .unwrap_or(Multiplier::from_u64_per_1000(1500)),
            max_interval: self.0.max_interval.unwrap_or(Duration::from_secs(120)),
            jitter: self.0.jitter.unwrap_or(Duration::from_secs(1)),
        })
    }
}

serde_conv!(pub NonInclusionRetryPolicy, TxRetryPolicy, NonInclusionRetryPolicyImpl::new, NonInclusionRetryPolicyImpl::get);

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct NonInclusionRetryPolicyImpl(ConfigTxRetryPolicy);

impl NonInclusionRetryPolicyImpl {
    fn new(value: &TxRetryPolicy) -> Self {
        NonInclusionRetryPolicyImpl(ConfigTxRetryPolicy::from(value))
    }

    pub fn get(self) -> Result<TxRetryPolicy, std::convert::Infallible> {
        Ok(TxRetryPolicy {
            initial_interval: self.0.initial_interval.unwrap_or(Duration::from_secs(60)),
            interval_multiplier_factor: self
                .0
                .interval_multiplier_factor
                .unwrap_or(Multiplier::from_u64_per_1000(2000)),
            max_interval: self.0.max_interval.unwrap_or(Duration::from_secs(600)),
            jitter: self.0.jitter.unwrap_or(Duration::from_secs(10)),
        })
    }
}

/// Transaction retry policy for failed or pending settlement transactions.
///
/// Defines the strategy used when retrying failed or pending transactions.
///
/// Future versions may include additional policies such as:
/// - **Exponential**: Exponential backoff with increasing intervals
/// - **Jittered**: Random jitter to avoid thundering herd issues
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
pub struct TxRetryPolicy {
    /// Initial retry interval.
    pub initial_interval: Duration,

    /// Interval multiplier for each subsequent retry.
    pub interval_multiplier_factor: Multiplier,

    /// Maximum interval between retries.
    pub max_interval: Duration,

    /// Jitter factor to add randomness to retry intervals.
    pub jitter: Duration,
}

impl TxRetryPolicy {
    /// Default retry policy for transient failures.
    pub fn default_on_transient_failure() -> Self {
        TransientRetryPolicyImpl(ConfigTxRetryPolicy::default())
            .get()
            .unwrap()
    }

    /// Default retry policy for non-inclusion on L1.
    pub fn default_non_inclusion_on_l1() -> Self {
        NonInclusionRetryPolicyImpl(ConfigTxRetryPolicy::default())
            .get()
            .unwrap()
    }
}

/// The settlement transaction configuration.
#[serde_as]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct SettlementTransactionConfig {
    /// Maximum number of retries for the transaction.
    /// Expected to be a big number.
    #[serde(default = "default_rpc_max_expected_retries")]
    pub max_expected_retries: usize,

    /// Retry policy for the transaction when there is a transient failure.
    #[serde(default = "TxRetryPolicy::default_on_transient_failure")]
    #[serde_as(as = "TransientRetryPolicy")]
    pub retry_on_transient_failure: TxRetryPolicy,

    /// Retry policy for the transaction when it is not included on L1.
    #[serde(default = "TxRetryPolicy::default_non_inclusion_on_l1")]
    #[serde_as(as = "NonInclusionRetryPolicy")]
    pub retry_on_not_included_on_l1: TxRetryPolicy,

    /// Finality level required for the transaction to be considered settled.
    ///
    /// For `LatestBlock` policy, this also includes the number of
    /// confirmations. For `SafeBlock` and `FinalizedBlock`, confirmations
    /// are not applicable.
    #[serde(default)]
    pub settlement_policy: SettlementPolicy,

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
            max_expected_retries: default_rpc_max_expected_retries(),
            retry_on_transient_failure: TransientRetryPolicyImpl(ConfigTxRetryPolicy::default())
                .get()
                .unwrap(),
            retry_on_not_included_on_l1:
                NonInclusionRetryPolicyImpl(ConfigTxRetryPolicy::default())
                    .get()
                    .unwrap(),
            settlement_policy: SettlementPolicy::default(),
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
/// - **Settlement policy**: Choose the appropriate finality level based on
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
    /// This controls how pessimistic proofs (proofs of state transitions for
    /// certificates) are submitted to the L1 settlement layer.
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

const fn default_rpc_max_expected_retries() -> usize {
    16 * 1024
}

/// Default number of confirmations required
/// for the transaction to resolve a receipt.
const fn default_latest_block_confirmations() -> usize {
    0
}

fn default_gas_limit_ceiling() -> U256 {
    U256::from(60_000_000_u64)
}

/// Default gas price ceiling for the transaction.
const fn default_gas_price_ceiling() -> u128 {
    // 100 gwei
    100_000_000_000_u128
}
