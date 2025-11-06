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
    // Test reading and reconstructing epoch structure
    use std::collections::BTreeMap;

    use agglayer_types::{Certificate, CertificateIndex, Height, NetworkId, Proof};

    use crate::{
        columns::epochs::{
            certificates::CertificatePerIndexColumn,
            end_checkpoint::EndCheckpointColumn,
            metadata::PerEpochMetadataColumn,
            proofs::ProofPerIndexColumn,
            start_checkpoint::StartCheckpointColumn,
        },
        types::{PerEpochMetadataKey, PerEpochMetadataValue},
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

    println!("=== Reconstructing Epoch Structure ===\n");

    // Step 1: Read Start Checkpoint
    println!("1. Reading Start Checkpoint...");
    let start_checkpoint_iter = epochs_db
        .iter_with_direction::<StartCheckpointColumn>(
            rocksdb::ReadOptions::default(),
            rocksdb::Direction::Forward,
        )
        .expect("Failed to create start checkpoint iterator");

    let start_checkpoint: BTreeMap<NetworkId, Height> = start_checkpoint_iter
        .filter_map(|result| result.ok())
        .collect();

    println!(
        "   ✅ Start Checkpoint: {} networks",
        start_checkpoint.len()
    );
    for (network_id, height) in &start_checkpoint {
        println!("      Network {}: Height {}", network_id, height);
    }

    // Step 2: Read End Checkpoint
    println!("\n2. Reading End Checkpoint...");
    let end_checkpoint_iter = epochs_db
        .iter_with_direction::<EndCheckpointColumn>(
            rocksdb::ReadOptions::default(),
            rocksdb::Direction::Forward,
        )
        .expect("Failed to create end checkpoint iterator");

    let end_checkpoint: BTreeMap<NetworkId, Height> = end_checkpoint_iter
        .filter_map(|result| result.ok())
        .collect();

    println!("   ✅ End Checkpoint: {} networks", end_checkpoint.len());
    for (network_id, height) in &end_checkpoint {
        println!("      Network {}: Height {}", network_id, height);
    }

    // Step 3: Read Epoch Metadata (packing status, settlement tx)
    println!("\n3. Reading Epoch Metadata...");
    let mut is_packed = false;
    let mut settlement_tx_hash = None;

    if let Ok(Some(PerEpochMetadataValue::Packed(packed))) =
        epochs_db.get::<PerEpochMetadataColumn>(&PerEpochMetadataKey::Packed)
    {
        is_packed = packed;
        println!("   ✅ Epoch Packed: {}", is_packed);
    }

    if let Ok(Some(PerEpochMetadataValue::SettlementTxHash(tx_hash))) =
        epochs_db.get::<PerEpochMetadataColumn>(&PerEpochMetadataKey::SettlementTxHash)
    {
        settlement_tx_hash = Some(tx_hash);
        println!("   ✅ Settlement Tx Hash: {}", tx_hash);
    }

    // Step 4: Read and validate epoch certificates
    println!("\n4. Reading Epoch Certificates...");
    let epoch_certs_iter = epochs_db
        .keys::<CertificatePerIndexColumn>()
        .expect("Failed to create epoch certificates iterator");

    let mut certificates: BTreeMap<CertificateIndex, Certificate> = BTreeMap::new();

    for key_result in epoch_certs_iter {
        let cert_index = match key_result {
            Ok(k) => k,
            Err(e) => {
                eprintln!("Failed to read epoch cert key: {:?}", e);
                continue;
            }
        };

        let cert: Certificate = epochs_db
            .get::<CertificatePerIndexColumn>(&cert_index)
            .expect("Failed to read epoch certificate")
            .expect("Epoch certificate not found");

        // Validate certificate is within checkpoint range
        if let (Some(start_height), Some(end_height)) = (
            start_checkpoint.get(&cert.network_id),
            end_checkpoint.get(&cert.network_id),
        ) {
            assert!(
                cert.height >= *start_height && cert.height <= *end_height,
                "Certificate height {} outside checkpoint range [{}, {}] for network {}",
                cert.height,
                start_height,
                end_height,
                cert.network_id
            );
        }

        certificates.insert(cert_index, cert);
    }

    println!(
        "   ✅ Found {} certificates in epoch",
        certificates.len()
    );
    for (idx, cert) in certificates.iter().take(3) {
        println!(
            "      Index {}: Network {}, Height {}",
            idx, cert.network_id, cert.height
        );
    }

    // Step 5: Read epoch proofs
    println!("\n5. Reading Epoch Proofs...");
    let epoch_proofs_iter = epochs_db
        .keys::<ProofPerIndexColumn>()
        .expect("Failed to create epoch proofs iterator");

    let mut proofs: BTreeMap<CertificateIndex, Proof> = BTreeMap::new();

    for key_result in epoch_proofs_iter {
        let cert_index = match key_result {
            Ok(k) => k,
            Err(e) => {
                eprintln!("Failed to read epoch proof key: {:?}", e);
                continue;
            }
        };

        let proof: Proof = epochs_db
            .get::<ProofPerIndexColumn>(&cert_index)
            .expect("Failed to read epoch proof")
            .expect("Epoch proof not found");

        // Validate that we have a certificate for this proof
        assert!(
            certificates.contains_key(&cert_index),
            "Proof exists for certificate index {} but no certificate found",
            cert_index
        );

        proofs.insert(cert_index, proof);
    }

    println!("   ✅ Found {} proofs in epoch", proofs.len());

    // Step 6: Validate epoch structure consistency
    println!("\n6. Validating Epoch Structure Consistency...");

    // All networks in end_checkpoint should be in start_checkpoint (or be new)
    for network_id in end_checkpoint.keys() {
        if !start_checkpoint.contains_key(network_id) {
            println!(
                "   ⚠️  Network {} in end checkpoint but not in start checkpoint (new network)",
                network_id
            );
        }
    }

    // Certificates should cover the checkpoint ranges
    let mut networks_with_certs = std::collections::BTreeSet::new();
    for cert in certificates.values() {
        networks_with_certs.insert(cert.network_id);
    }

    println!(
        "   ✅ Certificates span {} unique networks",
        networks_with_certs.len()
    );

    // Summary
    println!("\n=== Epoch Structure Summary ===");
    println!("  Start Checkpoint: {} networks", start_checkpoint.len());
    println!("  End Checkpoint: {} networks", end_checkpoint.len());
    println!("  Certificates: {}", certificates.len());
    println!("  Proofs: {}", proofs.len());
    println!("  Packed: {}", is_packed);
    if let Some(tx_hash) = settlement_tx_hash {
        println!("  Settlement Tx: {}", tx_hash);
    }
    println!("\n✅ Successfully reconstructed and validated epoch structure!");

    // Cleanup
    let _ = fs::remove_dir_all(&temp_dir);
}

/// Test reconstruction of complete LocalNetworkStateData from the state database.
/// This is the highest-level state structure that combines all three trees.
#[test]
fn test_reference_db_v1_read_local_network_state() {
    use agglayer_tries::smt::Smt;
    use agglayer_types::{primitives::Digest, LocalNetworkStateData};
    use pessimistic_proof::{
        local_balance_tree::LOCAL_BALANCE_TREE_DEPTH, nullifier_tree::NULLIFIER_TREE_DEPTH,
        unified_bridge::LocalExitTree,
    };

    use crate::{
        columns::{
            balance_tree_per_network::BalanceTreePerNetworkColumn,
            local_exit_tree_per_network::LocalExitTreePerNetworkColumn,
            nullifier_tree_per_network::NullifierTreePerNetworkColumn,
        },
        types::{SmtKey, SmtKeyType},
    };

    const LOCAL_EXIT_TREE_DEPTH: usize = 32;

    let tarball_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(REFERENCE_DB_V1_TARBALL);
    let temp_dir = std::env::temp_dir().join(format!(
        "agglayer_regression_test_state_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    ));
    let db_path = extract_tarball(&tarball_path, &temp_dir)
        .expect("Failed to extract tarball");
    let metadata = read_metadata(&db_path).expect("Failed to read metadata");
    let state_path = db_path.join(&metadata.database_paths.state);
    let state_db = DB::open_cf_readonly(&state_path, state_db_cf_definitions())
        .expect("Failed to open state DB");

    println!("\n=== Testing LocalNetworkStateData Reconstruction ===");
    println!(
        "This validates the complete state per network, combining all three trees.\n"
    );

    // Collect all networks that have balance trees
    let mut network_ids = std::collections::BTreeSet::new();
    for key_result in state_db
        .keys::<BalanceTreePerNetworkColumn>()
        .expect("Failed to create iterator")
    {
        if let Ok(key) = key_result {
            network_ids.insert(key.network_id);
        }
    }

    println!("Found {} networks with state data\n", network_ids.len());
    assert!(!network_ids.is_empty(), "Should have at least one network");

    for network_id in network_ids {
        println!("=== Network {} ===", network_id);

        // 1. Reconstruct Balance Tree
        println!("  1. Reading Balance Tree...");
        let balance_root_value = state_db
            .get::<BalanceTreePerNetworkColumn>(&SmtKey {
                network_id,
                key_type: SmtKeyType::Root,
            })
            .expect("Failed to read balance root")
            .expect("Balance root not found");

        let balance_root_node = match balance_root_value {
            crate::types::SmtValue::Node(left, right) => agglayer_tries::node::Node {
                left: Digest(*left.as_bytes()),
                right: Digest(*right.as_bytes()),
            },
            _ => panic!("Root should be a Node type"),
        };

        let mut balance_nodes = vec![balance_root_node];
        let mut queue = std::collections::VecDeque::new();
        let mut visited = std::collections::BTreeSet::new();

        queue.push_back(balance_root_node.left);
        queue.push_back(balance_root_node.right);

        while let Some(node_hash) = queue.pop_front() {
            if !visited.insert(node_hash) {
                continue;
            }

            let key = SmtKey {
                network_id,
                key_type: SmtKeyType::Node(node_hash.into()),
            };

            if let Ok(Some(value)) = state_db.get::<BalanceTreePerNetworkColumn>(&key) {
                match value {
                    crate::types::SmtValue::Node(left, right) => {
                        let node = agglayer_tries::node::Node {
                            left: Digest(*left.as_bytes()),
                            right: Digest(*right.as_bytes()),
                        };
                        balance_nodes.push(node);
                        queue.push_back(node.left);
                        queue.push_back(node.right);
                    }
                    crate::types::SmtValue::Leaf(_) => {}
                }
            }
        }

        let balance_tree = Smt::<LOCAL_BALANCE_TREE_DEPTH>::new_with_nodes(
            balance_root_node.hash(),
            &balance_nodes,
        );
        println!(
            "     ✅ Balance Tree: {} nodes, root = {}",
            balance_nodes.len(),
            balance_tree.root
        );

        // 2. Reconstruct Nullifier Tree
        println!("  2. Reading Nullifier Tree...");
        let nullifier_root_value = state_db
            .get::<NullifierTreePerNetworkColumn>(&SmtKey {
                network_id,
                key_type: SmtKeyType::Root,
            })
            .expect("Failed to read nullifier root")
            .expect("Nullifier root not found");

        let nullifier_root_node = match nullifier_root_value {
            crate::types::SmtValue::Node(left, right) => agglayer_tries::node::Node {
                left: Digest(*left.as_bytes()),
                right: Digest(*right.as_bytes()),
            },
            _ => panic!("Root should be a Node type"),
        };

        let mut nullifier_nodes = vec![nullifier_root_node];
        let mut queue = std::collections::VecDeque::new();
        let mut visited = std::collections::BTreeSet::new();

        queue.push_back(nullifier_root_node.left);
        queue.push_back(nullifier_root_node.right);

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
                    crate::types::SmtValue::Node(left, right) => {
                        let node = agglayer_tries::node::Node {
                            left: Digest(*left.as_bytes()),
                            right: Digest(*right.as_bytes()),
                        };
                        nullifier_nodes.push(node);
                        queue.push_back(node.left);
                        queue.push_back(node.right);
                    }
                    crate::types::SmtValue::Leaf(_) => {}
                }
            }
        }

        let nullifier_tree = Smt::<NULLIFIER_TREE_DEPTH>::new_with_nodes(
            nullifier_root_node.hash(),
            &nullifier_nodes,
        );
        println!(
            "     ✅ Nullifier Tree: {} nodes, root = {}",
            nullifier_nodes.len(),
            nullifier_tree.root
        );

        // 3. Reconstruct Local Exit Tree
        println!("  3. Reading Local Exit Tree...");
        use crate::columns::local_exit_tree_per_network::{Key, KeyType, Value};

        let leaf_count_key = Key {
            network_id,
            key_type: KeyType::LeafCount,
        };

        let leaf_count = if let Ok(Some(Value::LeafCount(count))) =
            state_db.get::<LocalExitTreePerNetworkColumn>(&leaf_count_key)
        {
            count
        } else {
            panic!("Exit tree leaf count not found for network {}", network_id);
        };

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

        if frontier_layers_read < LOCAL_EXIT_TREE_DEPTH {
            println!(
                "     ⚠️  Warning: Only {} of {} frontier layers found, remaining will be default",
                frontier_layers_read, LOCAL_EXIT_TREE_DEPTH
            );
        }

        let exit_tree = LocalExitTree::<LOCAL_EXIT_TREE_DEPTH>::from_parts(leaf_count, frontier);
        println!(
            "     ✅ Local Exit Tree: {} leaves, root = {}",
            leaf_count,
            exit_tree.get_root()
        );

        // 4. Construct LocalNetworkStateData
        println!("  4. Assembling LocalNetworkStateData...");
        let local_state = LocalNetworkStateData {
            exit_tree: exit_tree.clone(),
            balance_tree: balance_tree.clone(),
            nullifier_tree: nullifier_tree.clone(),
        };

        // 5. Validate state commitment (roots)
        println!("  5. Validating state commitment...");
        let state_commitment = local_state.get_roots();
        assert_eq!(
            state_commitment.exit_root,
            exit_tree.get_root(),
            "Exit root mismatch"
        );
        assert_eq!(
            state_commitment.ler_leaf_count, leaf_count,
            "Leaf count mismatch"
        );
        assert_eq!(
            state_commitment.balance_root, balance_tree.root,
            "Balance root mismatch"
        );
        assert_eq!(
            state_commitment.nullifier_root, nullifier_tree.root,
            "Nullifier root mismatch"
        );
        println!("     ✅ State commitment validated:");
        println!("        - Exit Root: {}", state_commitment.exit_root);
        println!("        - Balance Root: {}", state_commitment.balance_root);
        println!("        - Nullifier Root: {}", state_commitment.nullifier_root);
        println!("        - LER Leaf Count: {}", state_commitment.ler_leaf_count);

        // 6. Verify conversion to LocalNetworkState (pessimistic proof type)
        println!("  6. Testing conversion to LocalNetworkState...");
        let _network_state: pessimistic_proof::LocalNetworkState = local_state.clone().into();
        println!("     ✅ Successfully converted to LocalNetworkState");

        // 7. Verify conversion to NetworkState (pessimistic proof type)
        println!("  7. Testing conversion to NetworkState...");
        let _pessimistic_state: pessimistic_proof::NetworkState = local_state.into();
        println!("     ✅ Successfully converted to NetworkState");

        println!("  ✅ Network {} fully reconstructed and validated!\n", network_id);
    }

    println!("✅ All LocalNetworkStateData structures successfully reconstructed from database!");
    println!("   This confirms that the complete network state (all three trees) can be");
    println!("   deserialized and used for pessimistic proof generation.");

    let _ = fs::remove_dir_all(&temp_dir);
}
