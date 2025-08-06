use agglayer_primitives::{keccak::keccak256_combine, Digest};
pub use pessimistic_proof_core::local_balance_tree::{LocalBalancePath, LOCAL_BALANCE_TREE_DEPTH};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

// TODO: This is basically the same as the nullifier tree, consider refactoring
/// A commitment to the set of per-network nullifier trees maintained by the
/// local network
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocalBalanceTree {
    /// The Merkle Root of the nullifier tree
    #[serde_as(as = "_")]
    pub root: Digest,
    /// `empty_hash_at_height[i]` is the root of an empty Merkle tree of depth
    /// `i`.
    #[serde_as(as = "[_; LOCAL_BALANCE_TREE_DEPTH]")]
    empty_hash_at_height: [Digest; LOCAL_BALANCE_TREE_DEPTH],
}

impl Default for LocalBalanceTree {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalBalanceTree {
    pub fn new() -> Self {
        let mut empty_hash_at_height = [Digest::default(); LOCAL_BALANCE_TREE_DEPTH];
        for height in 1..LOCAL_BALANCE_TREE_DEPTH {
            empty_hash_at_height[height] = keccak256_combine([
                &empty_hash_at_height[height - 1],
                &empty_hash_at_height[height - 1],
            ]);
        }
        let root = keccak256_combine([
            &empty_hash_at_height[LOCAL_BALANCE_TREE_DEPTH - 1],
            &empty_hash_at_height[LOCAL_BALANCE_TREE_DEPTH - 1],
        ]);
        LocalBalanceTree {
            root,
            empty_hash_at_height,
        }
    }

    pub fn new_with_root(root: Digest) -> Self {
        let mut res = Self::new();
        res.root = root;
        res
    }
}

impl From<LocalBalanceTree> for pessimistic_proof_core::local_balance_tree::LocalBalanceTree {
    fn from(tree: LocalBalanceTree) -> Self {
        Self { root: tree.root }
    }
}
