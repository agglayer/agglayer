use agglayer_primitives::Digest;
use agglayer_tries::utils::empty_hash_at_height;
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
}

impl Default for LocalBalanceTree {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalBalanceTree {
    pub fn new() -> Self {
        // We add 1 to the depth here because the empty hash at height 0 is
        // already set to Digest::ZERO.
        let root = empty_hash_at_height::<{ LOCAL_BALANCE_TREE_DEPTH + 1 }>();
        LocalBalanceTree { root }
    }

    pub fn new_with_root(root: Digest) -> Self {
        LocalBalanceTree { root }
    }
}

impl From<LocalBalanceTree> for pessimistic_proof_core::local_balance_tree::LocalBalanceTree {
    fn from(tree: LocalBalanceTree) -> Self {
        Self { root: tree.root }
    }
}
