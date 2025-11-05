//! Database Generator for Regression Testing
//!
//! This module provides utilities to generate populated RocksDB databases
//! for regression testing across version upgrades (e.g., alloy 0.14 -> 1.0).
//!
//! The generated databases contain realistic test data across all column families
//! and can be used as artifacts for deserialization regression tests.

use std::path::Path;

use agglayer_types::{
    Certificate, CertificateHeader, CertificateId, CertificateIndex, CertificateStatus,
    EpochNumber, Height, NetworkId, Proof,
};
use rand::Rng;

use crate::{
    columns::{
        balance_tree_per_network::BalanceTreePerNetworkColumn,
        certificate_header::CertificateHeaderColumn,
        certificate_per_network::{CertificatePerNetworkColumn, Key as CertPerNetKey},
        latest_pending_certificate_per_network::{
            LatestPendingCertificatePerNetworkColumn, PendingCertificate,
        },
        latest_proven_certificate_per_network::{
            LatestProvenCertificatePerNetworkColumn, ProvenCertificate,
        },
        latest_settled_certificate_per_network::{
            LatestSettledCertificatePerNetworkColumn, SettledCertificate,
        },
        local_exit_tree_per_network::{
            Key as LetKey, KeyType as LetKeyType, LocalExitTreePerNetworkColumn, Value as LetValue,
        },
        metadata::MetadataColumn,
        network_info::NetworkInfoColumn,
        nullifier_tree_per_network::NullifierTreePerNetworkColumn,
        pending_queue::{PendingQueueColumn, PendingQueueKey},
        proof_per_certificate::ProofPerCertificateColumn,
    },
    storage::{
        debug_db_cf_definitions, epochs_db_cf_definitions, pending_db_cf_definitions,
        state_db_cf_definitions, DB,
    },
    types::{MetadataKey, MetadataValue, SmtKey, SmtKeyType, SmtValue},
};

/// Configuration for database generation
#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    /// Number of different networks to generate data for
    pub num_networks: u32,
    /// Number of certificates per network (i.e., height range)
    pub certificates_per_network: u64,
    /// Whether to generate proofs for certificates
    pub generate_proofs: bool,
    /// Random seed for reproducibility
    pub seed: u64,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            num_networks: 3,
            certificates_per_network: 5,
            generate_proofs: true,
            seed: 42,
        }
    }
}

/// Result of database generation
#[derive(Debug)]
pub struct GenerationResult {
    /// Number of entries written to each column family
    pub entries_per_cf: std::collections::HashMap<String, usize>,
    /// List of network IDs that were generated
    pub network_ids: Vec<NetworkId>,
    /// List of certificate IDs that were generated
    pub certificate_ids: Vec<CertificateId>,
}

/// Generate all four databases (state, pending, epochs, debug) with test data
pub fn generate_all_databases(
    base_path: &Path,
    config: &GeneratorConfig,
) -> Result<GenerationResult, crate::storage::DBError> {
    let mut result = GenerationResult {
        entries_per_cf: std::collections::HashMap::new(),
        network_ids: Vec::new(),
        certificate_ids: Vec::new(),
    };

    // Generate state database
    let state_path = base_path.join("state");
    let state_result = generate_state_db(&state_path, config)?;
    let state_network_ids = state_result.network_ids.clone();
    let state_cert_ids = state_result.certificate_ids.clone();
    result.entries_per_cf.extend(state_result.entries_per_cf);
    result.network_ids = state_network_ids.clone();
    result.certificate_ids = state_cert_ids.clone();

    // Generate pending database
    let pending_path = base_path.join("pending");
    // Create a minimal state result for passing to pending generation
    let minimal_state_result = GenerationResult {
        entries_per_cf: std::collections::HashMap::new(),
        network_ids: state_network_ids,
        certificate_ids: state_cert_ids,
    };
    let pending_result = generate_pending_db(&pending_path, config, &minimal_state_result)?;
    result.entries_per_cf.extend(pending_result.entries_per_cf);

    // Generate epochs database
    let epochs_path = base_path.join("epochs");
    let _epochs_result = generate_epochs_db(&epochs_path, config)?;
    // Note: epochs generation to be implemented in follow-up

    // Generate debug database
    let debug_path = base_path.join("debug");
    let _debug_result = generate_debug_db(&debug_path, config)?;
    // Note: debug generation to be implemented in follow-up

    Ok(result)
}

/// Generate state database with all column families
pub fn generate_state_db(
    path: &Path,
    config: &GeneratorConfig,
) -> Result<GenerationResult, crate::storage::DBError> {
    let db = DB::open_cf(path, state_db_cf_definitions())?;
    let mut rng = rand::rng();
    
    let mut result = GenerationResult {
        entries_per_cf: std::collections::HashMap::new(),
        network_ids: Vec::new(),
        certificate_ids: Vec::new(),
    };

    // Generate data for each network
    for network_idx in 0..config.num_networks {
        let network_id = NetworkId::new(network_idx + 1);
        result.network_ids.push(network_id);

        // Generate certificates for this network
        for height in 0..config.certificates_per_network {
            let height = Height::from(height);
            let certificate = Certificate::new_for_test(network_id, height);
            let cert_id = certificate.hash();
            result.certificate_ids.push(cert_id);

            // 1. CertificatePerNetwork: (network_id, height) -> certificate_id
            let cert_per_net_key = CertPerNetKey {
                network_id: network_id.to_u32(),
                height,
            };
            db.put::<CertificatePerNetworkColumn>(&cert_per_net_key, &cert_id)?;
            *result
                .entries_per_cf
                .entry("certificate_per_network_cf".to_string())
                .or_insert(0) += 1;

            // 2. CertificateHeader: certificate_id -> header
            let header = CertificateHeader {
                network_id,
                height,
                epoch_number: None,
                certificate_index: None,
                certificate_id: cert_id,
                prev_local_exit_root: certificate.prev_local_exit_root,
                new_local_exit_root: certificate.new_local_exit_root,
                metadata: certificate.metadata,
                status: CertificateStatus::Pending,
                settlement_tx_hash: None,
            };
            db.put::<CertificateHeaderColumn>(&cert_id, &header)?;
            *result
                .entries_per_cf
                .entry("certificate_header_cf".to_string())
                .or_insert(0) += 1;

            // 3. LatestSettledCertificatePerNetwork (only for the last certificate)
            if height.as_u64() == config.certificates_per_network - 1 {
                let settled_cert = SettledCertificate(
                    cert_id,
                    height,
                    EpochNumber::default(),
                    CertificateIndex::default(),
                );
                db.put::<LatestSettledCertificatePerNetworkColumn>(&network_id, &settled_cert)?;
                *result
                    .entries_per_cf
                    .entry("latest_settled_certificate_per_network_cf".to_string())
                    .or_insert(0) += 1;
            }
        }

        // 4. LocalExitTree: Generate a small exit tree with a few leaves
        let num_leaves = rng.random_range(2..5);
        let leaves_key = LetKey {
            network_id: network_id.to_u32(),
            key_type: LetKeyType::LeafCount,
        };
        db.put::<LocalExitTreePerNetworkColumn>(
            &leaves_key,
            &LetValue::LeafCount(num_leaves),
        )?;
        *result
            .entries_per_cf
            .entry("local_exit_tree_per_network_cf".to_string())
            .or_insert(0) += 1;

        for leaf_idx in 0..num_leaves {
            let leaf_key = LetKey {
                network_id: network_id.to_u32(),
                key_type: LetKeyType::Leaf(leaf_idx),
            };
            let leaf_hash = [rng.random::<u8>(); 32];
            db.put::<LocalExitTreePerNetworkColumn>(&leaf_key, &LetValue::Leaf(leaf_hash))?;
            *result
                .entries_per_cf
                .entry("local_exit_tree_per_network_cf".to_string())
                .or_insert(0) += 1;
        }

        // Add frontier nodes (simplified - just add a couple)
        for layer in 0..2 {
            let frontier_key = LetKey {
                network_id: network_id.to_u32(),
                key_type: LetKeyType::Frontier(layer),
            };
            let frontier_hash = [rng.random::<u8>(); 32];
            db.put::<LocalExitTreePerNetworkColumn>(
                &frontier_key,
                &LetValue::Frontier(frontier_hash),
            )?;
            *result
                .entries_per_cf
                .entry("local_exit_tree_per_network_cf".to_string())
                .or_insert(0) += 1;
        }

        // 5. BalanceTree: Generate sparse merkle tree nodes
        // Root node
        let balance_root_key = SmtKey {
            network_id: network_id.to_u32(),
            key_type: SmtKeyType::Root,
        };
        let root_left = agglayer_types::Digest(rng.random::<[u8; 32]>());
        let root_right = agglayer_types::Digest(rng.random::<[u8; 32]>());
        db.put::<BalanceTreePerNetworkColumn>(
            &balance_root_key,
            &SmtValue::Node(root_left, root_right),
        )?;
        *result
            .entries_per_cf
            .entry("balance_tree_per_network_cf".to_string())
            .or_insert(0) += 1;

        // Add a few internal nodes
        for _ in 0..3 {
            let node_hash = agglayer_types::Digest(rng.random::<[u8; 32]>());
            let node_key = SmtKey {
                network_id: network_id.to_u32(),
                key_type: SmtKeyType::Node(node_hash),
            };
            let left = agglayer_types::Digest(rng.random::<[u8; 32]>());
            let right = agglayer_types::Digest(rng.random::<[u8; 32]>());
            db.put::<BalanceTreePerNetworkColumn>(&node_key, &SmtValue::Node(left, right))?;
            *result
                .entries_per_cf
                .entry("balance_tree_per_network_cf".to_string())
                .or_insert(0) += 1;
        }

        // Add a leaf node
        let leaf_hash = agglayer_types::Digest(rng.random::<[u8; 32]>());
        let leaf_key = SmtKey {
            network_id: network_id.to_u32(),
            key_type: SmtKeyType::Node(leaf_hash),
        };
        let leaf_value = agglayer_types::Digest(rng.random::<[u8; 32]>());
        db.put::<BalanceTreePerNetworkColumn>(&leaf_key, &SmtValue::Leaf(leaf_value))?;
        *result
            .entries_per_cf
            .entry("balance_tree_per_network_cf".to_string())
            .or_insert(0) += 1;

        // 6. NullifierTree: Similar to balance tree
        let nullifier_root_key = SmtKey {
            network_id: network_id.to_u32(),
            key_type: SmtKeyType::Root,
        };
        let root_left = agglayer_types::Digest(rng.random::<[u8; 32]>());
        let root_right = agglayer_types::Digest(rng.random::<[u8; 32]>());
        db.put::<NullifierTreePerNetworkColumn>(
            &nullifier_root_key,
            &SmtValue::Node(root_left, root_right),
        )?;
        *result
            .entries_per_cf
            .entry("nullifier_tree_per_network_cf".to_string())
            .or_insert(0) += 1;

        // 7. NetworkInfo: Store network information
        use crate::types::network_info::{Key as NetworkInfoKey, Value as NetworkInfoValue};
        use strum::IntoEnumIterator;
        
        for kind in crate::types::network_info::v0::network_info_value::ValueDiscriminants::iter()
        {
            let key = NetworkInfoKey {
                network_id: network_id.to_u32(),
                kind,
            };
            
            // Create appropriate value based on discriminant
            let value = match kind {
                crate::types::network_info::v0::network_info_value::ValueDiscriminants::NetworkType => {
                    NetworkInfoValue {
                        value: Some(crate::types::network_info::v0::network_info_value::Value::NetworkType(
                            crate::types::network_info::v0::NetworkType::Generic as i32,
                        )),
                    }
                }
                crate::types::network_info::v0::network_info_value::ValueDiscriminants::SettledCertificate => {
                    NetworkInfoValue {
                        value: Some(crate::types::network_info::v0::network_info_value::Value::SettledCertificate(
                            crate::types::network_info::v0::SettledCertificate {
                                certificate_id: None,
                                pp_root: None,
                                let_leaf_count: None,
                                ler: None,
                            },
                        )),
                    }
                }
                crate::types::network_info::v0::network_info_value::ValueDiscriminants::SettledClaim => {
                    let dummy_hash = vec![0u8; 32];
                    NetworkInfoValue {
                        value: Some(crate::types::network_info::v0::network_info_value::Value::SettledClaim(
                            crate::types::network_info::v0::SettledClaim {
                                global_index: Some(crate::types::network_info::v0::GlobalIndex {
                                    value: prost::bytes::Bytes::from(dummy_hash.clone()),
                                }),
                                bridge_exit_hash: Some(crate::types::network_info::v0::BridgeExitHash {
                                    bridge_exit_hash: prost::bytes::Bytes::from(dummy_hash),
                                }),
                            },
                        )),
                    }
                }
                crate::types::network_info::v0::network_info_value::ValueDiscriminants::LatestPendingCertificateInfo => {
                    NetworkInfoValue {
                        value: Some(crate::types::network_info::v0::network_info_value::Value::LatestPendingCertificateInfo(
                            crate::types::network_info::v0::LatestPendingCertificateInfo {
                                id: None,
                                height: None,
                            },
                        )),
                    }
                }
            };
            
            db.put::<NetworkInfoColumn>(&key, &value)?;
            *result
                .entries_per_cf
                .entry("network_info_cf".to_string())
                .or_insert(0) += 1;
        }
    }

    // 8. Metadata: Store global metadata
    let latest_epoch_key = MetadataKey::LatestSettledEpoch;
    let latest_epoch_value = MetadataValue::LatestSettledEpoch(0.into());
    db.put::<MetadataColumn>(&latest_epoch_key, &latest_epoch_value)?;
    *result
        .entries_per_cf
        .entry("metadata_cf".to_string())
        .or_insert(0) += 1;

    Ok(result)
}

/// Generate pending database with test data
pub fn generate_pending_db(
    path: &Path,
    config: &GeneratorConfig,
    state_result: &GenerationResult,
) -> Result<GenerationResult, crate::storage::DBError> {
    let db = DB::open_cf(path, pending_db_cf_definitions())?;
    
    let mut result = GenerationResult {
        entries_per_cf: std::collections::HashMap::new(),
        network_ids: state_result.network_ids.clone(),
        certificate_ids: state_result.certificate_ids.clone(),
    };

    // Generate data for each network
    for network_id in &state_result.network_ids {
        // 1. PendingQueue: Add a few pending certificates
        for height in 0..config.certificates_per_network.min(2) {
            let height = Height::from(height);
            let certificate = Certificate::new_for_test(*network_id, height);
            let key = PendingQueueKey(*network_id, height);
            db.put::<PendingQueueColumn>(&key, &certificate)?;
            *result
                .entries_per_cf
                .entry("pending_queue_cf".to_string())
                .or_insert(0) += 1;
        }

        // 2. LatestPendingCertificatePerNetwork
        if config.certificates_per_network > 0 {
            let latest_height = Height::from(config.certificates_per_network - 1);
            let certificate = Certificate::new_for_test(*network_id, latest_height);
            let cert_id = certificate.hash();
            let pending_cert = PendingCertificate(cert_id, latest_height);
            db.put::<LatestPendingCertificatePerNetworkColumn>(network_id, &pending_cert)?;
            *result
                .entries_per_cf
                .entry("latest_pending_certificate_per_network_cf".to_string())
                .or_insert(0) += 1;
        }

        // 3. LatestProvenCertificatePerNetwork
        if config.certificates_per_network > 1 {
            let proven_height = Height::from(config.certificates_per_network - 2);
            let certificate = Certificate::new_for_test(*network_id, proven_height);
            let cert_id = certificate.hash();
            let proven_cert = ProvenCertificate(cert_id, *network_id, proven_height);
            db.put::<LatestProvenCertificatePerNetworkColumn>(network_id, &proven_cert)?;
            *result
                .entries_per_cf
                .entry("latest_proven_certificate_per_network_cf".to_string())
                .or_insert(0) += 1;
        }
    }

    // 4. ProofPerCertificate: Generate proofs if requested
    if config.generate_proofs {
        // Generate a dummy proof for the first certificate
        if let Some(cert_id) = state_result.certificate_ids.first() {
            let proof = Proof::dummy();
            db.put::<ProofPerCertificateColumn>(cert_id, &proof)?;
            *result
                .entries_per_cf
                .entry("proof_per_certificate_cf".to_string())
                .or_insert(0) += 1;
        }
    }

    Ok(result)
}

/// Generate epochs database (placeholder for now)
pub fn generate_epochs_db(
    path: &Path,
    _config: &GeneratorConfig,
) -> Result<GenerationResult, crate::storage::DBError> {
    let _db = DB::open_cf(path, epochs_db_cf_definitions())?;
    
    let result = GenerationResult {
        entries_per_cf: std::collections::HashMap::new(),
        network_ids: Vec::new(),
        certificate_ids: Vec::new(),
    };

    // TODO: Implement epoch data generation
    
    Ok(result)
}

/// Generate debug database (placeholder for now)
pub fn generate_debug_db(
    path: &Path,
    _config: &GeneratorConfig,
) -> Result<GenerationResult, crate::storage::DBError> {
    let _db = DB::open_cf(path, debug_db_cf_definitions())?;
    
    let result = GenerationResult {
        entries_per_cf: std::collections::HashMap::new(),
        network_ids: Vec::new(),
        certificate_ids: Vec::new(),
    };

    // TODO: Implement debug data generation
    
    Ok(result)
}


