use rand::random;
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

fn local_exit_tree_is_subtree(leaf_count_a: usize, leaf_count_b: usize) {
    assert!(leaf_count_a <= leaf_count_b);
    let mut let_a = LocalExitTreeData::<Keccak256Hasher>::new();
    for _ in 0..leaf_count_a {
        let_a.add_leaf(random());
    }
    let mut let_b = let_a.clone();
    for _ in 0..leaf_count_b - leaf_count_a {
        let_b.add_leaf(random());
    }

    let let_a_frontier = LocalExitTree::from(&let_a);
    let proof = let_b.get_proof(leaf_count_a as u32);
    assert!(let_a_frontier.is_subtree_of(let_b.get_root(), let_b.get(0, leaf_count_a), proof));
}

#[test]
fn test_local_exit_tree_is_subtree_different_sizes() {
    local_exit_tree_is_subtree(100, 200);
}

#[test]
fn test_local_exit_tree_is_subtree_empty() {
    local_exit_tree_is_subtree(0, 100);
}

#[test]
fn test_local_exit_tree_is_subtree_same_tree() {
    let leaf_count_a = 100;
    let mut let_a = LocalExitTreeData::<Keccak256Hasher>::new();
    for _ in 0..leaf_count_a {
        let_a.add_leaf(random());
    }
    let mut let_b = let_a.clone();
    let_b.add_leaf([0; 32]); // Hack so that `let_b.get_proof(leaf_count_a)` doesn't panic

    let let_a_frontier = LocalExitTree::from(&let_a);
    let proof = let_b.get_proof(leaf_count_a as u32);
    assert!(let_a_frontier.is_subtree_of(let_b.get_root(), let_b.get(0, leaf_count_a), proof));
}

#[test]
fn test_local_exit_tree_is_subtree_failing() {
    let leaf_count_a = 100;
    let leaf_count_b = 200;
    let mut let_a = LocalExitTreeData::<Keccak256Hasher>::new();
    for _ in 0..leaf_count_a {
        let_a.add_leaf(random());
    }
    let mut let_b = LocalExitTreeData::<Keccak256Hasher>::new();
    for _ in 0..leaf_count_b {
        let_b.add_leaf(random());
    }
    assert!(
        (0..leaf_count_a).any(|i| let_a.get(0, i) != let_b.get(0, i)),
        "Check your RNG"
    );

    let let_a_frontier = LocalExitTree::from(&let_a);
    let proof = let_b.get_proof(leaf_count_a as u32);
    assert!(!let_a_frontier.is_subtree_of(let_b.get_root(), let_b.get(0, leaf_count_a), proof));
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
