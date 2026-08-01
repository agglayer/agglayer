use std::{fmt, num::NonZeroU64, str::FromStr, time::Duration};

use agglayer_primitives::Address;
use serde::{Deserialize, Serialize};
use url::Url;

/// A configured L1 RPC endpoint.
///
/// Formatting never exposes the endpoint because any URL component may contain
/// credentials. Use [`Self::expose_url`] only when constructing a transport.
// Serde stays transparent so configuration files preserve the endpoint;
// Display and Debug are the redaction boundary for formatted values.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(transparent)]
pub struct L1RpcUrl(Url);

impl L1RpcUrl {
    /// Exposes the endpoint for transport construction.
    pub fn expose_url(&self) -> &Url {
        &self.0
    }
}

impl fmt::Display for L1RpcUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted>")
    }
}

impl fmt::Debug for L1RpcUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl FromStr for L1RpcUrl {
    type Err = url::ParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        value.parse().map(Self)
    }
}

impl From<Url> for L1RpcUrl {
    fn from(value: Url) -> Self {
        Self(value)
    }
}

/// The L1 configuration.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct L1 {
    pub chain_id: u64,
    pub node_url: L1RpcUrl,
    #[serde(default = "default_ws_node_url")]
    pub ws_node_url: L1RpcUrl,
    #[serde(
        default = "default_connect_attempt_timeout",
        with = "crate::with::HumanDuration"
    )]
    pub connect_attempt_timeout: Duration,

    #[serde(alias = "RollupManagerContract")]
    pub rollup_manager_contract: Address,

    #[serde(alias = "PolygonZkEVMGlobalExitRootV2Contract")]
    pub polygon_zkevm_global_exit_root_v2_contract: Address,

    #[serde(default = "L1::default_rpc_timeout")]
    #[serde(with = "crate::with::HumanDuration")]
    pub rpc_timeout: Duration,

    #[serde(default = "L1::default_event_filter_block_range")]
    pub event_filter_block_range: NonZeroU64,
}

impl L1 {
    const fn default_rpc_timeout() -> Duration {
        Duration::from_secs(45)
    }

    const fn default_event_filter_block_range() -> NonZeroU64 {
        NonZeroU64::new(10000).unwrap()
    }
}

impl Default for L1 {
    fn default() -> Self {
        // Values are coming from https://github.com/0xPolygon/agglayer/blob/main/config/default.go#L11
        Self {
            chain_id: 1337,
            node_url: "http://zkevm-mock-l1-network:8545".parse().unwrap(),
            connect_attempt_timeout: default_connect_attempt_timeout(),
            ws_node_url: default_ws_node_url(),
            rollup_manager_contract: "0xB7f8BC63BbcaD18155201308C8f3540b07f84F5e"
                .parse()
                .unwrap(),
            polygon_zkevm_global_exit_root_v2_contract:
                "0xB7f8BC63BbcaD18155201308C8f3540b07f84F5e"
                    .parse()
                    .unwrap(),
            rpc_timeout: Self::default_rpc_timeout(),
            event_filter_block_range: Self::default_event_filter_block_range(),
        }
    }
}

fn default_ws_node_url() -> L1RpcUrl {
    "ws://zkevm-mock-l1-network:8546".parse().unwrap()
}

const fn default_connect_attempt_timeout() -> Duration {
    Duration::from_secs(3)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOKENS: [&str; 6] = [
        "user-secret-803",
        "password-secret-803",
        "host-secret-803",
        "path-secret-803",
        "query-secret-803",
        "fragment-secret-803",
    ];

    fn secret_url(scheme: &str) -> L1RpcUrl {
        Url::parse(&format!(
            "{scheme}://{}:{}@{}.example/{}/?key={}#{}",
            TOKENS[0], TOKENS[1], TOKENS[2], TOKENS[3], TOKENS[4], TOKENS[5]
        ))
        .unwrap()
        .into()
    }

    #[test]
    fn l1_rpc_url_formatting_is_redacted() {
        let node_url = secret_url("https");
        assert_eq!(node_url.to_string(), "<redacted>");
        assert_eq!(format!("{node_url:?}"), "<redacted>");

        let mut l1 = L1::default();
        l1.node_url = node_url;
        l1.ws_node_url = secret_url("wss");

        let l1_debug = format!("{l1:?}");
        let mut config = crate::Config::default();
        config.l1 = l1;
        let config_debug = format!("{config:?}");

        for token in TOKENS {
            assert!(!l1_debug.contains(token));
            assert!(!config_debug.contains(token));
        }
    }

    #[test]
    fn l1_rpc_url_serde_is_transparent() {
        let mut l1 = L1::default();
        l1.node_url = secret_url("https");
        l1.ws_node_url = secret_url("wss");

        let serialized = toml::to_string(&l1).unwrap();
        assert!(serialized.contains(l1.node_url.expose_url().as_str()));
        assert!(serialized.contains(l1.ws_node_url.expose_url().as_str()));

        let deserialized: L1 = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized, l1);
    }
}
