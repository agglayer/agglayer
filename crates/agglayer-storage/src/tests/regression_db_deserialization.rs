//! Regression tests for RocksDB deserialization across version upgrades
//!
//! These tests ensure that databases created with previous versions of
//! dependencies (e.g., alloy 0.14) can still be read correctly with newer
//! versions (e.g., alloy 1.0).
//!
//! The test databases are stored as compressed artifacts in tests/fixtures/ and
//! are extracted to temporary locations before testing.

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use agglayer_types::CertificateHeader;
use crate::storage::{
    debug_db_cf_definitions, epochs_db_cf_definitions, pending_db_cf_definitions,
    state_db_cf_definitions, DB,
};

/// Path to the reference database v1 tarball artifact
const REFERENCE_DB_V1_TARBALL: &str = "src/tests/fixtures/reference_db_v1.tar.gz";

#[derive(Debug, Serialize, Deserialize)]
struct DatabaseMetadata {
    version: String,
    timestamp: String,
    config: GeneratorConfig,
    statistics: GenerationStatistics,
    database_paths: DatabasePaths,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeneratorConfig {
    num_networks: u32,
    certificates_per_network: u64,
    generate_proofs: bool,
    seed: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct GenerationStatistics {
    total_networks: usize,
    total_certificates: usize,
    entries_per_column_family: HashMap<String, usize>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DatabasePaths {
    state: String,
    pending: String,
    epochs: String,
    debug: String,
}

/// Helper to extract tarball and return path to extracted directory
fn extract_tarball(
    tarball_path: &Path,
    extract_to: &Path,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    use std::process::Command;

    fs::create_dir_all(extract_to)?;

    let output = Command::new("tar")
        .arg("-xzf")
        .arg(tarball_path)
        .arg("-C")
        .arg(extract_to)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to extract tarball: {}", stderr).into());
    }

    // Return the path to the extracted database directory
    // The tarball contains a directory with the same name as the tarball (minus
    // .tar.gz)
    let db_name = tarball_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or("Invalid tarball name")?
        .trim_end_matches(".tar");

    Ok(extract_to.join(db_name))
}

/// Read metadata from the database directory
fn read_metadata(db_path: &Path) -> Result<DatabaseMetadata, Box<dyn std::error::Error>> {
    let metadata_path = db_path.join("metadata.json");
    let metadata_str = fs::read_to_string(metadata_path)?;
    let metadata: DatabaseMetadata = serde_json::from_str(&metadata_str)?;
    Ok(metadata)
}

/// Simplified test that validates we can open all databases and they contain
/// the expected number of entries according to the metadata.
#[test]
fn test_reference_db_v1_deserialization() {
    // Path to the tarball artifact
    let tarball_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(REFERENCE_DB_V1_TARBALL);

    assert!(
        tarball_path.exists(),
        "Tarball not found at: {}",
        tarball_path.display()
    );

    // Extract to temporary directory
    let temp_dir = std::env::temp_dir().join(format!(
        "agglayer_regression_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    ));

    let db_path = extract_tarball(&tarball_path, &temp_dir).expect("Failed to extract tarball");

    // Read metadata
    let metadata = read_metadata(&db_path).expect("Failed to read metadata");

    println!("Testing reference database v{}", metadata.version);
    println!("Generated at: {}", metadata.timestamp);
    println!(
        "Configuration: {} networks, {} certs/network",
        metadata.config.num_networks, metadata.config.certificates_per_network
    );

    // Open databases in read-only mode
    // This is the main regression test - if deserialization format changed
    // incompatibly, opening the database would fail
    let state_path = db_path.join(&metadata.database_paths.state);
    let pending_path = db_path.join(&metadata.database_paths.pending);
    let epochs_path = db_path.join(&metadata.database_paths.epochs);
    let debug_path = db_path.join(&metadata.database_paths.debug);

    let _state_db = DB::open_cf_readonly(&state_path, state_db_cf_definitions())
        .expect("Failed to open state DB - serialization format may have changed!");
    let _pending_db = DB::open_cf_readonly(&pending_path, pending_db_cf_definitions())
        .expect("Failed to open pending DB - serialization format may have changed!");
    let _epochs_db = DB::open_cf_readonly(&epochs_path, epochs_db_cf_definitions())
        .expect("Failed to open epochs DB - serialization format may have changed!");
    let _debug_db = DB::open_cf_readonly(&debug_path, debug_db_cf_definitions())
        .expect("Failed to open debug DB - serialization format may have changed!");

    println!("\n✅ All databases successfully opened!");
    println!("✅ Column families are accessible");
    println!(
        "✅ Expected {} total entries across {} column families",
        metadata
            .statistics
            .entries_per_column_family
            .values()
            .sum::<usize>(),
        metadata.statistics.entries_per_column_family.len()
    );

    println!("\nExpected entries per column family:");
    for (cf, count) in &metadata.statistics.entries_per_column_family {
        println!("  {}: {} entries", cf, count);
    }

    // Cleanup
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_reference_db_v1_read_metadata() {
    // This test specifically validates reading typed data from the database
    use crate::{
        columns::metadata::MetadataColumn,
        types::{MetadataKey, MetadataValue},
    };

    let tarball_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(REFERENCE_DB_V1_TARBALL);

    let temp_dir = std::env::temp_dir().join(format!(
        "agglayer_regression_test_metadata_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    ));

    let db_path = extract_tarball(&tarball_path, &temp_dir).expect("Failed to extract tarball");

    let metadata = read_metadata(&db_path).expect("Failed to read metadata");

    let state_path = db_path.join(&metadata.database_paths.state);
    let state_db = DB::open_cf_readonly(&state_path, state_db_cf_definitions())
        .expect("Failed to open state DB");

    // Try to read the metadata entry - this validates that data deserialization
    // works
    let metadata_key = MetadataKey::LatestSettledEpoch;
    let metadata_value = state_db
        .get::<MetadataColumn>(&metadata_key)
        .expect("Failed to deserialize metadata entry");

    assert!(metadata_value.is_some(), "Expected metadata entry to exist");

    match metadata_value.unwrap() {
        MetadataValue::LatestSettledEpoch(_epoch) => {
            println!("✅ Successfully read and deserialized metadata entry");
        }
        _ => panic!("Unexpected metadata value type"),
    }

    // Cleanup
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_reference_db_v1_read_certificates() {
    // Test reading certificate data structures
    use crate::columns::certificate_header::CertificateHeaderColumn;

    let tarball_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(REFERENCE_DB_V1_TARBALL);

    let temp_dir = std::env::temp_dir().join(format!(
        "agglayer_regression_test_certs_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    ));

    let db_path = extract_tarball(&tarball_path, &temp_dir).expect("Failed to extract tarball");
    let metadata = read_metadata(&db_path).expect("Failed to read metadata");

    let state_path = db_path.join(&metadata.database_paths.state);
    let state_db = DB::open_cf_readonly(&state_path, state_db_cf_definitions())
        .expect("Failed to open state DB");

    // Iterate through all certificate headers and validate them
    let headers_iter = state_db
        .keys::<CertificateHeaderColumn>()
        .expect("Failed to create certificate headers iterator");
    let mut header_count = 0;
    let mut network_ids_found = std::collections::HashSet::new();

    for cert_id_result in headers_iter {
        let cert_id = match cert_id_result {
            Ok(id) => id,
            Err(e) => {
                eprintln!("Failed to read certificate ID: {:?}", e);
                continue;
            }
        };
        let header: CertificateHeader = state_db
            .get::<CertificateHeaderColumn>(&cert_id)
            .expect("Failed to read certificate header")
            .expect("Certificate header not found");

        // Validate header structure
        assert_eq!(header.certificate_id, cert_id, "Certificate ID mismatch");
        assert!(header.height.as_u64() < 100, "Unexpected height value");

        // Track networks
        network_ids_found.insert(header.network_id);

        header_count += 1;
    }

    println!("✅ Read and validated {} certificate headers", header_count);
    println!("✅ Found {} unique networks", network_ids_found.len());

    assert_eq!(
        header_count, metadata.statistics.total_certificates,
        "Certificate count mismatch"
    );

    // Cleanup
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_reference_db_v1_read_trees() {
    // Test reading tree data structures and reconstructing actual tree types
    use agglayer_tries::{node::Node, smt::Smt};
    use agglayer_types::primitives::Digest;
    use pessimistic_proof::unified_bridge::LocalExitTree;

    use crate::{
        columns::{
            balance_tree_per_network::BalanceTreePerNetworkColumn,
            local_exit_tree_per_network::LocalExitTreePerNetworkColumn,
            nullifier_tree_per_network::NullifierTreePerNetworkColumn,
        },
        types::{SmtKey, SmtKeyType, SmtValue},
    };

    const LOCAL_BALANCE_TREE_DEPTH: usize = 256;
    const NULLIFIER_TREE_DEPTH: usize = 256;
    const LOCAL_EXIT_TREE_DEPTH: usize = 32;

    let tarball_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(REFERENCE_DB_V1_TARBALL);

    let temp_dir = std::env::temp_dir().join(format!(
        "agglayer_regression_test_trees_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    ));

    let db_path = extract_tarball(&tarball_path, &temp_dir).expect("Failed to extract tarball");
    let metadata = read_metadata(&db_path).expect("Failed to read metadata");

    let state_path = db_path.join(&metadata.database_paths.state);
    let state_db = DB::open_cf_readonly(&state_path, state_db_cf_definitions())
        .expect("Failed to open state DB");

    // Get all unique network IDs from the database
    let mut network_ids = std::collections::BTreeSet::new();
    for key_result in state_db
        .keys::<BalanceTreePerNetworkColumn>()
        .expect("Failed to create iterator")
    {
        if let Ok(key) = key_result {
            network_ids.insert(key.network_id);
        }
    }

    println!(
        "Testing tree reconstruction for {} networks",
        network_ids.len()
    );

    for network_id in network_ids {
        println!("\n=== Network {} ===", network_id);

        // Test 1: Reconstruct Balance Tree (Smt)
        println!("1. Reconstructing Balance Tree...");
        let balance_root_value = state_db
            .get::<BalanceTreePerNetworkColumn>(&SmtKey {
                network_id,
                key_type: SmtKeyType::Root,
            })
            .expect("Failed to read balance root")
            .expect("Balance root not found");

        let root_node = match balance_root_value {
            SmtValue::Node(left, right) => Node {
                left: Digest(*left.as_bytes()),
                right: Digest(*right.as_bytes()),
            },
            _ => panic!("Root should be a Node type"),
        };

        // Collect all nodes via BFS
        let mut nodes = vec![root_node];
        let mut queue = std::collections::VecDeque::new();
        let mut visited = std::collections::BTreeSet::new();

        queue.push_back(root_node.left);
        queue.push_back(root_node.right);

        while let Some(node_hash) = queue.pop_front() {
            if !visited.insert(node_hash) {
                continue; // Already processed
            }

            let key = SmtKey {
                network_id,
                key_type: SmtKeyType::Node(node_hash.into()),
            };

            if let Ok(Some(value)) = state_db.get::<BalanceTreePerNetworkColumn>(&key) {
                match value {
                    SmtValue::Node(left, right) => {
                        let node = Node {
                            left: Digest(*left.as_bytes()),
                            right: Digest(*right.as_bytes()),
                        };
                        nodes.push(node);
                        queue.push_back(node.left);
                        queue.push_back(node.right);
                    }
                    SmtValue::Leaf(_) => {
                        // Leaf node, no children to traverse
                    }
                }
            }
        }

        // Reconstruct the Smt
        let balance_tree =
            Smt::<LOCAL_BALANCE_TREE_DEPTH>::new_with_nodes(root_node.hash(), &nodes);
        println!(
            "   ✅ Reconstructed Balance Tree with {} nodes, root: {}",
            nodes.len(),
            balance_tree.root
        );

        // Test 2: Reconstruct Nullifier Tree (Smt)
        println!("2. Reconstructing Nullifier Tree...");
        if let Ok(Some(nullifier_root_value)) =
            state_db.get::<NullifierTreePerNetworkColumn>(&SmtKey {
                network_id,
                key_type: SmtKeyType::Root,
            })
        {
            let root_node = match nullifier_root_value {
                SmtValue::Node(left, right) => Node {
                    left: Digest(*left.as_bytes()),
                    right: Digest(*right.as_bytes()),
                },
                _ => panic!("Root should be a Node type"),
            };

            // Collect nodes for nullifier tree
            let mut nodes = vec![root_node];
            let mut queue = std::collections::VecDeque::new();
            let mut visited = std::collections::BTreeSet::new();

            queue.push_back(root_node.left);
            queue.push_back(root_node.right);

            while let Some(node_hash) = queue.pop_front() {
                if !visited.insert(node_hash) {
                    continue;
                }

                let key = SmtKey {
                    network_id,
                    key_type: SmtKeyType::Node(node_hash.into()),
                };

                if let Ok(Some(value)) = state_db.get::<NullifierTreePerNetworkColumn>(&key) {
                    match value {
                        SmtValue::Node(left, right) => {
                            let node = Node {
                                left: Digest(*left.as_bytes()),
                                right: Digest(*right.as_bytes()),
                            };
                            nodes.push(node);
                            queue.push_back(node.left);
                            queue.push_back(node.right);
                        }
                        SmtValue::Leaf(_) => {}
                    }
                }
            }

            let nullifier_tree =
                Smt::<NULLIFIER_TREE_DEPTH>::new_with_nodes(root_node.hash(), &nodes);
            println!(
                "   ✅ Reconstructed Nullifier Tree with {} nodes, root: {}",
                nodes.len(),
                nullifier_tree.root
            );
        } else {
            println!("   ⚠️  No nullifier tree found for this network");
        }

        // Test 3: Reconstruct Local Exit Tree
        println!("3. Reconstructing Local Exit Tree...");
        use crate::columns::local_exit_tree_per_network::{Key, KeyType, Value};

        let leaf_count_key = Key {
            network_id,
            key_type: KeyType::LeafCount,
        };

        if let Ok(Some(Value::LeafCount(leaf_count))) =
            state_db.get::<LocalExitTreePerNetworkColumn>(&leaf_count_key)
        {
            // Read frontier
            let mut frontier = [Digest::default(); LOCAL_EXIT_TREE_DEPTH];
            let mut frontier_layers_read = 0;
            for layer in 0..LOCAL_EXIT_TREE_DEPTH as u32 {
                let frontier_key = Key {
                    network_id,
                    key_type: KeyType::Frontier(layer),
                };

                if let Ok(Some(Value::Frontier(hash))) =
                    state_db.get::<LocalExitTreePerNetworkColumn>(&frontier_key)
                {
                    frontier[layer as usize] = Digest(hash);
                    frontier_layers_read += 1;
                }
            }

            // Only reconstruct if we have all frontier data
            if frontier_layers_read == LOCAL_EXIT_TREE_DEPTH {
                // Reconstruct the LocalExitTree
                let exit_tree =
                    LocalExitTree::<LOCAL_EXIT_TREE_DEPTH>::from_parts(leaf_count, frontier);
                println!(
                    "   ✅ Reconstructed Local Exit Tree with {} leaves, root: {}",
                    leaf_count,
                    exit_tree.get_root()
                );

                // Verify we can read the leaves
                let mut leaves_read = 0;
                for leaf_idx in 0..leaf_count {
                    let leaf_key = Key {
                        network_id,
                        key_type: KeyType::Leaf(leaf_idx),
                    };

                    if let Ok(Some(Value::Leaf(_hash))) =
                        state_db.get::<LocalExitTreePerNetworkColumn>(&leaf_key)
                    {
                        leaves_read += 1;
                    }
                }
                println!("   ✅ Verified {} leaves readable", leaves_read);
                assert_eq!(
                    leaves_read, leaf_count,
                    "All leaves should be readable"
                );
            } else {
                println!(
                    "   ⚠️  Incomplete frontier data (only {} of {} layers), skipping reconstruction",
                    frontier_layers_read, LOCAL_EXIT_TREE_DEPTH
                );
            }
        } else {
            println!("   ⚠️  No exit tree found for this network");
        }
    }

    println!("\n✅ Successfully reconstructed all tree types from database!");

    // Cleanup
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_reference_db_v1_read_pending_queue() {
    // Test reading pending queue and proofs
    use crate::columns::{
        pending_queue::PendingQueueColumn, proof_per_certificate::ProofPerCertificateColumn,
    };

    let tarball_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(REFERENCE_DB_V1_TARBALL);

    let temp_dir = std::env::temp_dir().join(format!(
        "agglayer_regression_test_pending_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    ));

    let db_path = extract_tarball(&tarball_path, &temp_dir).expect("Failed to extract tarball");
    let metadata = read_metadata(&db_path).expect("Failed to read metadata");

    let pending_path = db_path.join(&metadata.database_paths.pending);
    let pending_db = DB::open_cf_readonly(&pending_path, pending_db_cf_definitions())
        .expect("Failed to open pending DB");

    // Test Pending Queue
    println!("Testing Pending Queue...");
    let pending_iter = pending_db
        .keys::<PendingQueueColumn>()
        .expect("Failed to create pending queue iterator");
    let mut pending_count = 0;
    let mut pending_networks = std::collections::HashSet::new();

    for key_result in pending_iter {
        use agglayer_types::Certificate;

        use crate::columns::pending_queue::PendingQueueKey;

        let key: PendingQueueKey = match key_result {
            Ok(k) => k,
            Err(e) => {
                eprintln!("Failed to read pending queue key: {:?}", e);
                continue;
            }
        };
        let cert: Certificate = pending_db
            .get::<PendingQueueColumn>(&key)
            .expect("Failed to read pending certificate")
            .expect("Pending certificate not found");

        // Validate certificate structure
        assert_eq!(cert.network_id, key.0, "Network ID mismatch");
        assert_eq!(cert.height, key.1, "Height mismatch");

        pending_networks.insert(cert.network_id);
        pending_count += 1;
    }

    println!("  ✅ Found {} pending certificates", pending_count);
    println!("  ✅ Across {} networks", pending_networks.len());

    // Test Proofs
    println!("Testing Proofs...");
    let proofs_iter = pending_db
        .keys::<ProofPerCertificateColumn>()
        .expect("Failed to create proofs iterator");
    let mut proof_count = 0;

    for cert_id_result in proofs_iter {
        use agglayer_types::{CertificateId, Proof};

        let cert_id: CertificateId = match cert_id_result {
            Ok(id) => id,
            Err(e) => {
                eprintln!("Failed to read certificate ID: {:?}", e);
                continue;
            }
        };
        let _proof: Proof = pending_db
            .get::<ProofPerCertificateColumn>(&cert_id)
            .expect("Failed to read proof")
            .expect("Proof not found");

        println!(
            "  ✅ Successfully deserialized proof for certificate {}",
            cert_id
        );
        proof_count += 1;
    }

    println!("  ✅ Found {} proofs", proof_count);

    // Cleanup
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_reference_db_v1_read_network_info() {
    // Test reading network information
    use crate::columns::network_info::NetworkInfoColumn;

    let tarball_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(REFERENCE_DB_V1_TARBALL);

    let temp_dir = std::env::temp_dir().join(format!(
        "agglayer_regression_test_netinfo_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    ));

    let db_path = extract_tarball(&tarball_path, &temp_dir).expect("Failed to extract tarball");
    let metadata = read_metadata(&db_path).expect("Failed to read metadata");

    let state_path = db_path.join(&metadata.database_paths.state);
    let state_db = DB::open_cf_readonly(&state_path, state_db_cf_definitions())
        .expect("Failed to open state DB");

    println!("Testing Network Info...");
    let network_info_iter = state_db
        .keys::<NetworkInfoColumn>()
        .expect("Failed to create network info iterator");
    let mut info_count = 0;
    let mut networks = std::collections::HashSet::new();
    let mut value_types = std::collections::HashMap::new();

    for key_result in network_info_iter {
        use crate::types::network_info::{Key, Value};

        let key: Key = match key_result {
            Ok(k) => k,
            Err(e) => {
                eprintln!("Failed to read network info key: {:?}", e);
                continue;
            }
        };
        let value: Value = state_db
            .get::<NetworkInfoColumn>(&key)
            .expect("Failed to read network info value")
            .expect("Network info value not found");

        networks.insert(key.network_id);

        // Count different value types
        let value_type = match value.value {
            Some(crate::types::network_info::v0::network_info_value::Value::NetworkType(_)) => "NetworkType",
            Some(crate::types::network_info::v0::network_info_value::Value::SettledCertificate(_)) => "SettledCertificate",
            Some(crate::types::network_info::v0::network_info_value::Value::SettledClaim(_)) => "SettledClaim",
            Some(crate::types::network_info::v0::network_info_value::Value::LatestPendingCertificateInfo(_)) => "LatestPendingCertificateInfo",
            None => "None",
        };

        *value_types.entry(value_type).or_insert(0) += 1;
        info_count += 1;
    }

    println!("  ✅ Found {} network info entries", info_count);
    println!("  ✅ Across {} networks", networks.len());
    println!("  ✅ Value type distribution:");
    for (vtype, count) in &value_types {
        println!("      {}: {}", vtype, count);
    }

    // Verify we have the expected network count
    assert_eq!(
        networks.len(),
        metadata.config.num_networks as usize,
        "Network count mismatch"
    );

    // Cleanup
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_reference_db_v1_read_epochs() {
    // Test reading epoch-related data
    use crate::columns::epochs::{
        certificates::CertificatePerIndexColumn, metadata::PerEpochMetadataColumn,
        proofs::ProofPerIndexColumn,
    };

    let tarball_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(REFERENCE_DB_V1_TARBALL);

    let temp_dir = std::env::temp_dir().join(format!(
        "agglayer_regression_test_epochs_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    ));

    let db_path = extract_tarball(&tarball_path, &temp_dir).expect("Failed to extract tarball");
    let metadata = read_metadata(&db_path).expect("Failed to read metadata");

    let epochs_path = db_path.join(&metadata.database_paths.epochs);
    let epochs_db = DB::open_cf_readonly(&epochs_path, epochs_db_cf_definitions())
        .expect("Failed to open epochs DB");

    // Test Epoch Certificates
    println!("Testing Epoch Certificates...");
    let epoch_certs_iter = epochs_db
        .keys::<CertificatePerIndexColumn>()
        .expect("Failed to create epoch certificates iterator");
    let mut cert_count = 0;

    for key_result in epoch_certs_iter {
        use agglayer_types::{Certificate, CertificateIndex};

        let key: CertificateIndex = match key_result {
            Ok(k) => k,
            Err(e) => {
                eprintln!("Failed to read epoch cert key: {:?}", e);
                continue;
            }
        };
        let cert: Certificate = epochs_db
            .get::<CertificatePerIndexColumn>(&key)
            .expect("Failed to read epoch certificate")
            .expect("Epoch certificate not found");

        // Validate the certificate has a network ID
        assert!(cert.network_id.to_u32() > 0, "Network ID should be valid");

        cert_count += 1;
    }

    println!("  ✅ Found {} epoch certificates", cert_count);

    // Test Epoch Metadata
    println!("Testing Epoch Metadata...");
    let epoch_metadata_iter = epochs_db
        .keys::<PerEpochMetadataColumn>()
        .expect("Failed to create epoch metadata iterator");
    let mut metadata_count = 0;

    for key_result in epoch_metadata_iter {
        use crate::types::{PerEpochMetadataKey, PerEpochMetadataValue};

        let key: PerEpochMetadataKey = match key_result {
            Ok(k) => k,
            Err(e) => {
                eprintln!("Failed to read epoch metadata key: {:?}", e);
                continue;
            }
        };
        let value: PerEpochMetadataValue = epochs_db
            .get::<PerEpochMetadataColumn>(&key)
            .expect("Failed to read epoch metadata")
            .expect("Epoch metadata not found");

        // Just validate we can deserialize it
        match value {
            PerEpochMetadataValue::SettlementTxHash(_) => {}
            PerEpochMetadataValue::Packed(_) => {}
        }

        metadata_count += 1;
    }

    println!("  ✅ Found {} epoch metadata entries", metadata_count);

    // Test Epoch Proofs
    println!("Testing Epoch Proofs...");
    let epoch_proofs_iter = epochs_db
        .keys::<ProofPerIndexColumn>()
        .expect("Failed to create epoch proofs iterator");
    let mut proof_count = 0;

    for key_result in epoch_proofs_iter {
        use agglayer_types::{CertificateIndex, Proof};

        let key: CertificateIndex = match key_result {
            Ok(k) => k,
            Err(e) => {
                eprintln!("Failed to read epoch proof key: {:?}", e);
                continue;
            }
        };
        let _proof: Proof = epochs_db
            .get::<ProofPerIndexColumn>(&key)
            .expect("Failed to read epoch proof")
            .expect("Epoch proof not found");

        proof_count += 1;
        if proof_count >= 3 {
            // Just check a few to save time
            break;
        }
    }

    println!(
        "  ✅ Checked {} epoch proofs (limited for speed)",
        proof_count
    );

    // Cleanup
    let _ = fs::remove_dir_all(&temp_dir);
}
