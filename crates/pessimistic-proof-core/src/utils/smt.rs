#![allow(clippy::needless_range_loop)]
use std::hash::Hash;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_with::serde_as;

use crate::{bridge_exit::TokenInfo, local_exit_tree::hasher::Hasher};

pub trait ToBits<const NUM_BITS: usize> {
    fn to_bits(&self) -> [bool; NUM_BITS];
}

impl ToBits<176> for TokenInfo {
    fn to_bits(&self) -> [bool; 176] {
        let mut bits = [false; 176];
        for i in 0..16 {
            bits[i] = (self.origin_network >> i) & 1 == 1;
        }
        for i in 16..176 {
            let byte_index = (i - 16) / 8;
            let bit_index = i % 8;
            bits[i] = (self.origin_token_address.0[byte_index] >> bit_index) & 1 == 1;
        }
        bits
    }
}

impl ToBits<8> for u8 {
    #[inline]
    fn to_bits(&self) -> [bool; 8] {
        std::array::from_fn(|i| (self >> i) & 1 == 1)
    }
}

impl ToBits<32> for u32 {
    fn to_bits(&self) -> [bool; 32] {
        std::array::from_fn(|i| (self >> i) & 1 == 1)
    }
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SmtMerkleProof<H, const DEPTH: usize>
where
    H: Hasher,
    H::Digest: Copy + Eq + Hash + Serialize + DeserializeOwned,
{
    #[serde_as(as = "[_; DEPTH]")]
    pub siblings: [H::Digest; DEPTH],
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SmtNonInclusionProof<H, const DEPTH: usize>
where
    H: Hasher,
    H::Digest: Copy + Eq + Serialize + DeserializeOwned,
{
    #[serde_as(as = "Vec<_>")]
    pub siblings: Vec<H::Digest>,
}

impl<H, const DEPTH: usize> SmtMerkleProof<H, DEPTH>
where
    H: Hasher,
    H::Digest: Copy + Eq + Hash + Serialize + DeserializeOwned,
{
    pub fn verify<K>(&self, key: K, value: H::Digest, root: H::Digest) -> bool
    where
        K: ToBits<DEPTH>,
    {
        let bits = key.to_bits();
        let mut hash = value;
        for i in 0..DEPTH {
            hash = if bits[DEPTH - i - 1] {
                H::merge(&self.siblings[i], &hash)
            } else {
                H::merge(&hash, &self.siblings[i])
            };
        }

        hash == root
    }

    pub fn verify_with_bits(&self, bits: [bool; DEPTH], value: H::Digest, root: H::Digest) -> bool {
        let mut hash = value;
        for i in 0..DEPTH {
            hash = if bits[DEPTH - i - 1] {
                H::merge(&self.siblings[i], &hash)
            } else {
                H::merge(&hash, &self.siblings[i])
            };
        }

        hash == root
    }

    /// Verify the inclusion proof (i.e. that `(key, old_value)` is in the SMT)
    /// and return the updated root of the SMT with `(key, new_value)`
    /// inserted, or `None` if the inclusion proof is invalid.
    pub fn verify_and_update<K>(
        &self,
        key: K,
        old_value: H::Digest,
        new_value: H::Digest,
        root: H::Digest,
    ) -> Option<H::Digest>
    where
        K: ToBits<DEPTH> + Copy,
    {
        let bits = key.to_bits();
        if !self.verify_with_bits(bits, old_value, root) {
            return None;
        }
        let mut hash = new_value;
        for i in 0..DEPTH {
            hash = if bits[DEPTH - i - 1] {
                H::merge(&self.siblings[i], &hash)
            } else {
                H::merge(&hash, &self.siblings[i])
            };
        }

        Some(hash)
    }
}

impl<H, const DEPTH: usize> SmtNonInclusionProof<H, DEPTH>
where
    H: Hasher,
    H::Digest: Copy + Eq + Serialize + DeserializeOwned,
{
    pub fn verify<K>(
        &self,
        key: K,
        root: H::Digest,
        empty_hash_at_height: &[H::Digest; DEPTH],
    ) -> bool
    where
        K: ToBits<DEPTH>,
    {
        if self.siblings.len() > DEPTH {
            return false;
        }
        if self.siblings.is_empty() {
            let empty_root = H::merge(
                &empty_hash_at_height[DEPTH - 1],
                &empty_hash_at_height[DEPTH - 1],
            );
            return root == empty_root;
        }
        let bits = key.to_bits();
        let mut entry = empty_hash_at_height[DEPTH - self.siblings.len()];
        for i in (0..self.siblings.len()).rev() {
            let sibling = self.siblings[i];
            entry = if bits[i] {
                H::merge(&sibling, &entry)
            } else {
                H::merge(&entry, &sibling)
            };
        }

        entry == root
    }

    pub fn verify_with_bits(
        &self,
        bits: [bool; DEPTH],
        root: H::Digest,
        empty_hash_at_height: &[H::Digest; DEPTH],
    ) -> bool {
        if self.siblings.len() > DEPTH {
            return false;
        }
        if self.siblings.is_empty() {
            let empty_root = H::merge(
                &empty_hash_at_height[DEPTH - 1],
                &empty_hash_at_height[DEPTH - 1],
            );
            return root == empty_root;
        }
        let mut entry = empty_hash_at_height[DEPTH - self.siblings.len()];
        for i in (0..self.siblings.len()).rev() {
            let sibling = self.siblings[i];
            entry = if bits[i] {
                H::merge(&sibling, &entry)
            } else {
                H::merge(&entry, &sibling)
            };
        }

        entry == root
    }

    /// Verify the non-inclusion proof (i.e. that `key` is not in the SMT) and
    /// return the updated root of the SMT with `(key, value)` inserted, or
    /// `None` if the inclusion proof is invalid.
    pub fn verify_and_update<K>(
        &self,
        key: K,
        new_value: H::Digest,
        root: H::Digest,
        empty_hash_at_height: &[H::Digest; DEPTH],
    ) -> Option<H::Digest>
    where
        K: Copy + ToBits<DEPTH>,
    {
        let bits = key.to_bits();
        if !self.verify_with_bits(bits, root, empty_hash_at_height) {
            return None;
        }

        let mut entry = new_value;
        for i in (self.siblings.len()..DEPTH).rev() {
            let sibling = empty_hash_at_height[DEPTH - i - 1];
            entry = if bits[i] {
                H::merge(&sibling, &entry)
            } else {
                H::merge(&entry, &sibling)
            };
        }
        for i in (0..self.siblings.len()).rev() {
            let sibling = self.siblings[i];
            entry = if bits[i] {
                H::merge(&sibling, &entry)
            } else {
                H::merge(&entry, &sibling)
            };
        }

        Some(entry)
    }
}
