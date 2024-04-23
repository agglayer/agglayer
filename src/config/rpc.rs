use std::{collections::HashMap, net::Ipv4Addr, str::FromStr};

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
pub(crate) struct RpcConfig {
    /// If the `PORT` environment variable is set, it will take precedence over
    /// the configuration file.
    #[serde(default = "default_port", deserialize_with = "deserialize_port")]
    pub(crate) port: u16,
    #[serde(default = "default_host")]
    pub(crate) host: Ipv4Addr,
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
