use std::time::Duration;
use agglayer_primitives::U256;
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub enum TxRetryPolicy {
    #[default] Linear,
}

/// The settlement transaction configuration.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub struct SettlementTransactionConfig {

    /// Maximum number of retries for the transaction.
    /// Expected to be a big number for settlement service.
    #[serde(default = "default_rpc_retries")]
    pub max_retries: usize,

    /// Retry approach for the transaction.
    #[serde(default = "default_rpc_retry_policy")]
    pub tx_retry_policy: TxRetryPolicy,

    /// Retry interval.
    #[serde(default = "default_rpc_retry_interval")]
    #[serde(with = "crate::with::HumanDuration")]
    pub retry_interval: Duration,

    /// Number of confirmations required for
    /// the transaction to resolve a receipt.
    #[serde(default = "default_rpc_confirmations")]
    pub confirmations: usize,

    /// Gas multiplier factor for the transaction.
    /// The gas is calculated as follows:
    /// `gas = (estimate_gas * gas_multiplier) / 100
    #[serde(
        default = "default_gas_multiplier_factor",
        skip_serializing_if = "same_as_default_gas_multiplier_factor"
    )]
    pub gas_multiplier_factor: u32,


    /// Gas limit
    #[serde(
        default = "default_gas_limit",
    )]
    pub gas_limit: U256,

}

/// The settlement service configuration.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub struct SettlementServiceConfig {}


/// Default number of retries for the transaction.
const fn default_rpc_retries() -> usize {
    128
}

const fn default_rpc_retry_policy() -> TxRetryPolicy {
    TxRetryPolicy::Linear
}

/// Default number of confirmations required
/// for the transaction to resolve a receipt.
const fn default_rpc_confirmations() -> usize {
    32
}


/// Default gas price multiplier for the transaction.
const fn default_gas_multiplier_factor() -> u32 {
    100
}

const fn same_as_default_gas_multiplier_factor(v: &u32) -> bool {
    *v == default_gas_multiplier_factor()
}

const fn default_gas_limit() -> U256 {
    U256::from(60_000_000_u64),
}