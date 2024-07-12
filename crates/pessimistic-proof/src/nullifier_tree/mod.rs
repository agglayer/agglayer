use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use crate::bridge_exit::{NetworkId};
use crate::imported_bridge_exit::{ImportedBridgeExit};
use crate::keccak::Digest;
use crate::local_exit_tree::hasher::Hasher;
use serde_with::serde_as;



/// A commitment to the set of per-network nullifier sets maintained by the local network
/// TODO: determine the appropriate TREE_DEPTH - unlikely that we'll have 4 billion chains :)
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NullifierTree<H, const TREE_DEPTH: usize = 16> where
    H: Hasher,
    H::Digest: Serialize + for<'a> Deserialize<'a>,
{
    /// The Merkle Root of the nullifier set
    #[serde_as(as = "_")]
    pub root: H::Digest,
}

impl<H, const TREE_DEPTH: usize> NullifierTree<H, TREE_DEPTH>
where
    H: Hasher,
    H::Digest: Copy + Default + Serialize + for<'a> Deserialize<'a>,
{
    pub fn new() -> Self {
        NullifierTree {
            root: H::Digest::default(),
        }
    }

    pub fn default() -> Self {
        Self::new()
    }


    pub fn recompute_root(&mut self, paths_to_update: Vec<(NullifierPath, NetworkNullifierTree<H>)>) {
        // TODO: verify that each nullifier path is valid and consistent with the current root.

        // TODO: recompute the root by replacing each NetworkNullifierSet root at the specified path. Check that the network ID matches.
    }
}

// TODO: implement hashing for the nullifier set. Insert the root of each non-empty NetworkNullifierSet at the index in NullifierSet matching foreign_network_id

/// The nullifier sets for each foreign network tracked by the local network.
/// Each network nullifier set can be thought of as a bit mask, where each index
/// corresponds to an index in the foreign network's local exit tree.
/// The value at an index is 1 if the message has been claimed by the local network,
/// and 0 if it has not.
///
/// The Merkle structure is a fixed-depth SMT, where each leaf maps to a leaf index in the
/// foreign network's LET.
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkNullifierTree<H, const TREE_DEPTH: usize = 32> where
    H: Hasher,
    H::Digest: Serialize + for<'a> Deserialize<'a>,
{
    /// The root of a Network nullifier set
    #[serde_as(as = "_")]
    pub root: H::Digest,

    #[serde_as(as = "_")]
    pub network_id:NetworkId
}

impl<H, const TREE_DEPTH: usize> NetworkNullifierTree<H, TREE_DEPTH> where
    H: Hasher,
    H::Digest: Copy + Default + Serialize + for<'a> Deserialize<'a>,
{
    pub fn new() -> Self {
        NetworkNullifierTree {
            root: H::Digest::default(),
            network_id:NetworkId::default(),
        }
    }

    pub fn recompute_root(&mut self, claimed_paths:Vec<NetworkNullifierPath>) {
        // TODO: verify that each network nullifier path is correct and consistent with the current NNS root. Verify that the value is set to zero at the leaf.

        // TODO: recompute the root sequentially by flipping each leaf value to 1 and then rehashing.
    }
}

// TODO: implement hashing for the network nullifier set. Maybe a zero-initialized tree with num_leaves = first claimed index, then adding zero-initialized subtrees as the number of leaves increases. We can update a claim by taking the current path to the index and re-hashing.

/// TODO: actually implement this. Should have network_id and inclusion proof in the nullifier set
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NullifierPath((NetworkId, Vec<Digest>));

/// TODO: actually implement this. Should have leaf_index and inclusion proof in the network nullifier set
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkNullifierPath((u32, Vec<Digest>));

