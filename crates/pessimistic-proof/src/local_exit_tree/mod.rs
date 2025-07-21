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

    fn try_from(data: &LocalExitTreeData<TREE_DEPTH>) -> Result<Self, Self::Error> {
        let leaf_count = data.layers[0].len();
        let mut frontier = [Digest::default(); TREE_DEPTH];
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
