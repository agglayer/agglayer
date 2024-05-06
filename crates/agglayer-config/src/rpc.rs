use std::{collections::HashMap, net::Ipv4Addr, str::FromStr, time::Duration};

use jsonrpsee::core::TEN_MB_SIZE_BYTES;
use serde::{
    de::{MapAccess, Visitor},
    Deserialize, Deserializer,
};
use url::Url;

/// The default port for the local RPC server.
const DEFAULT_PORT: u16 = 9090;

/// The local RPC server configuration.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct RpcConfig {
    /// If the `PORT` environment variable is set, it will take precedence over
    /// the configuration file.
    #[serde(default = "default_port", deserialize_with = "deserialize_port")]
    pub port: u16,
    #[serde(default = "default_host")]
    pub host: Ipv4Addr,

    // Skip serialization of these fields as we don't need to expose them in the
    // configuration yet.
    /// The maximum size of the request body in bytes.
    #[serde(skip, default = "default_body_size")]
    pub max_request_body_size: u32,
    /// The maximum size of the response body in bytes.
    #[serde(skip, default = "default_body_size")]
    pub max_response_body_size: u32,
    /// The maximum number of connections.
    #[serde(skip, default = "default_max_connections")]
    pub max_connections: u32,
    /// The maximum number of requests in a batch request. If `None`, the
    /// batch request limit is unlimited.
    #[serde(skip)]
    pub batch_request_limit: Option<u32>,
    /// The interval at which to send ping messages
    #[serde(skip)]
    pub ping_interval: Option<Duration>,
}

impl Default for RpcConfig {
    fn default() -> Self {
        Self {
            port: default_port(),
            host: default_host(),
            max_request_body_size: default_body_size(),
            max_response_body_size: default_body_size(),
            max_connections: default_max_connections(),
            batch_request_limit: None,
            ping_interval: None,
        }
    }
}

/// The default maximum number of connections.
fn default_max_connections() -> u32 {
    100
}

/// The default size of the request and response bodies in bytes.
fn default_body_size() -> u32 {
    TEN_MB_SIZE_BYTES
}

/// The default port for the local RPC server.
/// If the `PORT` environment variable is set, it will take precedence over
fn default_port() -> u16 {
    from_env_or_default("PORT", DEFAULT_PORT)
}

/// The default host for the local RPC server.
const fn default_host() -> Ipv4Addr {
    Ipv4Addr::new(0, 0, 0, 0)
}

fn deserialize_port<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: Deserializer<'de>,
{
    let port = u16::deserialize(deserializer)?;

    Ok(from_env_or_default("PORT", port))
}

/// Deserialize a map of RPCs from a TOML file, where the keys are integers and
/// the values are URLs.
pub(crate) fn deserialize_rpc_map<'de, D>(deserializer: D) -> Result<HashMap<u32, Url>, D::Error>
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

/// Get an environment variable or a default value if it is not set.
fn from_env_or_default<T: FromStr>(key: &str, default: T) -> T {
    std::env::var(key)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}
