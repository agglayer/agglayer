use pessimistic_proof_core::local_exit_tree::hasher::{Hasher, Keccak256Hasher};
pub use pessimistic_proof_core::nullifier_tree::NullifierKey;
pub use pessimistic_proof_core::nullifier_tree::NullifierPath;
pub use pessimistic_proof_core::nullifier_tree::NULLIFIER_TREE_DEPTH;
use pessimistic_proof_core::utils::FromBool;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

// TODO: This is basically the same as the local balance tree, consider
// refactoring TODO: Consider using an Indexed Merkle Tree instead of an SMT. See https://docs.aztec.network/aztec/concepts/storage/trees/indexed_merkle_tree.
/// A commitment to the set of per-network nullifier trees maintained by the
/// local network
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NullifierTree<H>
where
    H: Hasher,
    H::Digest: Serialize + for<'a> Deserialize<'a>,
{
    /// The Merkle Root of the nullifier tree
    #[serde_as(as = "_")]
    pub root: H::Digest,
    /// `empty_hash_at_height[i]` is the root of an empty Merkle tree of depth
    /// `i`.
    #[serde_as(as = "[_; NULLIFIER_TREE_DEPTH]")]
    empty_hash_at_height: [H::Digest; NULLIFIER_TREE_DEPTH],
}

impl<H> Default for NullifierTree<H>
where
    H: Hasher,
    H::Digest: Copy + Eq + Default + Serialize + for<'a> Deserialize<'a> + FromBool,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<H> NullifierTree<H>
where
    H: Hasher,
    H::Digest: Copy + Eq + Default + Serialize + for<'a> Deserialize<'a> + FromBool,
{
    pub fn new() -> Self {
        let mut empty_hash_at_height = [H::Digest::default(); NULLIFIER_TREE_DEPTH];
        for height in 1..NULLIFIER_TREE_DEPTH {
            empty_hash_at_height[height] = H::merge(
                &empty_hash_at_height[height - 1],
                &empty_hash_at_height[height - 1],
            );
        }
        let root = H::merge(
            &empty_hash_at_height[NULLIFIER_TREE_DEPTH - 1],
            &empty_hash_at_height[NULLIFIER_TREE_DEPTH - 1],
        );
        NullifierTree {
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

impl From<NullifierTree<Keccak256Hasher>>
    for pessimistic_proof_core::nullifier_tree::NullifierTree<Keccak256Hasher>
{
    fn from(tree: NullifierTree<Keccak256Hasher>) -> Self {
        Self {
            root: tree.root,
            empty_hash_at_height: tree.empty_hash_at_height,
        }
    }
}
