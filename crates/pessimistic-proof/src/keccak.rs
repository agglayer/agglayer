use std::fmt;

use alloy::primitives::U256;
use hex::FromHex;
#[cfg(test)]
use rand::distributions::{self, Standard};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tiny_keccak::{Hasher, Keccak};

use crate::{local_balance_tree::FromU256, nullifier_tree::FromBool};

#[derive(Hash, PartialEq, Default, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Digest(pub [u8; 32]);

#[cfg(test)]
impl distributions::Distribution<Digest> for Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Digest {
        let mut result = [0u8; 32];

        rng.fill(&mut result);

        result.into()
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
impl Digest {
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }
}

// Define an extension trait for arrays of Hashes
pub(crate) trait HashArrayConcat<const TREE_DEPTH: usize> {
    fn concat(&self) -> [u8; TREE_DEPTH];
}

// Implement the trait for `[Hash; 32]`
impl<const TREE_DEPTH: usize> HashArrayConcat<TREE_DEPTH> for [Digest; 32] {
    fn concat(&self) -> [u8; TREE_DEPTH] {
        let mut result = [0u8; TREE_DEPTH];

        // Concatenate each `Hash` into `result`
        for (i, hash) in self.iter().enumerate() {
            let start = i * 32;
            result[start..start + 32].copy_from_slice(&hash.0);
        }

        result
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
            #[serde(rename = "Hash")]
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
            serializer.serialize_newtype_struct("Hash", &self.0)
        }
    }
}

impl From<[u8; 32]> for Digest {
    fn from(bytes: [u8; 32]) -> Self {
        Digest(bytes)
    }
}

impl From<Digest> for [u8; 32] {
    fn from(hash: Digest) -> [u8; 32] {
        hash.0
    }
}
impl AsRef<[u8]> for Digest {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
impl FromBool for Digest {
    fn from_bool(b: bool) -> Self {
        if b {
            [
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0,
            ]
        } else {
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0,
            ]
        }
        .into()
    }
}
impl FromU256 for Digest {
    fn from_u256(u: U256) -> Self {
        u.to_be_bytes().into()
    }
}

/// Hashes the input data using a Keccak hasher with a 256-bit security level.
pub fn keccak256(data: &[u8]) -> Digest {
    let mut hasher = Keccak::v256();
    hasher.update(data);

    let mut output = [0u8; 32];
    hasher.finalize(&mut output);
    output.into()
}

/// Hashes the input items using a Keccak hasher with a 256-bit security level.
/// Safety: This function should only be called with fixed-size items to avoid
/// collisions.
pub fn keccak256_combine<I, T>(items: I) -> Digest
where
    I: IntoIterator<Item = T>,
    T: AsRef<[u8]>,
{
    let mut hasher = Keccak::v256();
    for data in items {
        hasher.update(data.as_ref());
    }

    let mut output = [0u8; 32];
    hasher.finalize(&mut output);
    output.into()
}
