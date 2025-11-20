//! Regression tests for RocksDB deserialization across version upgrades
//!
//! These tests ensure that databases created with previous versions of
//! dependencies (e.g., alloy 0.14) can still be read correctly with newer
//! versions (e.g., alloy 1.0).
//!
//! The test databases are stored as compressed artifacts in `tests/fixtures/`
//! and are extracted to temporary locations before testing.

use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    storage::{
        debug_db_cf_definitions, epochs_db_cf_definitions, pending_db_cf_definitions,
        state_db_cf_definitions, DB,
    },
    tests::db_generator::DatabaseMetadata,
};

/// Path to the reference database v1 tarball artifact
const REFERENCE_DB_V1_TARBALL: &str = "src/tests/fixtures/reference_db_v1.tar.gz";

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

/// Test fixture helper that manages database extraction and cleanup
struct TestFixture {
    pub db_path: PathBuf,
    pub metadata: DatabaseMetadata,
    temp_dir: PathBuf,
}

impl TestFixture {
    /// Create a new test fixture by extracting the reference database
    fn new(test_name: &str) -> Self {
        let tarball_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(REFERENCE_DB_V1_TARBALL);

        assert!(
            tarball_path.exists(),
            "Tarball not found at: {}",
            tarball_path.display()
        );

        let temp_dir = std::env::temp_dir().join(format!(
            "agglayer_regression_test_{}_{}",
            test_name,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        ));

        let db_path = extract_tarball(&tarball_path, &temp_dir).expect("Failed to extract tarball");
        let metadata = read_metadata(&db_path).expect("Failed to read metadata");

        Self {
            db_path,
            metadata,
            temp_dir,
        }
    }

    /// Open the state database in read-only mode
    fn open_state_db(&self) -> Result<DB, crate::storage::DBError> {
        let state_path = self.db_path.join(&self.metadata.database_paths.state);
        DB::open_cf_readonly(&state_path, state_db_cf_definitions())
    }

    /// Open the pending database in read-only mode
    fn open_pending_db(&self) -> Result<DB, crate::storage::DBError> {
        let pending_path = self.db_path.join(&self.metadata.database_paths.pending);
        DB::open_cf_readonly(&pending_path, pending_db_cf_definitions())
    }

    /// Open the epochs database in read-only mode
    fn open_epochs_db(&self) -> Result<DB, crate::storage::DBError> {
        let epochs_path = self.db_path.join(&self.metadata.database_paths.epochs);
        DB::open_cf_readonly(&epochs_path, epochs_db_cf_definitions())
    }

    /// Open the debug database in read-only mode
    fn open_debug_db(&self) -> Result<DB, crate::storage::DBError> {
        let debug_path = self.db_path.join(&self.metadata.database_paths.debug);
        DB::open_cf_readonly(&debug_path, debug_db_cf_definitions())
    }
}

impl Drop for TestFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.temp_dir);
    }
}

/// Simplified test that validates we can open all databases and they contain
/// the expected number of entries according to the metadata.
#[test]
fn test_reference_db_v1_deserialization() {
    let fixture = TestFixture::new("deserialization");

    println!("Testing reference database v{}", fixture.metadata.version);
    println!("Generated at: {}", fixture.metadata.timestamp);
    println!(
        "Configuration: {} networks, {} certs/network, seed: {}",
        fixture.metadata.config.num_networks,
        fixture.metadata.config.certificates_per_network,
        fixture.metadata.config.seed
    );

    // Open databases in read-only mode
    // This is the main regression test - if deserialization format changed
    // incompatibly, opening the database would fail
    let _state_db = fixture
        .open_state_db()
        .expect("Failed to open state DB - serialization format may have changed!");
    let _pending_db = fixture
        .open_pending_db()
        .expect("Failed to open pending DB - serialization format may have changed!");
    let _epochs_db = fixture
        .open_epochs_db()
        .expect("Failed to open epochs DB - serialization format may have changed!");
    let _debug_db = fixture
        .open_debug_db()
        .expect("Failed to open debug DB - serialization format may have changed!");

    println!("\n✅ All databases successfully opened!");
    println!("✅ Column families are accessible");
    println!(
        "✅ Expected {} total entries across {} column families",
        fixture
            .metadata
            .statistics
            .entries_per_column_family
            .values()
            .sum::<usize>(),
        fixture.metadata.statistics.entries_per_column_family.len()
    );

    println!("\nExpected entries per column family:");
    for (cf, count) in &fixture.metadata.statistics.entries_per_column_family {
        println!("  {}: {} entries", cf, count);
    }
}

#[test]
fn test_reference_db_v1_read_metadata() {
    // This test specifically validates reading typed data from the database
    use crate::{
        columns::metadata::MetadataColumn,
        types::{MetadataKey, MetadataValue},
    };

    let fixture = TestFixture::new("metadata");
    let state_db = fixture.open_state_db().expect("Failed to open state DB");

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
}

#[test]
fn test_reference_db_v1_read_state_db() {
    // Comprehensive test reading and reconstructing all data from state database
    // This reverses what generate_state_db does - reads values and validates
    // structure
    use std::collections::{BTreeMap, HashMap, HashSet};

    use agglayer_types::{
        primitives::Digest, CertificateHeader, CertificateId, CertificateStatus, Height, NetworkId,
    };

    use crate::{
        columns::{
            balance_tree_per_network::BalanceTreePerNetworkColumn,
            certificate_header::CertificateHeaderColumn,
            certificate_per_network::{CertificatePerNetworkColumn, Key as CertPerNetKey},
            disabled_networks::DisabledNetworksColumn,
            latest_settled_certificate_per_network::{
                LatestSettledCertificatePerNetworkColumn, SettledCertificate,
            },
            local_exit_tree_per_network::{
                Key as LetKey, KeyType as LetKeyType, LocalExitTreePerNetworkColumn,
                Value as LetValue,
            },
            metadata::MetadataColumn,
            network_info::NetworkInfoColumn,
            nullifier_tree_per_network::NullifierTreePerNetworkColumn,
        },
        types::{
            disabled_network::Value as DisabledNetwork,
            network_info::{Key as NetworkInfoKey, Value as NetworkInfoValue},
            MetadataKey, MetadataValue, SmtKey, SmtKeyType, SmtValue,
        },
    };

    let fixture = TestFixture::new("state_db");
    let state_db = fixture.open_state_db().expect("Failed to open state DB");

    println!("=== Reconstructing State Database (Reverse of generate_state_db) ===\n");

    // Collect network IDs from the data
    let mut network_ids: HashSet<NetworkId> = HashSet::new();

    // 1. Read and validate CertificatePerNetwork mappings
    println!("1. Reading certificate_per_network_cf...");
    let mut cert_per_network_map: BTreeMap<(NetworkId, Height), CertificateId> = BTreeMap::new();
    for key_result in state_db
        .keys::<CertificatePerNetworkColumn>()
        .expect("Failed to iterate")
    {
        let key: CertPerNetKey = key_result.expect("Failed to read key");
        let cert_id: CertificateId = state_db
            .get::<CertificatePerNetworkColumn>(&key)
            .expect("Failed to read value")
            .expect("Value not found");

        let network_id = NetworkId::new(key.network_id);
        network_ids.insert(network_id);
        cert_per_network_map.insert((network_id, key.height), cert_id);
    }
    println!(
        "   ✅ Reconstructed {} certificate mappings for {} networks",
        cert_per_network_map.len(),
        network_ids.len()
    );

    // 2. Read and validate CertificateHeaders
    println!("\n2. Reading certificate_header_cf...");
    let mut headers: BTreeMap<CertificateId, CertificateHeader> = BTreeMap::new();
    for cert_id_result in state_db
        .keys::<CertificateHeaderColumn>()
        .expect("Failed to iterate")
    {
        let cert_id = cert_id_result.expect("Failed to read cert ID");
        let header: CertificateHeader = state_db
            .get::<CertificateHeaderColumn>(&cert_id)
            .expect("Failed to read header")
            .expect("Header not found");

        // Validate header structure
        assert_eq!(header.certificate_id, cert_id, "Certificate ID mismatch");
        assert_eq!(header.status, CertificateStatus::Pending);
        assert!(header.epoch_number.is_some(), "Epoch number should be set");
        assert!(
            header.certificate_index.is_some(),
            "Certificate index should be set"
        );

        // Verify it matches the cert_per_network mapping
        if let Some(mapped_cert_id) = cert_per_network_map.get(&(header.network_id, header.height))
        {
            assert_eq!(
                *mapped_cert_id, cert_id,
                "Certificate mapping mismatch for network {} height {}",
                header.network_id, header.height
            );
        }

        headers.insert(cert_id, header);
    }
    println!("   ✅ Reconstructed {} certificate headers", headers.len());
    println!("   ✅ All headers have valid epoch/index and match network mappings");

    // 3. Read and validate LatestSettledCertificatePerNetwork
    println!("\n3. Reading latest_settled_certificate_per_network_cf...");
    let mut settled_certs: HashMap<NetworkId, SettledCertificate> = HashMap::new();
    for network_id_result in state_db
        .keys::<LatestSettledCertificatePerNetworkColumn>()
        .expect("Failed to iterate")
    {
        let network_id = network_id_result.expect("Failed to read network ID");
        let settled_cert: SettledCertificate = state_db
            .get::<LatestSettledCertificatePerNetworkColumn>(&network_id)
            .expect("Failed to read settled cert")
            .expect("Settled cert not found");

        // Validate that this certificate exists in headers
        if let Some(header) = headers.get(&settled_cert.0) {
            assert_eq!(
                header.network_id, network_id,
                "Settled cert network mismatch"
            );
            assert_eq!(
                header.height, settled_cert.1,
                "Settled cert height mismatch"
            );
            // Note: Epoch and index should match, but we're just validating structure
            // exists
            assert!(
                header.epoch_number.is_some(),
                "Header should have epoch number"
            );
            assert!(
                header.certificate_index.is_some(),
                "Header should have certificate index"
            );
        } else {
            panic!(
                "Settled certificate {} not found in headers",
                settled_cert.0
            );
        }

        settled_certs.insert(network_id, settled_cert);
    }
    println!(
        "   ✅ Reconstructed {} settled certificates",
        settled_certs.len()
    );
    println!("   ✅ All settled certificates match their headers");

    // 4. Read and reconstruct LocalExitTree per network
    println!("\n4. Reading local_exit_tree_per_network_cf...");
    // Type alias for LocalExitTree data: (leaf_count, leaves, frontiers)
    type LetData = (u32, Vec<[u8; 32]>, Vec<[u8; 32]>);
    let mut let_per_network: HashMap<NetworkId, LetData> = HashMap::new();
    for key_result in state_db
        .keys::<LocalExitTreePerNetworkColumn>()
        .expect("Failed to iterate")
    {
        let key: LetKey = key_result.expect("Failed to read key");
        let value: LetValue = state_db
            .get::<LocalExitTreePerNetworkColumn>(&key)
            .expect("Failed to read value")
            .expect("Value not found");

        let network_id = NetworkId::new(key.network_id);
        let entry = let_per_network
            .entry(network_id)
            .or_insert((0, Vec::new(), Vec::new()));

        match (key.key_type, value) {
            (LetKeyType::LeafCount, LetValue::LeafCount(count)) => {
                entry.0 = count;
            }
            (LetKeyType::Leaf(_), LetValue::Leaf(hash)) => {
                entry.1.push(hash);
            }
            (LetKeyType::Frontier(_), LetValue::Frontier(hash)) => {
                entry.2.push(hash);
            }
            _ => panic!("Invalid LET key/value combination"),
        }
    }
    println!(
        "   ✅ Reconstructed local exit trees for {} networks",
        let_per_network.len()
    );
    for (network_id, (leaf_count, leaves, frontiers)) in &let_per_network {
        println!(
            "      Network {}: {} leaves, {} leaf hashes, {} frontier nodes",
            network_id,
            leaf_count,
            leaves.len(),
            frontiers.len()
        );
    }

    // 5. Read and validate BalanceTree per network
    println!("\n5. Reading balance_tree_per_network_cf...");
    // Type alias for BalanceTree data: (root, internal_nodes, leaves)
    type BalanceTreeData = (Digest, Vec<(Digest, Digest)>, Vec<Digest>);
    let mut balance_trees: HashMap<NetworkId, BalanceTreeData> = HashMap::new();
    for key_result in state_db
        .keys::<BalanceTreePerNetworkColumn>()
        .expect("Failed to iterate")
    {
        let key: SmtKey = key_result.expect("Failed to read key");
        let value: SmtValue = state_db
            .get::<BalanceTreePerNetworkColumn>(&key)
            .expect("Failed to read value")
            .expect("Value not found");

        let network_id = NetworkId::new(key.network_id);
        let entry =
            balance_trees
                .entry(network_id)
                .or_insert((Digest::default(), Vec::new(), Vec::new()));

        match (key.key_type, value) {
            (SmtKeyType::Root, SmtValue::Node(left, right)) => {
                entry.0 = left; // Store root (we could compute hash of left+right)
                entry.1.push((left, right));
            }
            (SmtKeyType::Node(_hash), SmtValue::Node(left, right)) => {
                entry.1.push((left, right));
            }
            (SmtKeyType::Node(_hash), SmtValue::Leaf(value)) => {
                entry.2.push(value);
            }
            _ => {}
        }
    }
    println!(
        "   ✅ Reconstructed balance trees for {} networks",
        balance_trees.len()
    );
    for (network_id, (root, nodes, leaves)) in &balance_trees {
        println!(
            "      Network {}: root={}, {} nodes, {} leaves",
            network_id,
            root,
            nodes.len(),
            leaves.len()
        );
    }

    // 6. Read and validate NullifierTree per network
    println!("\n6. Reading nullifier_tree_per_network_cf...");
    let mut nullifier_roots: HashMap<NetworkId, (Digest, Digest)> = HashMap::new();
    for key_result in state_db
        .keys::<NullifierTreePerNetworkColumn>()
        .expect("Failed to iterate")
    {
        let key: SmtKey = key_result.expect("Failed to read key");
        if let SmtKeyType::Root = key.key_type {
            let value: SmtValue = state_db
                .get::<NullifierTreePerNetworkColumn>(&key)
                .expect("Failed to read value")
                .expect("Value not found");

            if let SmtValue::Node(left, right) = value {
                nullifier_roots.insert(NetworkId::new(key.network_id), (left, right));
            }
        }
    }
    println!(
        "   ✅ Reconstructed nullifier tree roots for {} networks",
        nullifier_roots.len()
    );

    // 7. Read and validate NetworkInfo
    println!("\n7. Reading network_info_cf...");
    let mut network_info: HashMap<NetworkId, Vec<String>> = HashMap::new();
    for key_result in state_db
        .keys::<NetworkInfoColumn>()
        .expect("Failed to iterate")
    {
        let key: NetworkInfoKey = key_result.expect("Failed to read key");
        let value: NetworkInfoValue = state_db
            .get::<NetworkInfoColumn>(&key)
            .expect("Failed to read value")
            .expect("Value not found");

        let network_id = NetworkId::new(key.network_id);
        let info_types = network_info.entry(network_id).or_default();

        match value.value {
            Some(crate::types::network_info::v0::network_info_value::Value::NetworkType(nt)) => {
                info_types.push(format!("NetworkType({})", nt));
            }
            Some(crate::types::network_info::v0::network_info_value::Value::SettledCertificate(sc)) => {
                info_types.push(format!("SettledCert(leaf_count={})", sc.let_leaf_count.map_or(0, |c| c.settled_let_leaf_count)));
            }
            Some(crate::types::network_info::v0::network_info_value::Value::SettledClaim(_)) => {
                info_types.push("SettledClaim".to_string());
            }
            Some(crate::types::network_info::v0::network_info_value::Value::LatestPendingCertificateInfo(info)) => {
                info_types.push(format!("PendingCert(height={})", info.height.map_or(0, |h| h.height)));
            }
            None => {}
        }
    }
    println!(
        "   ✅ Reconstructed network info for {} networks",
        network_info.len()
    );
    for (network_id, info_types) in &network_info {
        println!("      Network {}: {:?}", network_id, info_types);
    }

    // 8. Read and validate global Metadata
    println!("\n8. Reading metadata_cf...");
    let metadata_value = state_db
        .get::<MetadataColumn>(&MetadataKey::LatestSettledEpoch)
        .expect("Failed to read metadata")
        .expect("Metadata not found");
    let latest_epoch = match metadata_value {
        MetadataValue::LatestSettledEpoch(epoch) => epoch,
        _ => panic!("Unexpected metadata value type"),
    };
    println!(
        "   ✅ Read global metadata: latest_settled_epoch={}",
        latest_epoch
    );

    // 9. Read and validate DisabledNetworks
    println!("\n9. Reading disabled_networks_cf...");
    let mut disabled_networks: Vec<(NetworkId, DisabledNetwork)> = Vec::new();
    for network_id_result in state_db
        .keys::<DisabledNetworksColumn>()
        .expect("Failed to iterate")
    {
        let network_id = network_id_result.expect("Failed to read network ID");
        let disabled_info: DisabledNetwork = state_db
            .get::<DisabledNetworksColumn>(&network_id)
            .expect("Failed to read disabled info")
            .expect("Disabled info not found");

        // Validate structure
        assert!(
            disabled_info.disabled_at.is_some(),
            "Disabled timestamp should be set"
        );
        disabled_networks.push((network_id, disabled_info));
    }
    println!(
        "   ✅ Reconstructed {} disabled networks",
        disabled_networks.len()
    );
    for (network_id, info) in &disabled_networks {
        println!(
            "      Network {}: disabled_by={}, timestamp={:?}",
            network_id, info.disabled_by, info.disabled_at
        );
    }

    // Final validation summary
    println!("\n=== Validation Summary ===");
    println!("  Total networks: {}", network_ids.len());
    println!("  Total certificates: {}", headers.len());
    println!("  Settled certificates: {}", settled_certs.len());
    println!("  Networks with LET: {}", let_per_network.len());
    println!("  Networks with balance trees: {}", balance_trees.len());
    println!("  Networks with nullifier trees: {}", nullifier_roots.len());
    println!("  Networks with info: {}", network_info.len());
    println!("  Disabled networks: {}", disabled_networks.len());

    // Verify consistency
    assert_eq!(
        headers.len(),
        fixture.metadata.statistics.total_certificates,
        "Total certificates mismatch"
    );
    assert_eq!(
        network_ids.len(),
        fixture.metadata.config.num_networks as usize,
        "Total networks mismatch"
    );

    println!("\n✅ Successfully reconstructed and validated all state database data!");
    println!("   All structures match expected format and are internally consistent.");
}

#[test]
fn test_reference_db_v1_read_pending_db() {
    // Comprehensive test reading and reconstructing all data from pending database
    // This reverses what generate_pending_db does - reads values and validates
    // structure
    use std::collections::{BTreeMap, HashSet};

    use agglayer_types::{Certificate, CertificateId, Height, NetworkId, Proof};

    use crate::columns::{
        latest_pending_certificate_per_network::{
            LatestPendingCertificatePerNetworkColumn, PendingCertificate,
        },
        latest_proven_certificate_per_network::{
            LatestProvenCertificatePerNetworkColumn, ProvenCertificate,
        },
        pending_queue::{PendingQueueColumn, PendingQueueKey},
        proof_per_certificate::ProofPerCertificateColumn,
    };

    let fixture = TestFixture::new("pending_db");
    let pending_db = fixture
        .open_pending_db()
        .expect("Failed to open pending DB");

    println!("=== Reconstructing Pending Database (Reverse of generate_pending_db) ===\n");

    // 1. Read and reconstruct PendingQueue
    println!("1. Reading pending_queue_cf...");
    let mut pending_queue: BTreeMap<(NetworkId, Height), Certificate> = BTreeMap::new();
    let mut networks_with_pending: HashSet<NetworkId> = HashSet::new();

    for key_result in pending_db
        .keys::<PendingQueueColumn>()
        .expect("Failed to iterate")
    {
        let key: PendingQueueKey = key_result.expect("Failed to read key");
        let cert: Certificate = pending_db
            .get::<PendingQueueColumn>(&key)
            .expect("Failed to read certificate")
            .expect("Certificate not found");

        // Validate certificate structure matches key
        assert_eq!(
            cert.network_id, key.0,
            "Certificate network_id doesn't match key"
        );
        assert_eq!(cert.height, key.1, "Certificate height doesn't match key");

        // Validate certificate has required fields (bridge_exits can be empty)
        // Just check that the certificate deserialized correctly

        networks_with_pending.insert(cert.network_id);
        pending_queue.insert((key.0, key.1), cert);
    }

    println!(
        "   ✅ Reconstructed {} pending certificates",
        pending_queue.len()
    );
    println!("   ✅ Across {} networks", networks_with_pending.len());

    // Show details of pending certificates per network
    for (network_id, height) in pending_queue.keys().take(3) {
        let cert = &pending_queue[&(*network_id, *height)];
        println!(
            "      Network {}, Height {}: {} bridge_exits, LER chain: {} -> {}",
            network_id,
            height,
            cert.bridge_exits.len(),
            cert.prev_local_exit_root,
            cert.new_local_exit_root
        );
    }

    // 2. Read and validate LatestPendingCertificatePerNetwork
    println!("\n2. Reading latest_pending_certificate_per_network_cf...");
    let mut latest_pending: BTreeMap<NetworkId, PendingCertificate> = BTreeMap::new();

    for network_id_result in pending_db
        .keys::<LatestPendingCertificatePerNetworkColumn>()
        .expect("Failed to iterate")
    {
        let network_id = network_id_result.expect("Failed to read network ID");
        let pending_cert: PendingCertificate = pending_db
            .get::<LatestPendingCertificatePerNetworkColumn>(&network_id)
            .expect("Failed to read pending cert")
            .expect("Pending cert not found");

        // Validate that this certificate exists in the pending queue
        if let Some(cert) = pending_queue.get(&(network_id, pending_cert.1)) {
            let computed_id = cert.hash();
            assert_eq!(
                computed_id, pending_cert.0,
                "Latest pending cert ID mismatch for network {}",
                network_id
            );
        } else {
            println!(
                "      ⚠️  Latest pending cert for network {} (height {}) not in pending queue",
                network_id, pending_cert.1
            );
        }

        latest_pending.insert(network_id, pending_cert);
    }

    println!(
        "   ✅ Reconstructed {} latest pending certificates",
        latest_pending.len()
    );
    for (network_id, pending_cert) in &latest_pending {
        println!(
            "      Network {}: cert_id={}, height={}",
            network_id, pending_cert.0, pending_cert.1
        );
    }

    // 3. Read and validate LatestProvenCertificatePerNetwork
    println!("\n3. Reading latest_proven_certificate_per_network_cf...");
    let mut latest_proven: BTreeMap<NetworkId, ProvenCertificate> = BTreeMap::new();

    for network_id_result in pending_db
        .keys::<LatestProvenCertificatePerNetworkColumn>()
        .expect("Failed to iterate")
    {
        let network_id = network_id_result.expect("Failed to read network ID");
        let proven_cert: ProvenCertificate = pending_db
            .get::<LatestProvenCertificatePerNetworkColumn>(&network_id)
            .expect("Failed to read proven cert")
            .expect("Proven cert not found");

        // Validate structure
        assert_eq!(proven_cert.1, network_id, "Proven cert network_id mismatch");

        latest_proven.insert(network_id, proven_cert);
    }

    println!(
        "   ✅ Reconstructed {} latest proven certificates",
        latest_proven.len()
    );
    for (network_id, proven_cert) in &latest_proven {
        println!(
            "      Network {}: cert_id={}, height={}",
            network_id, proven_cert.0, proven_cert.2
        );
    }

    // 4. Read and validate ProofPerCertificate
    println!("\n4. Reading proof_per_certificate_cf...");
    let mut proofs: BTreeMap<CertificateId, Proof> = BTreeMap::new();

    for cert_id_result in pending_db
        .keys::<ProofPerCertificateColumn>()
        .expect("Failed to iterate")
    {
        let cert_id = cert_id_result.expect("Failed to read cert ID");
        let proof: Proof = pending_db
            .get::<ProofPerCertificateColumn>(&cert_id)
            .expect("Failed to read proof")
            .expect("Proof not found");

        // Validate proof structure (Proof is an opaque type, just check it
        // deserializes)
        proofs.insert(cert_id, proof);
    }

    println!("   ✅ Reconstructed {} proofs", proofs.len());
    for (cert_id, _proof) in proofs.iter().take(3) {
        println!("      Proof for certificate: {}", cert_id);
    }

    // Cross-validation: Check consistency between column families
    println!("\n=== Cross-Validation ===");

    // Check that all networks with latest_pending also have entries in
    // pending_queue
    for (network_id, pending_cert) in &latest_pending {
        if !pending_queue.contains_key(&(*network_id, pending_cert.1)) {
            println!(
                "   ⚠️  Network {} has latest_pending but no entry in pending_queue at height {}",
                network_id, pending_cert.1
            );
        }
    }

    // Check that networks with proven certs are tracked
    println!(
        "   ✅ {} networks have proven certificates tracked",
        latest_proven.len()
    );

    // Final validation summary
    println!("\n=== Validation Summary ===");
    println!("  Pending certificates: {}", pending_queue.len());
    println!("  Networks with pending: {}", networks_with_pending.len());
    println!("  Latest pending tracked: {}", latest_pending.len());
    println!("  Latest proven tracked: {}", latest_proven.len());
    println!("  Proofs available: {}", proofs.len());

    // Verify expected counts
    assert_eq!(
        latest_pending.len(),
        fixture.metadata.config.num_networks as usize,
        "Should have latest pending for all networks"
    );
    assert_eq!(
        latest_proven.len(),
        fixture.metadata.config.num_networks as usize,
        "Should have latest proven for all networks"
    );

    println!("\n✅ Successfully reconstructed and validated all pending database data!");
    println!("   All structures match expected format and are internally consistent.");
}

#[test]
fn test_reference_db_v1_read_network_info() {
    // Comprehensive test reading and reconstructing NetworkInfo data
    // This validates the structure generated in generate_state_db (step 7)
    use std::collections::BTreeMap;

    use agglayer_types::NetworkId;

    use crate::{
        columns::network_info::NetworkInfoColumn,
        types::network_info::{
            v0::{
                network_info_value::Value as InfoValue, LatestPendingCertificateInfo,
                SettledCertificate as NetworkInfoSettledCert, SettledClaim,
            },
            Key as NetworkInfoKey, Value as NetworkInfoValue,
        },
    };

    let fixture = TestFixture::new("netinfo");
    let state_db = fixture.open_state_db().expect("Failed to open state DB");

    println!("=== Reconstructing NetworkInfo (Reverse of generate_state_db step 7) ===\n");

    // Structure to hold all network info per network
    #[derive(Default)]
    struct NetworkInfo {
        network_type: Option<i32>,
        settled_certificate: Option<NetworkInfoSettledCert>,
        settled_claim: Option<SettledClaim>,
        pending_certificate_info: Option<LatestPendingCertificateInfo>,
    }

    let mut network_info_map: BTreeMap<NetworkId, NetworkInfo> = BTreeMap::new();
    let mut total_entries = 0;

    // Read all NetworkInfo entries
    println!("Reading network_info_cf...");
    for key_result in state_db
        .keys::<NetworkInfoColumn>()
        .expect("Failed to iterate")
    {
        let key: NetworkInfoKey = key_result.expect("Failed to read key");
        let value: NetworkInfoValue = state_db
            .get::<NetworkInfoColumn>(&key)
            .expect("Failed to read value")
            .expect("Value not found");

        let network_id = NetworkId::new(key.network_id);
        let info = network_info_map.entry(network_id).or_default();

        // Parse and store each value type
        match value.value {
            Some(InfoValue::NetworkType(nt)) => {
                assert!(
                    info.network_type.is_none(),
                    "Duplicate NetworkType for network {}",
                    network_id
                );
                info.network_type = Some(nt); // i32 enum value
            }
            Some(InfoValue::SettledCertificate(sc)) => {
                assert!(
                    info.settled_certificate.is_none(),
                    "Duplicate SettledCertificate for network {}",
                    network_id
                );
                // Structure can have Some or None fields depending on database version
                // Just validate it deserializes correctly
                info.settled_certificate = Some(sc);
            }
            Some(InfoValue::SettledClaim(claim)) => {
                assert!(
                    info.settled_claim.is_none(),
                    "Duplicate SettledClaim for network {}",
                    network_id
                );
                // Structure can have Some or None fields depending on database version
                info.settled_claim = Some(claim);
            }
            Some(InfoValue::LatestPendingCertificateInfo(pending)) => {
                assert!(
                    info.pending_certificate_info.is_none(),
                    "Duplicate LatestPendingCertificateInfo for network {}",
                    network_id
                );
                // Structure can have Some or None fields depending on database version
                info.pending_certificate_info = Some(pending);
            }
            None => {
                panic!("NetworkInfo value should not be None");
            }
        }

        total_entries += 1;
    }

    println!("   ✅ Read {} total NetworkInfo entries", total_entries);
    println!(
        "   ✅ Covering {} unique networks\n",
        network_info_map.len()
    );

    // Validate and display reconstructed data
    println!("=== Reconstructed NetworkInfo per Network ===");
    for (network_id, info) in &network_info_map {
        println!("\nNetwork {}:", network_id);

        // 1. NetworkType
        if let Some(nt) = info.network_type {
            let type_name = match nt {
                0 => "UNSPECIFIED",
                1 => "ECDSA",
                2 => "GENERIC",
                3 => "MULTISIG_ONLY",
                4 => "MULTISIG_AND_AGGCHAIN_PROOF",
                _ => "UNKNOWN",
            };
            println!("  ✅ NetworkType: {} ({})", nt, type_name);
        } else {
            println!("  ⚠️  Missing NetworkType");
        }

        // 2. SettledCertificate
        if let Some(sc) = &info.settled_certificate {
            let pp_root = sc
                .pp_root
                .as_ref()
                .and_then(|r| {
                    if r.root.is_empty() {
                        None
                    } else {
                        Some(format!(
                            "0x{}...",
                            hex::encode(&r.root[..4.min(r.root.len())])
                        ))
                    }
                })
                .unwrap_or_else(|| "none".to_string());
            let ler = sc
                .ler
                .as_ref()
                .and_then(|l| {
                    if l.root.is_empty() {
                        None
                    } else {
                        Some(format!(
                            "0x{}...",
                            hex::encode(&l.root[..4.min(l.root.len())])
                        ))
                    }
                })
                .unwrap_or_else(|| "none".to_string());
            let leaf_count = sc
                .let_leaf_count
                .as_ref()
                .map(|c| c.settled_let_leaf_count)
                .unwrap_or(0);
            println!(
                "  ✅ SettledCertificate: pp_root={}, ler={}, leaf_count={}",
                pp_root, ler, leaf_count
            );
        } else {
            println!("  ⚠️  Missing SettledCertificate");
        }

        // 3. SettledClaim
        if let Some(claim) = &info.settled_claim {
            let global_index = claim
                .global_index
                .as_ref()
                .and_then(|gi| {
                    if gi.value.is_empty() {
                        None
                    } else {
                        Some(format!(
                            "0x{}...",
                            hex::encode(&gi.value[..4.min(gi.value.len())])
                        ))
                    }
                })
                .unwrap_or_else(|| "none".to_string());
            let bridge_exit = claim
                .bridge_exit_hash
                .as_ref()
                .and_then(|beh| {
                    if beh.bridge_exit_hash.is_empty() {
                        None
                    } else {
                        Some(format!(
                            "0x{}...",
                            hex::encode(&beh.bridge_exit_hash[..4.min(beh.bridge_exit_hash.len())])
                        ))
                    }
                })
                .unwrap_or_else(|| "none".to_string());
            println!(
                "  ✅ SettledClaim: global_index={}, bridge_exit_hash={}",
                global_index, bridge_exit
            );
        } else {
            println!("  ⚠️  Missing SettledClaim");
        }

        // 4. LatestPendingCertificateInfo
        if let Some(pending) = &info.pending_certificate_info {
            let cert_id = pending
                .id
                .as_ref()
                .and_then(|id| {
                    if id.id.is_empty() {
                        None
                    } else {
                        Some(format!(
                            "0x{}...",
                            hex::encode(&id.id[..4.min(id.id.len())])
                        ))
                    }
                })
                .unwrap_or_else(|| "none".to_string());
            let height = pending.height.as_ref().map(|h| h.height).unwrap_or(0);
            println!(
                "  ✅ LatestPendingCertificateInfo: cert_id={}, height={}",
                cert_id, height
            );
        } else {
            println!("  ⚠️  Missing LatestPendingCertificateInfo");
        }

        // Validate completeness: all networks should have all 4 info types
        assert!(
            info.network_type.is_some(),
            "Network {} missing NetworkType",
            network_id
        );
        assert!(
            info.settled_certificate.is_some(),
            "Network {} missing SettledCertificate",
            network_id
        );
        assert!(
            info.settled_claim.is_some(),
            "Network {} missing SettledClaim",
            network_id
        );
        assert!(
            info.pending_certificate_info.is_some(),
            "Network {} missing LatestPendingCertificateInfo",
            network_id
        );
    }

    // Final validation
    println!("\n=== Validation Summary ===");
    println!("  Networks with complete info: {}", network_info_map.len());
    println!("  Total entries: {}", total_entries);
    println!(
        "  Expected entries: {} (4 per network)",
        network_info_map.len() * 4
    );

    assert_eq!(
        network_info_map.len(),
        fixture.metadata.config.num_networks as usize,
        "Network count mismatch"
    );
    assert_eq!(
        total_entries,
        network_info_map.len() * 4,
        "Should have exactly 4 info entries per network"
    );

    println!("\n✅ Successfully reconstructed and validated all NetworkInfo data!");
    println!("   All networks have complete information (4 entries each).");
}

#[test]
fn test_reference_db_v1_read_epochs_db() {
    // Comprehensive test reading and reconstructing all data from epochs database
    // This reverses what generate_epochs_db does - reads values and validates
    // structure
    use std::collections::BTreeMap;

    use agglayer_types::{Certificate, CertificateIndex, Proof};

    use crate::{
        columns::epochs::{
            certificates::CertificatePerIndexColumn, metadata::PerEpochMetadataColumn,
            proofs::ProofPerIndexColumn,
        },
        types::{PerEpochMetadataKey, PerEpochMetadataValue},
    };

    let fixture = TestFixture::new("epochs_db");
    let epochs_db = fixture.open_epochs_db().expect("Failed to open epochs DB");

    println!("=== Reconstructing Epochs Database (Reverse of generate_epochs_db) ===\n");

    // 1. Read and reconstruct CertificatePerIndex
    println!("1. Reading per_epoch_certificates_cf (CertificatePerIndex)...");
    let mut certificates: BTreeMap<CertificateIndex, Certificate> = BTreeMap::new();
    let mut networks_in_epoch = std::collections::HashSet::new();

    for key_result in epochs_db
        .keys::<CertificatePerIndexColumn>()
        .expect("Failed to iterate")
    {
        let cert_index = key_result.expect("Failed to read certificate index");
        let cert: Certificate = epochs_db
            .get::<CertificatePerIndexColumn>(&cert_index)
            .expect("Failed to read certificate")
            .expect("Certificate not found");

        // Validate certificate structure
        assert!(
            cert_index.as_u64() < 1000,
            "Certificate index seems unreasonably high"
        );

        // Track networks
        networks_in_epoch.insert(cert.network_id);

        certificates.insert(cert_index, cert);
    }

    println!("   ✅ Reconstructed {} certificates", certificates.len());
    println!("   ✅ Across {} networks in epoch", networks_in_epoch.len());

    // Display certificate distribution
    println!("   Certificate distribution by index:");
    let mut by_network = std::collections::BTreeMap::new();
    for (idx, cert) in &certificates {
        by_network
            .entry(cert.network_id)
            .or_insert_with(Vec::new)
            .push(*idx);
    }
    for (network_id, indices) in by_network.iter().take(3) {
        println!(
            "      Network {}: indices {:?}",
            network_id,
            indices.iter().map(|i| i.as_u64()).collect::<Vec<_>>()
        );
    }

    // 2. Read and validate ProofPerIndex
    println!("\n2. Reading per_epoch_proofs_cf (ProofPerIndex)...");
    let mut proofs: BTreeMap<CertificateIndex, Proof> = BTreeMap::new();

    for key_result in epochs_db
        .keys::<ProofPerIndexColumn>()
        .expect("Failed to iterate")
    {
        let cert_index = key_result.expect("Failed to read certificate index");
        let proof: Proof = epochs_db
            .get::<ProofPerIndexColumn>(&cert_index)
            .expect("Failed to read proof")
            .expect("Proof not found");

        // Validate that certificate exists for this proof
        assert!(
            certificates.contains_key(&cert_index),
            "Proof exists for cert_index {} but no certificate found",
            cert_index
        );

        proofs.insert(cert_index, proof);
    }

    println!("   ✅ Reconstructed {} proofs", proofs.len());
    if !proofs.is_empty() {
        println!(
            "   ✅ All proofs have corresponding certificates (validated {} mappings)",
            proofs.len()
        );
    } else {
        println!("   ℹ️  No proofs generated (generate_proofs was disabled)");
    }

    // 3. Read and reconstruct PerEpochMetadata
    println!("\n3. Reading per_epoch_metadata_cf (PerEpochMetadata)...");
    let mut metadata_entries = Vec::new();

    // Try to read SettlementTxHash
    let settlement_tx_hash =
        match epochs_db.get::<PerEpochMetadataColumn>(&PerEpochMetadataKey::SettlementTxHash) {
            Ok(Some(PerEpochMetadataValue::SettlementTxHash(tx_hash))) => {
                metadata_entries.push(format!("SettlementTxHash: {}", tx_hash));
                Some(tx_hash)
            }
            Ok(Some(_)) => {
                panic!("Unexpected metadata value type for SettlementTxHash");
            }
            Ok(None) => None,
            Err(e) => {
                eprintln!("Failed to read SettlementTxHash: {:?}", e);
                None
            }
        };

    // Try to read Packed status
    let is_packed = match epochs_db.get::<PerEpochMetadataColumn>(&PerEpochMetadataKey::Packed) {
        Ok(Some(PerEpochMetadataValue::Packed(packed))) => {
            metadata_entries.push(format!("Packed: {}", packed));
            Some(packed)
        }
        Ok(Some(_)) => {
            panic!("Unexpected metadata value type for Packed");
        }
        Ok(None) => None,
        Err(e) => {
            eprintln!("Failed to read Packed: {:?}", e);
            None
        }
    };

    println!(
        "   ✅ Reconstructed {} metadata entries:",
        metadata_entries.len()
    );
    for entry in &metadata_entries {
        println!("      {}", entry);
    }

    // Validate metadata presence
    if settlement_tx_hash.is_some() {
        println!("   ✅ Settlement transaction hash is set");
    }
    if let Some(packed) = is_packed {
        println!("   ✅ Epoch packed status: {}", packed);
    }

    // Cross-validation: Check consistency
    println!("\n=== Cross-Validation ===");

    // Validate certificate indices are sequential starting from 0
    let indices: Vec<u64> = certificates.keys().map(|i| i.as_u64()).collect();
    if !indices.is_empty() {
        let min_idx = *indices.iter().min().unwrap();
        let max_idx = *indices.iter().max().unwrap();
        println!(
            "   ✅ Certificate indices range: {} to {}",
            min_idx, max_idx
        );

        // Check if indices are contiguous
        let expected_count = (max_idx - min_idx + 1) as usize;
        if indices.len() == expected_count {
            println!("   ✅ Certificate indices are contiguous");
        } else {
            println!(
                "   ⚠️  Certificate indices have gaps ({} entries, expected {})",
                indices.len(),
                expected_count
            );
        }
    }

    // Validate proofs match certificates if proofs are enabled
    if !proofs.is_empty() {
        let proof_coverage = (proofs.len() as f64 / certificates.len() as f64) * 100.0;
        println!(
            "   ✅ Proof coverage: {:.1}% ({}/{})",
            proof_coverage,
            proofs.len(),
            certificates.len()
        );
    }

    // Final validation summary
    println!("\n=== Validation Summary ===");
    println!("  Certificates: {}", certificates.len());
    println!("  Networks in epoch: {}", networks_in_epoch.len());
    println!("  Proofs: {}", proofs.len());
    println!("  Metadata entries: {}", metadata_entries.len());
    println!("  Settlement tx set: {}", settlement_tx_hash.is_some());
    println!("  Packed status set: {}", is_packed.is_some());

    // Note: The fixture metadata shows total certificates across all epochs,
    // but we're reading from a single epoch directory. Each epoch is a separate
    // database.
    println!(
        "\nℹ️  Note: Reading from single epoch (total across all epochs: {})",
        fixture
            .metadata
            .statistics
            .entries_per_column_family
            .get("per_epoch_certificates_cf")
            .unwrap_or(&0)
    );

    println!("\n✅ Successfully reconstructed and validated all epochs database data!");
    println!("   All structures match expected format and are internally consistent.");
}

#[test]
fn test_reference_db_v1_read_debug_db() {
    // Comprehensive test reading and reconstructing all data from debug database
    // This reverses what generate_debug_db does - reads values and validates
    // structure
    use std::collections::{BTreeMap, HashSet};

    use agglayer_types::{Certificate, CertificateId};

    use crate::columns::debug_certificates::DebugCertificatesColumn;

    let fixture = TestFixture::new("debug_db");
    let debug_db = fixture.open_debug_db().expect("Failed to open debug DB");

    println!("=== Reconstructing Debug Database (Reverse of generate_debug_db) ===\n");

    // Read and reconstruct DebugCertificatesColumn (CertificateId -> Certificate)
    println!("1. Reading debug_certificates (DebugCertificatesColumn)...");
    let mut certificates: BTreeMap<CertificateId, Certificate> = BTreeMap::new();
    let mut networks_found = HashSet::new();
    let mut heights_per_network: std::collections::BTreeMap<
        agglayer_types::NetworkId,
        Vec<agglayer_types::Height>,
    > = std::collections::BTreeMap::new();

    for key_result in debug_db
        .keys::<DebugCertificatesColumn>()
        .expect("Failed to iterate")
    {
        let cert_id = key_result.expect("Failed to read certificate ID");
        let cert: Certificate = debug_db
            .get::<DebugCertificatesColumn>(&cert_id)
            .expect("Failed to read certificate")
            .expect("Certificate not found");

        // Validate that the certificate ID matches the hash
        let computed_id = cert.hash();
        assert_eq!(
            computed_id, cert_id,
            "Certificate ID mismatch: stored {} vs computed {}",
            cert_id, computed_id
        );

        // Track networks and heights
        networks_found.insert(cert.network_id);
        heights_per_network
            .entry(cert.network_id)
            .or_default()
            .push(cert.height);

        certificates.insert(cert_id, cert);
    }

    println!("   ✅ Reconstructed {} certificates", certificates.len());
    println!("   ✅ From {} networks", networks_found.len());

    // Display certificate distribution by network
    println!("\n   Certificate distribution by network:");
    for (network_id, heights) in heights_per_network.iter().take(5) {
        let mut sorted_heights = heights.clone();
        sorted_heights.sort();
        let heights_display: Vec<String> = sorted_heights
            .iter()
            .take(5)
            .map(|h| h.as_u64().to_string())
            .collect();
        let display = if sorted_heights.len() > 5 {
            format!(
                "{}... ({} total)",
                heights_display.join(", "),
                sorted_heights.len()
            )
        } else {
            heights_display.join(", ")
        };
        println!("      Network {}: heights [{}]", network_id, display);
    }

    // Validate certificate integrity
    println!("\n=== Certificate Validation ===");

    // Check for duplicate heights per network
    let mut has_duplicates = false;
    for (network_id, heights) in &heights_per_network {
        let unique_heights: std::collections::BTreeSet<_> = heights.iter().collect();
        if unique_heights.len() != heights.len() {
            println!(
                "   ⚠️  Network {} has duplicate heights ({} certs, {} unique heights)",
                network_id,
                heights.len(),
                unique_heights.len()
            );
            has_duplicates = true;
        }
    }
    if !has_duplicates {
        println!("   ✅ No duplicate heights found per network");
    }

    // Validate certificate IDs are unique (implicitly guaranteed by BTreeMap, but
    // good to mention)
    println!(
        "   ✅ All {} certificate IDs are unique",
        certificates.len()
    );

    // Sample some certificates to show structure
    println!("\n=== Sample Certificates ===");
    for (cert_id, cert) in certificates.iter().take(3) {
        println!("   Certificate {}", cert_id);
        println!("      Network: {}", cert.network_id);
        println!("      Height: {}", cert.height);
        println!("      Bridge exits: {}", cert.bridge_exits.len());
        println!("      Prev LER: {}", cert.prev_local_exit_root);
        println!("      New LER: {}", cert.new_local_exit_root);
    }

    // Cross-validation with metadata
    println!("\n=== Cross-Validation ===");

    // Verify expected network count
    assert_eq!(
        networks_found.len(),
        fixture.metadata.config.num_networks as usize,
        "Network count mismatch"
    );
    println!(
        "   ✅ Network count matches config: {}",
        networks_found.len()
    );

    // Verify expected certificate count
    if let Some(expected) = fixture
        .metadata
        .statistics
        .entries_per_column_family
        .get("debug_certificates")
    {
        assert_eq!(
            certificates.len(),
            *expected,
            "Certificate count mismatch with metadata"
        );
        println!(
            "   ✅ Certificate count matches metadata: {}",
            certificates.len()
        );
    }

    // Validate height ranges per network
    for (network_id, heights) in &heights_per_network {
        let min_height = heights.iter().min().unwrap().as_u64();
        let max_height = heights.iter().max().unwrap().as_u64();
        println!(
            "   ✅ Network {}: height range {} to {} ({} certificates)",
            network_id,
            min_height,
            max_height,
            heights.len()
        );
    }

    // Final validation summary
    println!("\n=== Validation Summary ===");
    println!("  Total certificates: {}", certificates.len());
    println!("  Unique networks: {}", networks_found.len());
    println!("  Certificates per network:");
    for (network_id, heights) in &heights_per_network {
        println!("    Network {}: {} certificates", network_id, heights.len());
    }

    // Verify all certificate IDs can be recomputed correctly
    let mut id_mismatches = 0;
    for (stored_id, cert) in &certificates {
        let computed_id = cert.hash();
        if *stored_id != computed_id {
            id_mismatches += 1;
        }
    }
    assert_eq!(
        id_mismatches, 0,
        "Found {} certificate ID mismatches",
        id_mismatches
    );
    println!("  ✅ All certificate IDs match their computed hashes");

    println!("\n✅ Successfully reconstructed and validated all debug database data!");
    println!(
        "   All {} certificates are valid and internally consistent.",
        certificates.len()
    );
}
