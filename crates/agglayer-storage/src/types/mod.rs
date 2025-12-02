use agglayer_types::{
    primitives::Digest, CertificateHeader, CertificateId, CertificateIndex, EpochNumber, Height,
    NetworkId, Proof,
};
use serde::{Deserialize, Serialize};

mod certificate;
pub(crate) mod disabled_network;
mod generated;
pub(crate) mod network_info;

#[derive(Debug, Serialize, Deserialize)]
pub enum MetadataKey {
    LatestSettledEpoch,
    EpochSynchronization, // Actually unused, kept for storage backward compatibility
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MetadataValue {
    LatestSettledEpoch(EpochNumber),
    EpochSynchronization(u64), // Actually unused, kept for storage backward compatibility
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PerEpochMetadataKey {
    SettlementTxHash,
    Packed,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PerEpochMetadataValue {
    SettlementTxHash(Digest),
    Packed(bool),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct SmtKey {
    pub(crate) network_id: u32,
    pub(crate) key_type: SmtKeyType,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum SmtKeyType {
    Root,
    Node(Digest),
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum SmtValue {
    Node(Digest, Digest),
    Leaf(Digest),
}

crate::columns::impl_codec_using_bincode_for!(
    u64,
    u32,
    CertificateId,
    CertificateIndex,
    CertificateHeader,
    Digest,
    Height,
    MetadataKey,
    MetadataValue,
    NetworkId,
    PerEpochMetadataKey,
    PerEpochMetadataValue,
    Proof,
    SmtKey,
    SmtValue,
    network_info::Key
);

#[cfg(feature = "testutils")]
pub mod testutils {
    use super::{Digest, NetworkId, SmtKey, SmtKeyType, SmtValue};
    
    /// Generate a sparse merkle tree for testing.
    /// 
    /// Returns a vector of (SmtKey, SmtValue) pairs that can be inserted into the database.
    /// The tree includes:
    /// - A root node
    /// - A configurable number of internal nodes
    /// - A configurable number of leaf nodes
    /// 
    /// # Arguments
    /// * `seed` - Seed for deterministic random generation
    /// * `network_id` - Network ID for the tree
    /// * `num_internal_nodes` - Number of internal nodes to generate
    /// * `num_leaf_nodes` - Number of leaf nodes to generate
    pub fn generate_smt_for_test(
        seed: u64,
        network_id: NetworkId,
        num_internal_nodes: usize,
        num_leaf_nodes: usize,
    ) -> Vec<(SmtKey, SmtValue)> {
        use rand::{Rng, SeedableRng};
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let mut entries = Vec::new();

        // 1. Generate root node
        let root_key = SmtKey {
            network_id: network_id.to_u32(),
            key_type: SmtKeyType::Root,
        };
        let root_left = Digest(rng.random::<[u8; 32]>());
        let root_right = Digest(rng.random::<[u8; 32]>());
        entries.push((root_key, SmtValue::Node(root_left, root_right)));

        // 2. Generate internal nodes
        for _ in 0..num_internal_nodes {
            let node_hash = Digest(rng.random::<[u8; 32]>());
            let node_key = SmtKey {
                network_id: network_id.to_u32(),
                key_type: SmtKeyType::Node(node_hash),
            };
            let left = Digest(rng.random::<[u8; 32]>());
            let right = Digest(rng.random::<[u8; 32]>());
            entries.push((node_key, SmtValue::Node(left, right)));
        }

        // 3. Generate leaf nodes
        for _ in 0..num_leaf_nodes {
            let leaf_hash = Digest(rng.random::<[u8; 32]>());
            let leaf_key = SmtKey {
                network_id: network_id.to_u32(),
                key_type: SmtKeyType::Node(leaf_hash),
            };
            let leaf_value = Digest(rng.random::<[u8; 32]>());
            entries.push((leaf_key, SmtValue::Leaf(leaf_value)));
        }

        entries
    }

    /// Generate a local exit tree for testing.
    /// 
    /// Returns a vector of (Key, Value) pairs for the local exit tree column family.
    /// The tree includes:
    /// - A leaf count entry
    /// - Configurable number of leaves
    /// - Configurable number of frontier nodes
    /// 
    /// # Arguments
    /// * `seed` - Seed for deterministic random generation
    /// * `network_id` - Network ID for the tree
    /// * `num_leaves` - Number of leaves to generate
    /// * `num_frontier_nodes` - Number of frontier nodes to generate
    pub fn generate_let_for_test(
        seed: u64,
        network_id: NetworkId,
        num_leaves: u32,
        num_frontier_nodes: u32,
    ) -> Vec<(crate::columns::local_exit_tree_per_network::Key, crate::columns::local_exit_tree_per_network::Value)> {
        use rand::{Rng, SeedableRng};
        use crate::columns::local_exit_tree_per_network::{Key, KeyType, Value};
        
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let mut entries = Vec::new();

        // 1. Generate leaf count entry
        let leaves_key = Key {
            network_id: network_id.to_u32(),
            key_type: KeyType::LeafCount,
        };
        entries.push((leaves_key, Value::LeafCount(num_leaves)));

        // 2. Generate leaves
        for leaf_idx in 0..num_leaves {
            let leaf_key = Key {
                network_id: network_id.to_u32(),
                key_type: KeyType::Leaf(leaf_idx),
            };
            let leaf_hash = rng.random::<[u8; 32]>();
            entries.push((leaf_key, Value::Leaf(leaf_hash)));
        }

        // 3. Generate frontier nodes
        for layer in 0..num_frontier_nodes {
            let frontier_key = Key {
                network_id: network_id.to_u32(),
                key_type: KeyType::Frontier(layer),
            };
            let frontier_hash = rng.random::<[u8; 32]>();
            entries.push((frontier_key, Value::Frontier(frontier_hash)));
        }

        entries
    }
}
