use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::{
    bridge_exit::NetworkId,
    local_exit_tree::hasher::Hasher,
    utils::smt::{SmtNonInclusionProof, ToBits},
    ProofError,
};

// 32 bits for the network id and 32 bits for the LET index
// TODO: consider using less than 32 bits for the network id - unlikely that we'll have 4 billion chains :)
pub const NULLIFIER_TREE_DEPTH: usize = 64;

// TODO: This is basically the same as the local balance tree, consider refactoring
// TODO: Consider using an Indexed Merkle Tree instead of an SMT. See https://docs.aztec.network/aztec/concepts/storage/trees/indexed_merkle_tree.
/// A commitment to the set of per-network nullifier sets maintained by the local network
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NullifierTree<H>
where
    H: Hasher,
    H::Digest: Serialize + for<'a> Deserialize<'a>,
{
    /// The Merkle Root of the nullifier set
    #[serde_as(as = "_")]
    pub root: H::Digest,
    /// `empty_hash_at_height[i]` is the root of an empty Merkle tree of depth
    /// `i`.
    #[serde_as(as = "[_; NULLIFIER_TREE_DEPTH]")]
    empty_hash_at_height: [H::Digest; NULLIFIER_TREE_DEPTH],
}

pub type NullifierPath<H> = SmtNonInclusionProof<H, NULLIFIER_TREE_DEPTH>;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct NullifierKey {
    pub network_id: NetworkId,
    pub let_index: u32,
}

impl ToBits<64> for NullifierKey {
    fn to_bits(&self) -> [bool; 64] {
        std::array::from_fn(|i| {
            if i < 32 {
                (*self.network_id >> i) & 1 == 1
            } else {
                (self.let_index >> (i - 32)) & 1 == 1
            }
        })
    }
}

pub trait FromBool {
    fn from_bool(b: bool) -> Self;
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
            empty_hash_at_height[height] =
                H::merge(&empty_hash_at_height[height - 1], &empty_hash_at_height[height - 1]);
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

    // TODO: Consider batching the updates per network for efficiency
    pub fn verify_and_update(
        &mut self,
        key: NullifierKey,
        path_to_update: &NullifierPath<H>,
    ) -> Result<(), ProofError> {
        self.root = path_to_update
            .verify_and_update(
                key,
                H::Digest::from_bool(true),
                self.root,
                &self.empty_hash_at_height,
            )
            .ok_or(ProofError::InvalidNullifierPath)?;

        Ok(())
    }
}
