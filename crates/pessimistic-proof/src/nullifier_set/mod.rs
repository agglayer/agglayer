use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use crate::bridge_exit::{NetworkId};
use crate::imported_bridge_exit::{ImportedBridgeExit};

/// A commitment to the set of per-network nullifier sets maintained by the local network
/// TODO: determine the appropriate TREE_DEPTH - unlikely that we'll have 4 billion chains :)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NullifierSet(BTreeMap<NetworkId, NetworkNullifierSet>);

impl NullifierSet {
    pub fn new() -> NullifierSet {
        NullifierSet(BTreeMap::<NetworkId, NetworkNullifierSet>::new())
    }

    /// Takes an imported bridge exit and updates the nullifier set to reflect that it's been claimed
    pub fn claim_bridge_exit(&mut self, network_id:NetworkId, leaf_index:u32) {
        if !self.contains_key(network_id) {
            self.insert(network_id, vec![leaf_index]);
        }else {
            self[network_id].add_nullifier_for_imported_exit(leaf_index);
        }
    }
}

// TODO: implement hashing for the nullifier set. Insert the root of each non-empty NetworkNullifierSet at the index in NullifierSet matching foreign_network_id

/// The nullifier sets for each foreign network tracked by the local network.
/// Each network nullifier set can be represented as 0-initialized bit masks,
/// where each index corresponds to an index in the foreign network's local exit tree.
/// The value at an index is 1 if the message has been claimed by the local network,
/// and 0 if it has not.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkNullifierSet <const TREE_DEPTH: usize = 32> {
    /// The set of indices that have been claimed.
    pub claimed_indices: Vec<u32>,
}

impl<const TREE_DEPTH: usize> NetworkNullifierSet<TREE_DEPTH> {

    pub fn new() -> Self {
        let mut claimed_indices = Vec::<u32>::new();
        NetworkNullifierSet {
            claimed_indices
        }
    }

    pub fn new_from_indices(claimed_indices:Vec<u32>) -> Self {
        NetworkNullifierSet {
            claimed_indices
        }
    }

    pub fn add_nullifier_for_imported_exit(&mut self, leaf_index:u32) {
        if !self.claimed_indices.contains(&leaf_index) {
           self.claimed_indices.push(leaf_index);
        }else {
            // TODO: what's the best way to handle errors? Panic?
        }
    }
}

// TODO: implement hashing for the network nullifier set. Maybe a zero-initialized tree with num_leaves = first claimed index, then adding zero-initialized subtrees as the number of leaves increases. We can update a claim by taking the current path to the index and re-hashing.