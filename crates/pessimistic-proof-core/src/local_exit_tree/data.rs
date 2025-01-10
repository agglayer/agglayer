#![allow(clippy::needless_range_loop)]
use std::fmt::Debug;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_with::serde_as;

use crate::local_exit_tree::hasher::Hasher;

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LETMerkleProof<H, const TREE_DEPTH: usize = 32>
where
    H: Hasher,
    H::Digest: Serialize + DeserializeOwned,
{
    #[serde_as(as = "[_; TREE_DEPTH]")]
    pub siblings: [H::Digest; TREE_DEPTH],
}

impl<H, const TREE_DEPTH: usize> LETMerkleProof<H, TREE_DEPTH>
where
    H: Hasher,
    H::Digest: Eq + Copy + Default + Serialize + DeserializeOwned,
{
    pub fn verify(&self, leaf: H::Digest, leaf_index: u32, root: H::Digest) -> bool {
        let mut entry = leaf;
        let mut index = leaf_index;
        for &sibling in &self.siblings {
            entry = if index & 1 == 0 {
                H::merge(&entry, &sibling)
            } else {
                H::merge(&sibling, &entry)
            };
            index >>= 1;
        }
        if index != 0 {
            return false;
        }

        entry == root
    }
}
