use std::hash::Hash;

pub use pessimistic_proof_core::local_balance_tree::{LocalBalancePath, LOCAL_BALANCE_TREE_DEPTH};
use pessimistic_proof_core::{
    local_exit_tree::hasher::{Hasher, Keccak256Hasher},
    utils::FromU256,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_with::serde_as;

// TODO: This is basically the same as the nullifier tree, consider refactoring
/// A commitment to the set of per-network nullifier trees maintained by the
/// local network
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocalBalanceTree<H>
where
    H: Hasher,
    H::Digest: Serialize + DeserializeOwned,
{
    /// The Merkle Root of the nullifier tree
    #[serde_as(as = "_")]
    pub root: H::Digest,
    /// `empty_hash_at_height[i]` is the root of an empty Merkle tree of depth
    /// `i`.
    #[serde_as(as = "[_; LOCAL_BALANCE_TREE_DEPTH]")]
    empty_hash_at_height: [H::Digest; LOCAL_BALANCE_TREE_DEPTH],
}

impl<H> Default for LocalBalanceTree<H>
where
    H: Hasher,
    H::Digest: Copy + Eq + Hash + Default + Serialize + for<'a> Deserialize<'a> + FromU256,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<H> LocalBalanceTree<H>
where
    H: Hasher,
    H::Digest: Copy + Eq + Hash + Default + Serialize + for<'a> Deserialize<'a> + FromU256,
{
    pub fn new() -> Self {
        let mut empty_hash_at_height = [H::Digest::default(); LOCAL_BALANCE_TREE_DEPTH];
        for height in 1..LOCAL_BALANCE_TREE_DEPTH {
            empty_hash_at_height[height] = H::merge(
                &empty_hash_at_height[height - 1],
                &empty_hash_at_height[height - 1],
            );
        }
        let root = H::merge(
            &empty_hash_at_height[LOCAL_BALANCE_TREE_DEPTH - 1],
            &empty_hash_at_height[LOCAL_BALANCE_TREE_DEPTH - 1],
        );
        LocalBalanceTree {
            root,
            empty_hash_at_height,
        }
    }

    pub fn new_with_root(root: H::Digest) -> Self {
        let mut res = Self::new();
        res.root = root;
        res
    }
}

impl From<LocalBalanceTree<Keccak256Hasher>>
    for pessimistic_proof_core::local_balance_tree::LocalBalanceTree<Keccak256Hasher>
{
    fn from(tree: LocalBalanceTree<Keccak256Hasher>) -> Self {
        Self { root: tree.root }
    }
}
