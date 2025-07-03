use std::{net::Ipv4Addr, str::FromStr};

use agglayer_types::NetworkId;
use serde::{Deserialize, Deserializer, Serialize};
use serde_with::serde_as;

/// The local RPC server configuration.
#[serde_as]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct PrivateNetworksConfig {
    /// List of the private networks.
    pub networks: Vec<NetworkId>,

    /// The port for the local gRPC server for private networks.
    /// Overridden by `AGGLAYER_PRIVATE_GRPC_PORT` environment variable.
    #[serde(deserialize_with = "deserialize_grpc_port")]
    pub grpc_port: u16,

    /// The host for the local RPC server for private networks.
    pub host: Ipv4Addr,
}

impl PrivateNetworksConfig {
    /// Create a new `PrivateNetworksConfig` with default values.
    pub fn for_tests(networks: Vec<NetworkId>) -> Self {
        Self {
            networks,
            // Invalid host/port, to be overridden by the test context
            grpc_port: u16::MAX,
            host: Ipv4Addr::new(255, 255, 255, 255),
        }
    }
}

fn deserialize_grpc_port<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: Deserializer<'de>,
{
    let port = u16::deserialize(deserializer)?;

    Ok(from_env_or_default("AGGLAYER_PRIVATE_GRPC_PORT", port))
}

/// Get an environment variable or a default value if it is not set.
fn from_env_or_default<T: FromStr>(key: &str, default: T) -> T {
    std::env::var(key)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}
