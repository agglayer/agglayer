use agglayer_primitives::Digest;
use agglayer_tries::utils::empty_hash_at_height;
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
}

impl Default for NullifierTree {
    fn default() -> Self {
        Self::new()
    }
}

impl NullifierTree {
    pub fn new() -> Self {
        // We add 1 to the depth here because the empty hash at height 0 is
        // already set to Digest::ZERO.
        let root = empty_hash_at_height::<{ NULLIFIER_TREE_DEPTH + 1 }>();
        NullifierTree { root }
    }

    pub fn new_with_root(root: Digest) -> Self {
        NullifierTree { root }
    }
}

impl From<NullifierTree> for pessimistic_proof_core::nullifier_tree::NullifierTree {
    fn from(tree: NullifierTree) -> Self {
        Self { root: tree.root }
    }
}
