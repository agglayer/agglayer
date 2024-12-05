use std::{fmt, ops::Deref};

use hex::FromHex;
use reth_primitives::U256;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tiny_keccak::{Hasher, Keccak};

use crate::{local_balance_tree::FromU256, nullifier_tree::FromBool};

#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct NewDigest(pub [u8; 32]);

impl Deref for NewDigest {
    type Target = [u8; 32];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<[u8]> for NewDigest {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Display for NewDigest {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:#x}", self)
    }
}

impl fmt::Debug for NewDigest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}", self)
    }
}

impl fmt::UpperHex for NewDigest {
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

impl NewDigest {
    pub const ZERO: NewDigest = NewDigest([0u8; 32]);

    pub fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl From<[u8; 32]> for NewDigest {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl FromBool for NewDigest {
    fn from_bool(b: bool) -> Self {
        let array = if b {
            [
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0,
            ]
        } else {
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0,
            ]
        };

        Self(array)
    }
}
impl fmt::LowerHex for NewDigest {
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

impl<'de> Deserialize<'de> for NewDigest {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = <String>::deserialize(deserializer)?;

            let s = s.trim_start_matches("0x");
            let s = <[u8; 32]>::from_hex(s).map_err(serde::de::Error::custom)?;

            Ok(NewDigest(s))
        } else {
            #[derive(::serde::Deserialize)]
            #[serde(rename = "NewDigest")]
            struct Value([u8; 32]);

            let value = Value::deserialize(deserializer)?;
            Ok(NewDigest(value.0))
        }
    }
}
impl Serialize for NewDigest {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            format!("{:#x}", self).serialize(serializer)
        } else {
            serializer.serialize_newtype_struct("NewDigest", &self.0)
        }
    }
}
