use std::fmt::Debug;

use pessimistic_proof::local_exit_tree::hasher::Hasher;
use pessimistic_proof::local_exit_tree::LocalExitTree;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

/// Represents a local exit tree as defined by the LxLy bridge.
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocalExitTreeData<H, const TREE_DEPTH: usize = 32>
where
    H: Hasher,
    H::Digest: Serialize + for<'a> Deserialize<'a>,
{
    #[serde_as(as = "[_; TREE_DEPTH]")]
    layers: [Vec<H::Digest>; TREE_DEPTH],
    #[serde_as(as = "[_; TREE_DEPTH]")]
    empty_hash_at_height: [H::Digest; TREE_DEPTH],
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LETMerkleProof<H, const TREE_DEPTH: usize = 32>
where
    H: Hasher,
    H::Digest: Serialize + for<'a> Deserialize<'a>,
{
    #[serde_as(as = "[_; TREE_DEPTH]")]
    siblings: [H::Digest; TREE_DEPTH],
}

impl<H, const TREE_DEPTH: usize> LocalExitTreeData<H, TREE_DEPTH>
where
    H: Hasher,
    H::Digest: Copy + Default + Serialize + for<'a> Deserialize<'a>,
{
    /// Creates a new empty [`LocalExitTreeData`].
    pub fn new() -> Self {
        let mut empty_hash_at_height = [H::Digest::default(); TREE_DEPTH];
        for height in 1..TREE_DEPTH {
            empty_hash_at_height[height] = H::merge(
                &empty_hash_at_height[height - 1],
                &empty_hash_at_height[height - 1],
            );
        }
        LocalExitTreeData {
            layers: std::array::from_fn(|_| Vec::new()),
            empty_hash_at_height,
        }
    }

    /// Creates a new [`LocalExitTreeData`] and populates its leaves.
    pub fn from_leaves(leaves: impl Iterator<Item = H::Digest>) -> Self
    where
        H::Digest: Debug,
    {
        let mut tree = Self::new();

        for leaf in leaves {
            tree.add_leaf(leaf);
        }

        tree
    }

    /// Appends a leaf to the tree.
    pub fn add_leaf(&mut self, leaf: H::Digest) -> usize
    where
        H::Digest: Debug,
    {
        let leaf_index = self.layers[0].len();
        assert_eq!(leaf_index >> TREE_DEPTH, 0, "Too many leaves.");
        self.layers[0].push(leaf);
        let mut index = leaf_index;
        let mut entry = leaf;
        for height in 0..TREE_DEPTH - 1 {
            dbg!(leaf_index, height, index);
            let sibling = self.layers[height]
                .get(index ^ 1)
                .unwrap_or(&self.empty_hash_at_height[height]);
            dbg!(height + 1, index, sibling, entry);
            entry = if index & 1 == 1 {
                H::merge(&sibling, &entry)
            } else {
                H::merge(&entry, sibling)
            };
            index >>= 1;
            if index < self.layers[height + 1].len() {
                self.layers[height + 1][index] = entry;
            } else {
                self.layers[height + 1].push(entry);
            }
        }

        leaf_index
    }

    pub fn is_empty(&self) -> bool {
        self.layers[0].is_empty()
    }

    /// Returns the root of the tree.
    pub fn get_root(&self) -> H::Digest {
        // let frontier: LocalExitTree<_, TREE_DEPTH> = self.into();
        // frontier.get_root()
        let get_last_layer = |i| {
            self.layers[TREE_DEPTH - 1]
                .get(i)
                .unwrap_or(&self.empty_hash_at_height[TREE_DEPTH - 1])
        };
        H::merge(get_last_layer(0), get_last_layer(1))
        // if self.is_empty() {
        //     (0..TREE_DEPTH).fold(H::Digest::default(), |acc, _|
        // H::merge(&acc, &acc)) } else {
        //     assert_eq!(self.layers[TREE_DEPTH - 1].len(), 2);
        //     H::merge(
        //         &self.layers[TREE_DEPTH - 1][0],
        //         &self.layers[TREE_DEPTH - 1][1],
        //     )
        // }
    }

    pub fn get_proof(&self, leaf_index: usize) -> LETMerkleProof<H, TREE_DEPTH> {
        assert!(
            leaf_index < self.layers[0].len(),
            "Leaf index out of bounds."
        );
        let mut siblings = [Default::default(); TREE_DEPTH];
        let mut index = leaf_index;
        let mut empty_hash_at_height = H::Digest::default();
        for height in 0..TREE_DEPTH {
            let sibling = *self.layers[height]
                .get(index ^ 1)
                .unwrap_or(&empty_hash_at_height);
            siblings[height] = sibling;
            empty_hash_at_height = H::merge(&empty_hash_at_height, &empty_hash_at_height);
            index >>= 1;
        }

        LETMerkleProof { siblings }
    }
}

// impl<H, const TREE_DEPTH: usize> Into<LocalExitTree<H, TREE_DEPTH>>
//     for &LocalExitTreeData<H, TREE_DEPTH>
// where
//     H: Hasher,
//     H::Digest: Copy + Default + Serialize + for<'a> Deserialize<'a>,
// {
//     fn into(self) -> LocalExitTree<H, TREE_DEPTH> {
//         let mut frontier = [H::Digest::default(); TREE_DEPTH];
//         for height in 0..TREE_DEPTH {
//             if let Some(hash) = self.layers[height].last() {
//                 frontier[height] = *hash;
//             } else {
//                 break;
//             }
//         }
//
//         LocalExitTree::from_parts(
//             self.layers[0]
//                 .len()
//                 .try_into()
//                 .expect("usize expected to be at least 32 bits"),
//             frontier,
//         )
//     }
// }

impl<H, const TREE_DEPTH: usize> LETMerkleProof<H, TREE_DEPTH>
where
    H: Hasher,
    H::Digest: Eq + Copy + Default + Serialize + for<'a> Deserialize<'a>,
{
    pub fn verify(&self, leaf: H::Digest, leaf_index: usize, root: H::Digest) -> bool {
        dbg!(1);
        let mut entry = leaf;
        let mut index = leaf_index;
        for &sibling in &self.siblings {
            entry = if index & 1 == 0 {
                H::merge(&entry, &sibling)
            } else {
                H::merge(&sibling, &entry)
            };
            index >>= 1;
        }
        dbg!(3);
        if index != 0 {
            return false;
        }
        dbg!(4);
        if entry != root {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use pessimistic_proof::local_exit_tree::hasher::Keccak256Hasher;
    use pessimistic_proof::local_exit_tree::LocalExitTree;
    use rand::{random, thread_rng, Rng};

    use crate::local_exit_tree::LocalExitTreeData;

    const TREE_DEPTH: usize = 32;
    type H = Keccak256Hasher;

    fn compare_let_data_let_frontier(num_leaves: usize) {
        let leaves = (0..num_leaves).map(|_| random()).collect::<Vec<_>>();
        // let leaves = (0..num_leaves)
        //     .map(|i| [(i + 1) as u8; 32])
        //     .collect::<Vec<_>>();
        let local_exit_tree_frontier: LocalExitTree<H, TREE_DEPTH> =
            LocalExitTree::from_leaves(leaves.iter().cloned());
        dbg!(leaves.len(), TREE_DEPTH);
        let local_exit_tree_data: LocalExitTreeData<H, TREE_DEPTH> =
            LocalExitTreeData::from_leaves(leaves.into_iter());
        dbg!(&local_exit_tree_data);
        dbg!(&local_exit_tree_frontier);
        assert_eq!(
            local_exit_tree_frontier.get_root(),
            local_exit_tree_data.get_root()
        );
    }

    #[test]
    fn test_let_empty() {
        compare_let_data_let_frontier(0)
    }

    #[test]
    fn test_let() {
        let num_leaves = thread_rng().gen_range(1..100.min(1 << TREE_DEPTH));
        dbg!(num_leaves);
        compare_let_data_let_frontier(num_leaves)
    }

    #[test]
    fn test_merkle_proofs() {
        let num_leaves = thread_rng().gen_range(1..=100.min(1 << TREE_DEPTH));
        // let num_leaves = 3;
        let leaves = (0..num_leaves).map(|_| random()).collect::<Vec<_>>();
        // let leaves = vec![[1; 32], [2; 32], [3; 32]];
        let leaf_index = thread_rng().gen_range(0..num_leaves);
        // let leaf_index = 1;
        let leaf = leaves[leaf_index];
        let local_exit_tree_data: LocalExitTreeData<H, TREE_DEPTH> =
            LocalExitTreeData::from_leaves(leaves.into_iter());
        let root = local_exit_tree_data.get_root();
        let proof = local_exit_tree_data.get_proof(leaf_index);
        dbg!(&proof);
        dbg!(&local_exit_tree_data);
        // let frontier: LocalExitTree<Keccak256Hasher, TREE_DEPTH> =
        // (&local_exit_tree_data).into(); dbg!(frontier);
        dbg!(&root);
        assert!(proof.verify(leaf, leaf_index, root));
    }
}
