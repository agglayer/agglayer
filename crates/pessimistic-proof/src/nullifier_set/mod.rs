use std::collections::BTreeMap;
use crate::bridge_exit::{NetworkId};

/// A commitment to the set of per-network nullifier sets maintained by the local network
/// TODO: determine the appropriate TREE_DEPTH - unlikely that we'll have 4 billion chains :)
pub struct NullifierSet(BTreeMap<NetworkId, NetworkNullifierSet>);

// TODO: implement hashing for the nullifier set. Insert the root of each non-empty NetworkNullifierSet at the index in NullifierSet matching foreign_network_id

/// The nullifier sets for each network. These can be represented as 0-initialized bit masks,
/// where each index corresponds to an index in the network's local exit tree.
/// The value at an index is 1 if the message has been claimed by the local network,
/// and 0 if it has not.
pub struct NetworkNullifierSet <const TREE_DEPTH: usize = 32> {
    /// The set of indices that have been claimed.
    pub claimed_indices: Vec<u32>,
}

// TODO: implement hashing for the network nullifier set. Would recommend a zero-initialized tree with num_leaves = first claimed index, then adding zero-initialized subtrees as the number of leaves increases. We can update a claim by taking the current path to the index and re-hashing.