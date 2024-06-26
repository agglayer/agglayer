use serde::{Deserialize, Serialize};
use serde_with::serde_as;

pub mod hasher;
use hasher::Hasher;

#[cfg(test)]
mod tests;

/// Represents a local exit tree as defined by the LxLy bridge.
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocalExitTree<H, const TREE_DEPTH: usize = 32>
where
    H: Hasher,
    H::Digest: Serialize + for<'a> Deserialize<'a>,
{
    /// The number of inserted (non-empty) leaves.
    leaf_count: u32,
    #[serde_as(as = "[_; TREE_DEPTH]")]
    frontier: [H::Digest; TREE_DEPTH],
}

impl<H, const TREE_DEPTH: usize> LocalExitTree<H, TREE_DEPTH>
where
    H: Hasher,
    H::Digest: Copy + Default + Serialize + for<'a> Deserialize<'a>,
{
    /// Creates a new empty [`LocalExitTree`].
    pub fn new() -> Self {
        LocalExitTree {
            leaf_count: 0,
            frontier: [H::Digest::default(); TREE_DEPTH],
        }
    }

    /// Creates a new [`LocalExitTree`] and populates its leaves.
    pub fn from_leaves(leaves: impl Iterator<Item = H::Digest>) -> Self {
        let mut tree = Self::new();

        for leaf in leaves {
            tree.add_leaf(leaf);
        }

        tree
    }

    /// Creates a new [`LocalExitTree`] from its parts: leaf count, and frontier.
    pub fn from_parts(leaf_count: u32, frontier: [H::Digest; TREE_DEPTH]) -> Self {
        Self {
            leaf_count,
            frontier,
        }
    }

    /// Appends a leaf to the tree.
    pub fn add_leaf(&mut self, leaf: H::Digest) {
        // the index at which the new entry will be inserted
        let frontier_insertion_index: usize = {
            let leaf_count_after_insertion = self.leaf_count + 1;

            leaf_count_after_insertion
                .trailing_zeros()
                .try_into()
                .expect("usize expected to be at least 32 bits")
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
        self.leaf_count += 1;
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

impl<H, const TREE_DEPTH: usize> Default for LocalExitTree<H, TREE_DEPTH>
where
    H: Hasher,
    H::Digest: Copy + Default + Serialize + for<'a> Deserialize<'a>,
{
    fn default() -> Self {
        Self::new()
    }
}

/// Returns the bit value at index `bit_idx` in `target`
fn get_bit_at(target: u32, bit_idx: usize) -> u32 {
    (target >> bit_idx) & 1
}
