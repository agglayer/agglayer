use agglayer_primitives::{keccak::keccak256_combine, Digest};
pub use pessimistic_proof_core::nullifier_tree::{
    NullifierKey, NullifierPath, NULLIFIER_TREE_DEPTH,
};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

// TODO: This is basically the same as the local balance tree, consider
// refactoring TODO: Consider using an Indexed Merkle Tree instead of an SMT. See https://docs.aztec.network/aztec/concepts/storage/trees/indexed_merkle_tree.
/// A commitment to the set of per-network nullifier trees maintained by the
/// local network
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NullifierTree {
    /// The Merkle Root of the nullifier tree
    #[serde_as(as = "_")]
    pub root: Digest,
    /// `empty_hash_at_height[i]` is the root of an empty Merkle tree of depth
    /// `i`.
    #[serde_as(as = "[_; NULLIFIER_TREE_DEPTH]")]
    empty_hash_at_height: [Digest; NULLIFIER_TREE_DEPTH],
}

impl Default for NullifierTree {
    fn default() -> Self {
        Self::new()
    }
}

impl NullifierTree {
    pub fn new() -> Self {
        let mut empty_hash_at_height = [Digest::default(); NULLIFIER_TREE_DEPTH];
        for height in 1..NULLIFIER_TREE_DEPTH {
            empty_hash_at_height[height] = keccak256_combine([
                &empty_hash_at_height[height - 1],
                &empty_hash_at_height[height - 1],
            ]);
        }
        let root = keccak256_combine([
            &empty_hash_at_height[NULLIFIER_TREE_DEPTH - 1],
            &empty_hash_at_height[NULLIFIER_TREE_DEPTH - 1],
        ]);
        NullifierTree {
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

impl From<NullifierTree> for pessimistic_proof_core::nullifier_tree::NullifierTree {
    fn from(tree: NullifierTree) -> Self {
        Self {
            root: tree.root,
            empty_hash_at_height: tree.empty_hash_at_height,
        }
    }
}
