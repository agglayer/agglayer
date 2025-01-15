use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use thiserror::Error;

pub mod hasher;
use hasher::Hasher;

pub mod proof;

/// Represents a local exit tree as defined by the LxLy bridge.
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocalExitTree<H, const TREE_DEPTH: usize = 32>
where
    H: Hasher,
    H::Digest: Serialize + for<'a> Deserialize<'a>,
{
    /// The number of inserted (non-empty) leaves.
    pub leaf_count: u32,
    #[serde_as(as = "[_; TREE_DEPTH]")]
    pub frontier: [H::Digest; TREE_DEPTH],
}

#[derive(Clone, Debug, Error, Serialize, Deserialize, PartialEq, Eq)]
pub enum LocalExitTreeError {
    #[error("Leaf index overflow")]
    LeafIndexOverflow,
    #[error("Index out of bounds")]
    IndexOutOfBounds,
    #[error("Frontier index out of bounds")]
    FrontierIndexOutOfBounds,
}

impl<H, const TREE_DEPTH: usize> LocalExitTree<H, TREE_DEPTH>
where
    H: Hasher,
    H::Digest: Copy + Default + Serialize + for<'a> Deserialize<'a>,
{
    const MAX_NUM_LEAVES: u32 = ((1u64 << TREE_DEPTH) - 1) as u32;

    /// Appends a leaf to the tree.
    pub fn add_leaf(&mut self, leaf: H::Digest) -> Result<u32, LocalExitTreeError> {
        if self.leaf_count >= Self::MAX_NUM_LEAVES {
            return Err(LocalExitTreeError::LeafIndexOverflow);
        }
        // the index at which the new entry will be inserted
        let frontier_insertion_index: usize = {
            let leaf_count_after_insertion = self.leaf_count + 1;

            leaf_count_after_insertion.trailing_zeros() as usize
        };

        // the new entry to be inserted in the frontier
        let new_frontier_entry = {
            let mut entry = leaf;
            for frontier_ele in &self.frontier[0..frontier_insertion_index] {
                entry = H::merge(frontier_ele, &entry);
            }

            entry
        };

        // update tree
        self.frontier[frontier_insertion_index] = new_frontier_entry;
        self.leaf_count = self
            .leaf_count
            .checked_add(1)
            .ok_or(LocalExitTreeError::LeafIndexOverflow)?;

        Ok(self.leaf_count)
    }

    /// Computes and returns the root of the tree.
    pub fn get_root(&self) -> H::Digest {
        let mut root = H::Digest::default();
        let mut empty_hash_at_height = H::Digest::default();

        for height in 0..TREE_DEPTH {
            if get_bit_at(self.leaf_count, height) == 1 {
                root = H::merge(&self.frontier[height], &root);
            } else {
                root = H::merge(&root, &empty_hash_at_height);
            }

            empty_hash_at_height = H::merge(&empty_hash_at_height, &empty_hash_at_height);
        }

        root
    }
}

/// Returns the bit value at index `bit_idx` in `target`
fn get_bit_at(target: u32, bit_idx: usize) -> u32 {
    (target >> bit_idx) & 1
}
