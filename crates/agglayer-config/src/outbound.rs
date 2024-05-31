use std::time::Duration;

use serde::Deserialize;
use serde_with::serde_as;
use serde_with::DurationSeconds;

/// Outbound configuration.
#[derive(Default, Debug, Deserialize)]
#[serde(rename = "outbound")]
pub struct OutboundConfig {
    pub rpc: OutboundRpcConfig,
}

/// Outbound RPC configuration that is used to configure the outbound RPC
/// clients and their RPC calls.
#[derive(Default, Debug, Deserialize)]
#[serde(rename = "rpc")]
pub struct OutboundRpcConfig {
    /// Outbound configuration of the RPC settle function call.
    pub settle: OutboundRpcSettleConfig,
}

/// Outbound RPC settle configuration that is used to configure the outbound
/// RPC settle function call.
#[serde_as]
#[derive(Debug, Deserialize)]
#[serde(rename = "settle")]
pub struct OutboundRpcSettleConfig {
    /// Maximum number of retries for the transaction.
    #[serde(default = "default_rpc_retries")]
    pub max_retries: usize,

    /// Interval for the polling of the transaction.
    #[serde(default = "default_rpc_retry_interval")]
    #[serde_as(as = "DurationSeconds")]
    pub retry_interval: Duration,

    /// Number of confirmations required for the transaction to resolve a
    /// receipt.
    #[serde(default = "default_rpc_confirmations")]
    pub confirmations: usize,
}

impl Default for OutboundRpcSettleConfig {
    fn default() -> Self {
        OutboundRpcSettleConfig {
            max_retries: default_rpc_retries(),
            retry_interval: default_rpc_retry_interval(),
            confirmations: default_rpc_confirmations(),
        }
    }
}

/// Default number of retries for the transaction. It matches the ethers default
/// value.
const fn default_rpc_retries() -> usize {
    3
}

/// Default interval for the polling of the transaction.
const fn default_rpc_retry_interval() -> Duration {
    Duration::from_secs(7)
}

/// Default number of confirmations required for the transaction to resolve a
/// receipt.
const fn default_rpc_confirmations() -> usize {
    1
}

#[cfg(test)]
mod tests {
    mod outbound {
        use serde::Deserialize;

        use crate::outbound::OutboundConfig;

        #[test]
        fn expected_namespace() {
            #[derive(Debug, Deserialize)]
            struct DummyContainer {
                outbound: OutboundConfig,
            }

            let toml = r#"
                [outbound.rpc.settle]
                max_retries = 10
                "#;

            let config = toml::from_str::<DummyContainer>(toml).unwrap();

            assert_eq!(config.outbound.rpc.settle.max_retries, 10);
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

                    assert_eq!(config.max_retries, 3);
                    assert_eq!(config.retry_interval, Duration::from_secs(7));
                    assert_eq!(config.confirmations, 1);
                }

                #[test]
                fn test_custom() {
                    let toml = r#"
                        max_retries = 10
                        retry_interval = 1
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
