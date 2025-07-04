use std::net::Ipv4Addr;

use agglayer_types::NetworkId;
use serde::{Deserialize, Deserializer, Serialize};
use serde_with::serde_as;

use crate::from_env_or_default;

/// The local RPC server configuration.
#[serde_as]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct ProxiedNetworksConfig {
    /// List of the private networks.
    pub networks: Vec<NetworkId>,

    /// The port for the local gRPC server for private networks.
    /// Overridden by `AGGLAYER_PRIVATE_GRPC_PORT` environment variable.
    #[serde(deserialize_with = "deserialize_grpc_port")]
    pub grpc_port: u16,

    /// The host for the local RPC server for private networks.
    pub host: Ipv4Addr,
}

impl ProxiedNetworksConfig {
    /// Create a new `ProxiedNetworksConfig` with default values.
    #[cfg(feature = "testutils")]
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
