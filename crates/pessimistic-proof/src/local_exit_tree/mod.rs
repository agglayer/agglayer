use pessimistic_proof_core::local_exit_tree::hasher::Hasher;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_with::serde_as;

use crate::local_exit_tree::data::LocalExitTreeData;
pub mod data;

pub use pessimistic_proof_core::local_exit_tree::hasher;
pub use pessimistic_proof_core::local_exit_tree::LocalExitTreeError;

#[cfg(test)]
mod tests;

impl<H, const TREE_DEPTH: usize> TryFrom<&LocalExitTreeData<H, TREE_DEPTH>>
    for LocalExitTree<H, TREE_DEPTH>
where
    H: Hasher,
    H::Digest: std::fmt::Debug + Copy + Default + Serialize + DeserializeOwned,
{
    type Error = LocalExitTreeError;

    fn try_from(data: &LocalExitTreeData<H, TREE_DEPTH>) -> Result<Self, Self::Error> {
        let leaf_count = data.layers[0].len();
        let mut frontier = [H::Digest::default(); TREE_DEPTH];
        let mut index = leaf_count;
        let mut height = 0;
        while index != 0 {
            if height >= TREE_DEPTH {
                return Err(LocalExitTreeError::FrontierIndexOutOfBounds);
            }
            if index & 1 == 1 {
                frontier[height] = data.get(height, index ^ 1)?;
            }
            height += 1;
            index >>= 1;
        }

        Ok(LocalExitTree::from_parts(
            leaf_count
                .try_into()
                .map_err(|_| LocalExitTreeError::LeafIndexOverflow)?,
            frontier,
        ))
    }
}

/// Represents a local exit tree as defined by the LxLy bridge.
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocalExitTree<H, const TREE_DEPTH: usize = 32>
where
    H: Hasher,
    H::Digest: Copy + std::fmt::Debug + Serialize + for<'a> Deserialize<'a>,
{
    inner: pessimistic_proof_core::local_exit_tree::LocalExitTree<H, TREE_DEPTH>,
}

impl<H, const TREE_DEPTH: usize> Default for LocalExitTree<H, TREE_DEPTH>
where
    H: Hasher,
    H::Digest: std::fmt::Debug + Copy + Default + Serialize + for<'a> Deserialize<'a>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<H, const TREE_DEPTH: usize> LocalExitTree<H, TREE_DEPTH>
where
    H: Hasher,
    H::Digest: std::fmt::Debug + Copy + Default + Serialize + for<'a> Deserialize<'a>,
{
    /// Creates a new empty [`LocalExitTree`].
    pub fn new() -> Self {
        Self {
            inner: pessimistic_proof_core::local_exit_tree::LocalExitTree {
                leaf_count: 0,
                frontier: [H::Digest::default(); TREE_DEPTH],
            },
        }
    }

    pub fn leaf_count(&self) -> u32 {
        self.inner.leaf_count
    }

    pub fn frontier(&self) -> [H::Digest; TREE_DEPTH] {
        self.inner.frontier
    }

    /// Creates a new [`LocalExitTree`] and populates its leaves.
    pub fn from_leaves(
        leaves: impl Iterator<Item = H::Digest>,
    ) -> Result<Self, LocalExitTreeError> {
        let mut tree = Self::new();

        for leaf in leaves {
            tree.add_leaf(leaf)?;
        }

        Ok(tree)
    }

    /// Creates a new [`LocalExitTree`] from its parts: leaf count, and
    /// frontier.
    pub fn from_parts(leaf_count: u32, frontier: [H::Digest; TREE_DEPTH]) -> Self {
        Self {
            inner: pessimistic_proof_core::local_exit_tree::LocalExitTree {
                leaf_count,
                frontier,
            },
        }
    }
}

impl<H, const TREE_DEPTH: usize> LocalExitTree<H, TREE_DEPTH>
where
    H: Hasher,
    H::Digest: std::fmt::Debug + Copy + Default + Serialize + for<'a> Deserialize<'a>,
{
    /// Appends a leaf to the tree.
    pub fn add_leaf(&mut self, leaf: H::Digest) -> Result<u32, LocalExitTreeError> {
        self.inner.add_leaf(leaf)
    }

    /// Computes and returns the root of the tree.
    pub fn get_root(&self) -> H::Digest {
        self.inner.get_root()
    }
}

impl<H, const TREE_DEPTH: usize> From<LocalExitTree<H, TREE_DEPTH>>
    for pessimistic_proof_core::local_exit_tree::LocalExitTree<H, TREE_DEPTH>
where
    H: Hasher,
    H::Digest: std::fmt::Debug + Copy + Default + Serialize + for<'a> Deserialize<'a>,
{
    fn from(value: LocalExitTree<H, TREE_DEPTH>) -> Self {
        value.inner
    }
}
