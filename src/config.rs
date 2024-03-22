//! Agglayer configuration.
//!
//! The agglayer is configured via its TOML configuration file, `agglayer.toml`
//! by default, which is deserialized into the [`Config`] struct.

use std::{collections::HashMap, net::Ipv4Addr};

use ethers::types::Address;
use serde::{
    de::{MapAccess, Visitor},
    Deserialize, Deserializer,
};
use url::Url;

/// The Agglayer configuration.
#[derive(Deserialize, Debug)]
pub(crate) struct Config {
    /// A map of Zkevm node RPC endpoints for each rollup.
    ///
    /// The key is the rollup ID, and the value is the URL of the associated RPC
    /// endpoint.
    #[serde(rename = "FullNodeRPCs", deserialize_with = "deserialize_rpc_map")]
    #[allow(dead_code)]
    pub(crate) full_node_rpcs: HashMap<u32, Url>,
    /// The local gRPC server configuration.
    #[serde(rename = "GRPC")]
    pub(crate) grpc: GrpcConfig,
    /// The L1 configuration.
    #[serde(rename = "L1")]
    pub(crate) l1: L1,
}

/// The local gRPC server configuration.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct GrpcConfig {
    pub(crate) port: u16,
    pub(crate) host: Ipv4Addr,
}

/// The L1 configuration.
#[derive(Deserialize, Debug)]
pub(crate) struct L1 {
    #[serde(rename = "ChainID")]
    #[allow(dead_code)]
    pub(crate) chain_id: u64,
    #[serde(rename = "NodeURL")]
    pub(crate) node_url: Url,
    #[serde(rename = "RollupManagerContract")]
    pub(crate) rollup_manager_contract: Address,
}

/// Deserialize a map of RPCs from a TOML file, where the keys are integers and
/// the values are URLs.
fn deserialize_rpc_map<'de, D>(deserializer: D) -> Result<HashMap<u32, Url>, D::Error>
where
    D: Deserializer<'de>,
{
    struct RpcMapVisitor;

    impl<'de> Visitor<'de> for RpcMapVisitor {
        type Value = HashMap<u32, Url>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a map of string keys to string values")
        }

        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut map = HashMap::with_capacity(access.size_hint().unwrap_or(0));
            while let Some((key, value)) = access.next_entry::<String, String>()? {
                let key: u32 = key.parse().map_err(serde::de::Error::custom)?;
                let value = Url::parse(&value).map_err(serde::de::Error::custom)?;
                map.insert(key, value);
            }
            Ok(map)
        }
    }

    deserializer.deserialize_map(RpcMapVisitor)
}
