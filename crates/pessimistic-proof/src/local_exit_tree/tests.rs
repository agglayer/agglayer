use rs_merkle::{Hasher as MerkleHasher, MerkleTree};
use tiny_keccak::{Hasher as _, Keccak};

use super::*;
use crate::local_exit_tree::hasher::Keccak256Hasher;

#[test]
fn test_local_exit_tree_basic() {
    const TREE_DEPTH: usize = 3;
    let leaves = [[1_u8; 32], [2_u8; 32], [3_u8; 32]];

    let local_exit_tree: LocalExitTree<Keccak256Hasher, TREE_DEPTH> =
        LocalExitTree::from_leaves(leaves.into_iter());

    let ground_truth_tree: MerkleTree<TestKeccak256> = {
        // explicitly add the other empty leaves to fill the bottom layer
        let leaves: Vec<_> = leaves.into_iter().chain([[0_u8; 32]; 5]).collect();

        MerkleTree::from_leaves(&leaves)
    };

    assert_eq!(
        ground_truth_tree.root().unwrap(),
        local_exit_tree.get_root()
    );
}

#[derive(Clone, Debug)]
pub struct TestKeccak256;

impl MerkleHasher for TestKeccak256 {
    type Hash = [u8; 32];

    fn hash(data: &[u8]) -> [u8; 32] {
        let mut keccak256 = Keccak::v256();
        keccak256.update(data);
        let mut output = [0u8; 32];
        keccak256.finalize(&mut output);
        output
    }
}
