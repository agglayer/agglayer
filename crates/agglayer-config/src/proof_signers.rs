use std::{collections::HashMap, str::FromStr as _};

use ethers::types::Address;
use serde::{
    de::{MapAccess, Visitor},
    Deserializer,
};

/// Deserialize a map of Signers from a TOML file, where the keys are integers
/// and the values are Address.
pub(crate) fn deserialize_signers_map<'de, D>(
    deserializer: D,
) -> Result<HashMap<u32, Address>, D::Error>
where
    D: Deserializer<'de>,
{
    struct SignerMapVisitor;

    impl<'de> Visitor<'de> for SignerMapVisitor {
        type Value = HashMap<u32, Address>;

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
                let value = Address::from_str(&value).map_err(serde::de::Error::custom)?;
                map.insert(key, value);
            }
            Ok(map)
        }
    }

    deserializer.deserialize_map(SignerMapVisitor)
}
