use std::{collections::HashMap, net::Ipv4Addr};

use serde::{
    de::{MapAccess, Visitor},
    Deserialize, Deserializer,
};
use url::Url;

/// The local gRPC server configuration.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct RpcConfig {
    pub(crate) port: u16,
    pub(crate) host: Ipv4Addr,
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
