use pessimistic_proof_core::local_exit_tree::{
    hasher::Hasher, proof::LETMerkleProof, LocalExitTreeError,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_with::serde_as;

use crate::utils::empty_hash::empty_hash_at_height;

/// Represents a local exit tree as defined by the LxLy bridge.
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocalExitTreeData<H, const TREE_DEPTH: usize = 32>
where
    H: Hasher,
    H::Digest: Serialize + DeserializeOwned,
{
    /// The layers of the Merkle tree from bottom to top (i.e., the leaves are
    /// in `layers[0]`)
    #[serde_as(as = "[_; TREE_DEPTH]")]
    pub layers: [Vec<H::Digest>; TREE_DEPTH],
    /// `empty_hash_at_height[i]` is the root of an empty Merkle tree of depth
    /// `i`.
    #[serde_as(as = "[_; TREE_DEPTH]")]
    empty_hash_at_height: [H::Digest; TREE_DEPTH],
}

impl<H, const TREE_DEPTH: usize> Default for LocalExitTreeData<H, TREE_DEPTH>
where
    H: Hasher,
    H::Digest: Copy + Default + Serialize + DeserializeOwned,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<H, const TREE_DEPTH: usize> LocalExitTreeData<H, TREE_DEPTH>
where
    H: Hasher,
    H::Digest: Copy + Default + Serialize + DeserializeOwned,
{
    const MAX_NUM_LEAVES: u32 = ((1u64 << TREE_DEPTH) - 1) as u32;

    /// Creates a new empty [`LocalExitTreeData`].
    pub fn new() -> Self {
        let empty_hash_at_height = empty_hash_at_height::<H, TREE_DEPTH>();
        LocalExitTreeData {
            layers: std::array::from_fn(|_| Vec::new()),
            empty_hash_at_height,
        }
    }

    /// Creates a new [`LocalExitTreeData`] and populates its leaves.
    pub fn from_leaves(
        leaves: impl Iterator<Item = H::Digest>,
    ) -> Result<Self, LocalExitTreeError> {
        let mut tree = Self::new();

        for leaf in leaves {
            tree.add_leaf(leaf)?;
        }

        Ok(tree)
    }

    /// Appends a leaf to the tree.
    pub fn add_leaf(&mut self, leaf: H::Digest) -> Result<u32, LocalExitTreeError> {
        let leaf_index = self.layers[0].len();

        if leaf_index >= Self::MAX_NUM_LEAVES as usize {
            return Err(LocalExitTreeError::LeafIndexOverflow);
        }
        self.layers[0].push(leaf);
        let mut index = leaf_index;
        let mut entry = leaf;
        for height in 0..TREE_DEPTH - 1 {
            let sibling = self.get(height, index ^ 1)?;
            entry = if index & 1 == 1 {
                H::merge(&sibling, &entry)
            } else {
                H::merge(&entry, &sibling)
            };
            index >>= 1;
            if index < self.layers[height + 1].len() {
                self.layers[height + 1][index] = entry;
            } else {
                self.layers[height + 1].push(entry);
            }
        }

        leaf_index
            .try_into()
            .map_err(|_| LocalExitTreeError::LeafIndexOverflow)
    }

    pub fn get(&self, height: usize, index: usize) -> Result<H::Digest, LocalExitTreeError> {
        if index >= 1 << (TREE_DEPTH - height) {
            return Err(LocalExitTreeError::IndexOutOfBounds);
        }
        Ok(*self.layers[height]
            .get(index)
            .unwrap_or(&self.empty_hash_at_height[height]))
    }

    pub fn is_empty(&self) -> bool {
        self.layers[0].is_empty()
    }

    /// Returns the root of the tree.
    pub fn get_root(&self) -> H::Digest {
        let get_last_layer = |i| {
            self.layers[TREE_DEPTH - 1]
                .get(i)
                .unwrap_or(&self.empty_hash_at_height[TREE_DEPTH - 1])
        };
        H::merge(get_last_layer(0), get_last_layer(1))
    }

    pub fn get_proof(
        &self,
        leaf_index: u32,
    ) -> Result<LETMerkleProof<H, TREE_DEPTH>, LocalExitTreeError> {
        let leaf_index: usize = leaf_index
            .try_into()
            .map_err(|_| LocalExitTreeError::LeafIndexOverflow)?;
        if leaf_index >= self.layers[0].len() {
            return Err(LocalExitTreeError::IndexOutOfBounds);
        }
        let mut siblings = [Default::default(); TREE_DEPTH];
        let mut index = leaf_index;

        for (height, sibling) in siblings.iter_mut().enumerate().take(TREE_DEPTH) {
            *sibling = self.get(height, index ^ 1)?;
            index >>= 1;
        }

        Ok(LETMerkleProof { siblings })
    }
}

#[cfg(test)]
mod tests {
    use pessimistic_proof_core::local_exit_tree::hasher::Keccak256Hasher;
    use rand::{random, thread_rng, Rng};

    use crate::local_exit_tree::{data::LocalExitTreeData, LocalExitTree, LocalExitTreeError};

    const TREE_DEPTH: usize = 32;
    type H = Keccak256Hasher;

    fn compare_let_data_let_frontier(num_leaves: usize) {
        let leaves = (0..num_leaves).map(|_| random()).collect::<Vec<_>>();
        let local_exit_tree_frontier: LocalExitTree<H, TREE_DEPTH> =
            LocalExitTree::from_leaves(leaves.iter().cloned()).unwrap();
        let local_exit_tree_data: LocalExitTreeData<H, TREE_DEPTH> =
            LocalExitTreeData::from_leaves(leaves.into_iter()).unwrap();
        assert_eq!(
            local_exit_tree_frontier.get_root(),
            local_exit_tree_data.get_root()
        );
    }

    #[test]
    fn test_data_vs_frontier_empty() {
        compare_let_data_let_frontier(0)
    }

    #[test]
    fn test_data_vs_frontier_root() {
        let num_leaves = thread_rng().gen_range(1..100.min(1 << TREE_DEPTH));
        compare_let_data_let_frontier(num_leaves)
    }

    #[test]
    fn test_data_vs_frontier_add_leaf() -> Result<(), LocalExitTreeError> {
        let num_leaves = thread_rng().gen_range(1usize..100.min(1 << TREE_DEPTH));
        let leaves = (0..num_leaves).map(|_| random()).collect::<Vec<_>>();
        let mut local_exit_tree_data: LocalExitTreeData<H, TREE_DEPTH> =
            LocalExitTreeData::from_leaves(leaves.into_iter())?;
        let mut local_exit_tree_frontier: LocalExitTree<_, TREE_DEPTH> =
            LocalExitTree::try_from(&local_exit_tree_data)?;
        assert_eq!(
            local_exit_tree_data.get_root(),
            local_exit_tree_frontier.get_root()
        );
        let leaf = random();
        local_exit_tree_data.add_leaf(leaf)?;
        local_exit_tree_frontier.add_leaf(leaf)?;
        assert_eq!(
            local_exit_tree_data.get_root(),
            local_exit_tree_frontier.get_root()
        );
        Ok(())
    }

    #[test]
    fn test_merkle_proofs() {
        let num_leaves = thread_rng().gen_range(1..=100.min(1 << TREE_DEPTH));
        let leaves = (0..num_leaves).map(|_| random()).collect::<Vec<_>>();
        let leaf_index = thread_rng().gen_range(0..num_leaves);
        let leaf = leaves[leaf_index];
        let local_exit_tree_data: LocalExitTreeData<H, TREE_DEPTH> =
            LocalExitTreeData::from_leaves(leaves.into_iter()).unwrap();
        let root = local_exit_tree_data.get_root();
        let proof = local_exit_tree_data.get_proof(leaf_index as u32).unwrap();
        assert!(proof.verify(leaf, leaf_index as u32, root));
    }
}
