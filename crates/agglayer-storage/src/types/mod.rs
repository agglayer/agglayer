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
    use pessimistic_proof::keccak::keccak256_combine;
    
    /// Generate a sparse merkle tree for testing.
    /// 
    /// Returns a vector of (SmtKey, SmtValue) pairs that can be inserted into the database.
    /// The generated tree is cryptographically correct - node hashes are derived from their
    /// children using keccak256_combine, making it a proper Merkle tree structure.
    /// 
    /// The tree is built bottom-up from the leaves, creating internal nodes layer by layer
    /// until reaching the root. The number of internal nodes is automatically determined
    /// by the number of leaves.
    /// 
    /// # Arguments
    /// * `seed` - Seed for deterministic random generation of leaf values
    /// * `network_id` - Network ID for the tree
    /// * `num_leaves` - Number of leaf nodes to generate (minimum 2)
    /// 
    /// # Returns
    /// A vector of (SmtKey, SmtValue) pairs representing:
    /// - Leaf nodes with random hash values
    /// - Internal nodes computed as keccak256(left || right)
    /// - A root node marked with SmtKeyType::Root
    /// 
    /// # Note
    /// If num_leaves < 2, it will be set to 2 to ensure a valid tree structure.
    pub fn generate_smt_for_test(
        seed: u64,
        network_id: NetworkId,
        num_leaves: usize,
    ) -> Vec<(SmtKey, SmtValue)> {
        use rand::{Rng, SeedableRng};
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let mut entries = Vec::new();

        // Ensure we have at least 2 leaves for a valid tree
        let num_leaves = num_leaves.max(2);
        
        // 1. Generate leaf nodes (bottom layer)
        let mut current_layer: Vec<Digest> = Vec::new();
        for _ in 0..num_leaves {
            let leaf_hash = Digest(rng.random::<[u8; 32]>());
            current_layer.push(leaf_hash);
            
            let leaf_key = SmtKey {
                network_id: network_id.to_u32(),
                key_type: SmtKeyType::Node(leaf_hash),
            };
            // For leaves, the value is the leaf hash itself
            entries.push((leaf_key, SmtValue::Leaf(leaf_hash)));
        }

        // 2. Build internal nodes layer by layer (bottom-up)
        // Continue until we have a single root node
        while current_layer.len() > 1 {
            let mut next_layer = Vec::new();
            
            // Pair up nodes in the current layer
            for i in (0..current_layer.len()).step_by(2) {
                let left = current_layer[i];
                // If odd number of nodes, pair the last one with empty hash (sparse tree property)
                let right = if i + 1 < current_layer.len() {
                    current_layer[i + 1]
                } else {
                    Digest::ZERO
                };
                
                // Compute the hash for this internal node from its children
                let node_hash = keccak256_combine([left.as_ref(), right.as_ref()]);
                
                let node_key = SmtKey {
                    network_id: network_id.to_u32(),
                    key_type: SmtKeyType::Node(node_hash),
                };
                
                entries.push((node_key, SmtValue::Node(left, right)));
                next_layer.push(node_hash);
            }
            
            current_layer = next_layer;
        }

        // 3. The last remaining node becomes the root
        // We need to add it with SmtKeyType::Root
        if let Some(&root_hash) = current_layer.first() {
            // Find the root node's value in entries and get its children
            if let Some(root_entry) = entries.iter().find(|(key, _)| {
                if let SmtKeyType::Node(hash) = key.key_type {
                    hash == root_hash
                } else {
                    false
                }
            }) {
                if let SmtValue::Node(left, right) = root_entry.1 {
                    // Add the root with SmtKeyType::Root
                    let root_key = SmtKey {
                        network_id: network_id.to_u32(),
                        key_type: SmtKeyType::Root,
                    };
                    entries.push((root_key, SmtValue::Node(left, right)));
                }
            }
        }

        entries
    }

    /// Generate a local exit tree for testing.
    /// 
    /// Returns a vector of (Key, Value) pairs for the local exit tree column family.
    /// The tree includes:
    /// - A leaf count entry
    /// - The specified number of leaves with random hashes
    /// - Exactly 32 frontier nodes (one per tree layer) computed properly from the leaf structure
    /// 
    /// The frontier is computed using the incremental Merkle tree algorithm, representing
    /// the rightmost path needed to compute the tree root when leaves are added incrementally.
    /// 
    /// # Arguments
    /// * `seed` - Seed for deterministic random generation of leaf hashes
    /// * `network_id` - Network ID for the tree
    /// * `num_leaves` - Number of leaves to generate
    /// 
    /// # Note
    /// The function uses LocalExitTreeData to properly compute frontier hashes from leaves.
    pub fn generate_let_for_test(
        seed: u64,
        network_id: NetworkId,
        num_leaves: u32,
    ) -> Vec<(crate::columns::local_exit_tree_per_network::Key, crate::columns::local_exit_tree_per_network::Value)> {
        use rand::{Rng, SeedableRng};
        use crate::columns::local_exit_tree_per_network::{Key, KeyType, Value};
        use pessimistic_proof::local_exit_tree::{data::LocalExitTreeData, LocalExitTree};
        
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let mut entries = Vec::new();

        // 1. Generate random leaf hashes
        let leaf_hashes: Vec<Digest> = (0..num_leaves)
            .map(|_| Digest(rng.random::<[u8; 32]>()))
            .collect();

        // 2. Build the Local Exit Tree Data structure to compute proper frontier
        let let_data = LocalExitTreeData::<32>::from_leaves(leaf_hashes.iter().copied())
            .expect("Failed to create LocalExitTreeData");
        
        // 3. Convert to LocalExitTree to get the proper frontier
        let local_exit_tree: LocalExitTree<32> = (&let_data)
            .try_into()
            .expect("Failed to convert to LocalExitTree");

        // 4. Generate leaf count entry
        let leaves_key = Key {
            network_id: network_id.to_u32(),
            key_type: KeyType::LeafCount,
        };
        entries.push((leaves_key, Value::LeafCount(num_leaves)));

        // 5. Generate leaf entries
        for (leaf_idx, leaf_hash) in leaf_hashes.iter().enumerate() {
            let leaf_key = Key {
                network_id: network_id.to_u32(),
                key_type: KeyType::Leaf(leaf_idx as u32),
            };
            entries.push((leaf_key, Value::Leaf(*leaf_hash.as_bytes())));
        }

        // 6. Generate frontier nodes (always 32 layers for TREE_DEPTH=32)
        for layer in 0..32 {
            let frontier_key = Key {
                network_id: network_id.to_u32(),
                key_type: KeyType::Frontier(layer),
            };
            let frontier_hash = *local_exit_tree.frontier()[layer as usize];
            entries.push((frontier_key, Value::Frontier(frontier_hash)));
        }

        entries
    }
}
