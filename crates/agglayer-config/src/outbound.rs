use std::time::Duration;

use serde::{Deserialize, Deserializer, Serialize};
use serde_with::serde_as;
use tracing::warn;

use crate::Multiplier;

/// Outbound configuration.
/// After introduction of the`agglayer-settlement-service`, OutboundConfig
/// will be deprecated and removed.
#[derive(Serialize, Default, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename = "outbound", rename_all = "kebab-case")]
pub struct OutboundConfig {
    pub rpc: OutboundRpcConfig,
}

/// Outbound RPC configuration that is used to configure the outbound RPC
/// clients and their RPC calls.
#[derive(Serialize, Default, Debug, PartialEq, Eq)]
#[serde(rename = "rpc", rename_all = "kebab-case")]
pub struct OutboundRpcConfig {
    /// Outbound configuration of the RPC settle function call.
    pub settle_tx: OutboundRpcSettleConfig,
    pub settle_cert: OutboundRpcSettleConfig,
}

impl<'de> Deserialize<'de> for OutboundRpcConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename = "rpc", rename_all = "kebab-case")]
        pub struct Intermediate {
            pub settle_tx: Option<OutboundRpcSettleConfig>,
            pub settle_cert: Option<OutboundRpcSettleConfig>,
            pub settle: Option<OutboundRpcSettleConfig>,
        }

        let deserialized = Intermediate::deserialize(deserializer)?;

        let (settle_tx, settle_cert) = match (
            deserialized.settle_tx,
            deserialized.settle_cert,
            deserialized.settle,
        ) {
            (Some(tx), Some(cert), _) => (tx, cert),
            (None, None, Some(settle)) => {
                warn!("'settle' is deprecated. Please use 'settle-tx' and 'settle-cert' instead.");
                (settle.clone(), settle)
            }
            _ => {
                return Err(serde::de::Error::custom(
                    "Either both ['settle-tx','settle-cert'] or 'settle' must be specified",
                ));
            }
        };

        Ok(OutboundRpcConfig {
            settle_tx,
            settle_cert,
        })
    }
}

/// Outbound RPC settle configuration that is used to configure the outbound
/// RPC settle function call.
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename = "settle", rename_all = "kebab-case")]
pub struct OutboundRpcSettleConfig {
    /// Maximum number of retries for the transaction.
    #[serde(default = "default_rpc_retries")]
    pub max_retries: usize,

    /// Interval for the polling of the transaction.
    #[serde(default = "default_rpc_retry_interval")]
    #[serde(with = "crate::with::HumanDuration")]
    pub retry_interval: Duration,

    /// Number of confirmations required for the transaction to resolve a
    /// receipt.
    #[serde(default = "default_rpc_confirmations")]
    pub confirmations: usize,

    /// Gas multiplier factor for the transaction.
    /// The gas is calculated as follows:
    /// `gas = estimate_gas * (gas_multiplier / 100)
    #[serde(
        default = "default_gas_multiplier_factor",
        skip_serializing_if = "same_as_default_gas_multiplier_factor"
    )]
    pub gas_multiplier_factor: u32,

    /// Gas price configuration.
    #[serde(default)]
    pub gas_price: GasPriceConfig,
}

impl Default for OutboundRpcSettleConfig {
    fn default() -> Self {
        OutboundRpcSettleConfig {
            max_retries: default_rpc_retries(),
            retry_interval: default_rpc_retry_interval(),
            confirmations: default_rpc_confirmations(),
            gas_multiplier_factor: default_gas_multiplier_factor(),
            gas_price: GasPriceConfig::default(),
        }
    }
}

/// Gas price configuration for settlement transactions.
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct GasPriceConfig {
    /// Gas price multiplier for the transaction.
    /// The gas price is calculated as follows:
    /// `gas_price = estimate_gas_price * multiplier`
    #[serde(default, skip_serializing_if = "crate::is_default")]
    pub multiplier: Multiplier,

    /// Minimum gas price floor (in wei) for the transaction.
    /// Can be specified with units: "1gwei", "0.1eth", "1000000000wei"
    #[serde(default, skip_serializing_if = "crate::is_default")]
    #[serde_as(as = "crate::with::EthAmount")]
    pub floor: u128,

    /// Maximum gas price ceiling (in wei) for the transaction.
    /// Can be specified with units: "100gwei", "0.01eth", "10000000000wei"
    #[serde(default = "default_gas_price_ceiling")]
    #[serde_as(as = "crate::with::EthAmount")]
    pub ceiling: u128,
}

impl Default for GasPriceConfig {
    fn default() -> Self {
        GasPriceConfig {
            multiplier: Multiplier::default(),
            floor: 0,
            ceiling: default_gas_price_ceiling(),
        }
    }
}

/// Default gas price multiplier for the transaction.
const fn default_gas_multiplier_factor() -> u32 {
    100
}

const fn same_as_default_gas_multiplier_factor(v: &u32) -> bool {
    *v == default_gas_multiplier_factor()
}

/// Default number of retries for the transaction.
const fn default_rpc_retries() -> usize {
    30
}

/// Default interval for the polling of the transaction.
const fn default_rpc_retry_interval() -> Duration {
    Duration::from_secs(10)
}

/// Default number of confirmations required for the transaction to resolve a
/// receipt.
const fn default_rpc_confirmations() -> usize {
    1
}

/// Default gas price ceiling for the transaction.
const fn default_gas_price_ceiling() -> u128 {
    // 100 gwei
    100_000_000_000_u128
}

#[cfg(test)]
mod tests {
    mod outbound {
        use serde::Deserialize;

        use crate::outbound::OutboundConfig;

        #[test]
        #[allow(deprecated)]
        fn expected_namespace() {
            #[derive(Debug, Deserialize)]
            struct DummyContainer {
                outbound: OutboundConfig,
            }

            let toml = r#"
                [outbound.rpc.settle-tx]
                max-retries = 10
                [outbound.rpc.settle-cert]
                max-retries = 11
                [outbound.rpc.settle]
                max-retries = 12
                "#;

            let config = toml::from_str::<DummyContainer>(toml).unwrap();

            assert_eq!(config.outbound.rpc.settle_tx.max_retries, 10);
            assert_eq!(config.outbound.rpc.settle_cert.max_retries, 11);
        }

        mod rpc {
            mod settle {
                use std::time::Duration;

                use crate::outbound::OutboundRpcSettleConfig;

                #[test]
                fn test_default() {
                    let toml = r#"
                        "#;

                    let config = toml::from_str::<OutboundRpcSettleConfig>(toml).unwrap();

                    assert_eq!(config.max_retries, 30);
                    assert_eq!(config.retry_interval, Duration::from_secs(10));
                    assert_eq!(config.confirmations, 1);
                }

                #[test]
                fn test_custom() {
                    let toml = r#"
                        max-retries = 10
                        retry-interval = 1
                        confirmations = 5
                        "#;

                    let config = toml::from_str::<OutboundRpcSettleConfig>(toml).unwrap();

                    assert_eq!(config.max_retries, 10);
                    assert_eq!(config.retry_interval, Duration::from_secs(1));
                    assert_eq!(config.confirmations, 5);
                }
            }
        }
    }
}
