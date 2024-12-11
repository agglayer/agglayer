use std::{fmt, ops::Deref};

use hex::FromHex;
use reth_primitives::U256;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{local_balance_tree::FromU256, nullifier_tree::FromBool};

#[derive(Default, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Digest(pub [u8; 32]);

impl Deref for Digest {
    type Target = [u8; 32];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<[u8]> for Digest {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Display for Digest {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:#x}", self)
    }
}

impl fmt::Debug for Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}", self)
    }
}

impl fmt::UpperHex for Digest {
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

impl Digest {
    pub const ZERO: Digest = Digest([0u8; 32]);

    pub fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl From<[u8; 32]> for Digest {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl fmt::LowerHex for Digest {
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

impl<'de> Deserialize<'de> for Digest {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = <String>::deserialize(deserializer)?;

            let s = s.trim_start_matches("0x");
            let s = <[u8; 32]>::from_hex(s).map_err(serde::de::Error::custom)?;

            Ok(Digest(s))
        } else {
            #[derive(::serde::Deserialize)]
            #[serde(rename = "NewDigest")]
            struct Value([u8; 32]);

            let value = Value::deserialize(deserializer)?;
            Ok(Digest(value.0))
        }
    }
}

impl Serialize for Digest {
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

const DIGEST_FROM_BOOL_TRUE: Digest = Digest([
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
]);
const DIGEST_FROM_BOOL_FALSE: Digest = Digest([
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
]);

impl FromBool for Digest {
    fn from_bool(b: bool) -> Self {
        if b {
            DIGEST_FROM_BOOL_TRUE
        } else {
            DIGEST_FROM_BOOL_FALSE
        }
    }
}

impl FromU256 for Digest {
    fn from_u256(u: U256) -> Self {
        Self(u.to_be_bytes())
    }
}

impl TryFrom<Vec<u8>> for Digest {
    type Error = hex::FromHexError;
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let mut bytes = [0u8; 32];
        let len = value.len();
        bytes[..len].copy_from_slice(&value);

        Ok(Digest(bytes))
    }
}

impl From<Digest> for Vec<u8> {
    fn from(value: Digest) -> Self {
        value.0.to_vec()
    }
}

#[cfg(test)]
impl rand::distributions::Distribution<Digest> for rand::distributions::Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Digest {
        let raw: [u8; 32] = rng.gen();

        Digest(raw)
    }
}
