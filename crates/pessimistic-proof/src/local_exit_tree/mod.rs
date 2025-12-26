use agglayer_primitives::Digest;
use unified_bridge::LocalExitTreeError;

use crate::local_exit_tree::data::LocalExitTreeData;
pub mod data;

pub use unified_bridge::LocalExitTree;

#[cfg(test)]
mod tests;

impl<const TREE_DEPTH: usize> TryFrom<&LocalExitTreeData<TREE_DEPTH>>
    for LocalExitTree<TREE_DEPTH>
{
    type Error = LocalExitTreeError;

    /// Converts a full tree representation into a frontier-based
    /// representation.
    ///
    /// The frontier is a compact representation of a Merkle tree that stores
    /// only the "right siblings" needed to reconstruct the root. For an
    /// incremental tree with `n` leaves, the frontier contains the right
    /// sibling of each odd-indexed node along the path from the last leaf
    /// to the root.
    ///
    /// The algorithm traverses the path from the last leaf (`leaf_count`)
    /// upward to the root, collecting right siblings at each level where
    /// the current node is a left child (i.e., when `index & 1 == 1`).
    fn try_from(data: &LocalExitTreeData<TREE_DEPTH>) -> Result<Self, Self::Error> {
        let leaf_count = data.layers[0].len();
        let mut frontier = [Digest::default(); TREE_DEPTH];
        // Start from the last leaf index and traverse upward to the root
        let mut index = leaf_count;
        let mut height = 0;
        while index != 0 {
            if height >= TREE_DEPTH {
                return Err(LocalExitTreeError::FrontierIndexOutOfBounds);
            }
            // If the current node is a left child (odd index), store its right sibling
            // in the frontier. The right sibling is at index ^ 1 (flips the LSB).
            if index & 1 == 1 {
                frontier[height] = data.get(height, index ^ 1)?;
            }
            height += 1;
            // Move to the parent node by shifting right (dividing by 2)
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
