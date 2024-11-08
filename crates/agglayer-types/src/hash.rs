use std::fmt;

use hex::FromHex;
use serde::{Deserialize, Serialize};
use serde::{Deserializer, Serializer};

#[derive(Default, std::hash::Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Hash(pub [u8; 32]);

impl Hash {
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:#x}", self)
    }
}

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}", self)
    }
}

impl fmt::UpperHex for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "0x")?;
        }

        for byte in &self.0 {
            write!(f, "{:02X}", byte)?;
        }

        Ok(())
    }
}

impl fmt::LowerHex for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "0x")?;
        }

        for byte in &self.0 {
            write!(f, "{:02x}", byte)?;
        }

        Ok(())
    }
}
impl<'de> Deserialize<'de> for Hash {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = <String>::deserialize(deserializer)?;

            let s = s.trim_start_matches("0x");
            let s = <[u8; 32]>::from_hex(s).map_err(serde::de::Error::custom)?;

            Ok(Hash(s))
        } else {
            #[derive(::serde::Deserialize)]
            #[serde(rename = "Hash")]
            struct Value([u8; 32]);

            let value = Value::deserialize(deserializer)?;
            Ok(Hash(value.0))
        }
    }
}
impl Serialize for Hash {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            format!("{:#x}", self).serialize(serializer)
        } else {
            serializer.serialize_newtype_struct("Hash", &self.0)
        }
    }
}

impl From<[u8; 32]> for Hash {
    fn from(bytes: [u8; 32]) -> Self {
        Hash(bytes)
    }
}
