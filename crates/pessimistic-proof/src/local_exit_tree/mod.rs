use agglayer_primitives::keccak::Hasher;
use serde::{de::DeserializeOwned, Serialize};
use unified_bridge::LocalExitTreeError;

use crate::local_exit_tree::data::LocalExitTreeData;
pub mod data;

pub use unified_bridge::LocalExitTree;

#[cfg(test)]
mod tests;

impl<H, const TREE_DEPTH: usize> TryFrom<&LocalExitTreeData<H, TREE_DEPTH>>
    for LocalExitTree<H, TREE_DEPTH>
where
    H: Hasher,
    H::Digest: std::fmt::Debug + Copy + Default + Serialize + DeserializeOwned + rkyv::Archive,
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
