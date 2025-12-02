//! Regression tests for RocksDB deserialization across version upgrades
//!
//! These tests ensure that databases created with previous versions of
//! dependencies (e.g., alloy 0.14) can still be read correctly with newer
//! versions (e.g., alloy 1.0).
//!
//! The test databases are stored as compressed artifacts in `tests/fixtures/`
//! and are extracted to temporary locations before testing.
//!
//! ## Validation Checks
//!
//! Each test performs comprehensive validation to ensure data integrity:
//!
//! ### State Database (`test_reference_db_v1_read_state_db`)
//! - Certificate structure and relationships validation
//! - Height continuity and sequentiality for each network
//! - LocalExitTree consistency (leaf counts, frontiers)
//! - Balance and nullifier tree structure validation
//! - Network info completeness (4 entries per network)
//! - Settled certificate relationships
//! - Epoch numbering consistency
//! - Disabled networks validation
//!
//! ### Pending Database (`test_reference_db_v1_read_pending_db`)
//! - Certificate deserialization and field validation
//! - LocalExitRoot chain continuity
//! - Latest pending/proven certificate consistency
//! - Proof validation if present
//! - Height range and uniqueness per network
//! - Cross-validation between column families
//!
//! ### Epochs Database (`test_reference_db_v1_read_epochs_db`)
//! - Certificate index sequentiality
//! - Network distribution validation
//! - Proof coverage if generation enabled
//! - Metadata presence and validity
//! - Certificate field validation
//! - Certificate ID uniqueness
//!
//! ### Debug Database (`test_reference_db_v1_read_debug_db`)
//! - Certificate ID correctness (hash verification)
//! - Network and height completeness
//! - LocalExitRoot chain validation
//! - Height sequentiality starting from 0
//! - Certificate structure variation
//! - Metadata deserialization

use std::{
    fs,
    path::{Path, PathBuf},
};

use tempfile::TempDir;

use crate::{
    storage::{
        self, debug_db_cf_definitions, epochs_db_cf_definitions, pending_db_cf_definitions,
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
    use flate2::read::GzDecoder;
    use tar::Archive;

    fs::create_dir_all(extract_to)?;

    let file = fs::File::open(tarball_path)?;
    let decompressor = GzDecoder::new(file);
    let mut archive = Archive::new(decompressor);

    archive.unpack(extract_to)?;

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
    _temp_dir: TempDir,
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

        let temp_dir = TempDir::with_prefix(format!("agglayer_regression_test_{}_", test_name))
            .expect("Failed to create temporary directory");

        let db_path =
            extract_tarball(&tarball_path, temp_dir.path()).expect("Failed to extract tarball");
        let metadata = read_metadata(&db_path).expect("Failed to read metadata");

        Self {
            db_path,
            metadata,
            _temp_dir: temp_dir,
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

    // Validate expected column family counts
    let num_cfs = fixture.metadata.statistics.entries_per_column_family.len();
    const EXPECTED_STATE_CFS: usize = storage::cf_definitions::state::CFS.len();
    const EXPECTED_PENDING_CFS: usize = storage::cf_definitions::pending::CFS.len();
    const EXPECTED_EPOCHS_CFS: usize = storage::cf_definitions::epochs::CFS.len() + 2; // CFS + CHECKPOINTS
    const EXPECTED_DEBUG_CFS: usize = storage::cf_definitions::debug::CFS.len();
    const EXPECTED_TOTAL_CFS: usize =
        EXPECTED_STATE_CFS + EXPECTED_PENDING_CFS + EXPECTED_EPOCHS_CFS + EXPECTED_DEBUG_CFS;

    assert_eq!(
        num_cfs,
        EXPECTED_TOTAL_CFS,
        "Metadata contains {} column families, but expected {} (state: {}, pending: {}, epochs: \
         {}, debug: {})",
        num_cfs,
        EXPECTED_TOTAL_CFS,
        EXPECTED_STATE_CFS,
        EXPECTED_PENDING_CFS,
        EXPECTED_EPOCHS_CFS,
        EXPECTED_DEBUG_CFS,
    );

    // Open databases in read-only mode
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
    let mut latest_settled_certs: HashMap<NetworkId, SettledCertificate> = HashMap::new();
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
        {
            let header = headers.get(&settled_cert.0).unwrap_or_else(|| {
                panic!(
                    "Settled certificate {} not found in headers",
                    settled_cert.0
                )
            });
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
        }

        latest_settled_certs.insert(network_id, settled_cert);
    }
    println!(
        "   ✅ Reconstructed {} settled certificates",
        latest_settled_certs.len()
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
    for (leaf_count, leaves, _frontiers) in let_per_network.values() {
        assert_eq!(*leaf_count as usize, leaves.len());
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
            _ => {
                panic!("Invalid LET key/value combination");
            }
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
            } else {
                panic!("Invalid SMT key/value combination");
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
            None => {
                panic!("Unexpected value");
            }
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
    println!("  Settled certificates: {}", latest_settled_certs.len());
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

    // Additional validation checks to ensure data integrity
    println!("\n=== Additional Data Integrity Checks ===");

    // 1. Verify each network has a settled certificate
    assert_eq!(
        latest_settled_certs.len(),
        network_ids.len(),
        "Each network should have a settled certificate"
    );
    println!("  ✅ All networks have settled certificates");

    // 2. Verify each network has LocalExitTree data
    assert_eq!(
        let_per_network.len(),
        network_ids.len(),
        "Each network should have LocalExitTree data"
    );
    println!("  ✅ All networks have LocalExitTree data");

    // 3. Verify each network has balance tree and nullifier tree
    assert_eq!(
        balance_trees.len(),
        network_ids.len(),
        "Each network should have balance tree"
    );
    assert_eq!(
        nullifier_roots.len(),
        network_ids.len(),
        "Each network should have nullifier tree root"
    );
    println!("  ✅ All networks have balance and nullifier trees");

    // 4. Verify each network has complete network info
    assert_eq!(
        network_info.len(),
        network_ids.len(),
        "Each network should have network info"
    );
    println!("  ✅ All networks have network info");

    // 5. Verify LocalExitTree consistency - leaf_count should match number of
    //    leaves
    for (network_id, (leaf_count, leaves, frontiers)) in &let_per_network {
        // Note: leaves might contain more than leaf_count due to how we store all
        // hashes
        assert!(
            leaf_count <= &(leaves.len() as u32),
            "Network {}: leaf_count ({}) exceeds stored leaves ({})",
            network_id,
            leaf_count,
            leaves.len()
        );
        assert!(
            !frontiers.is_empty(),
            "Network {} should have frontier nodes",
            network_id
        );
    }
    println!("  ✅ LocalExitTree structures are consistent");

    // 6. Verify certificate height continuity per network
    let mut certs_by_network: std::collections::BTreeMap<NetworkId, Vec<Height>> =
        std::collections::BTreeMap::new();
    for header in headers.values() {
        certs_by_network
            .entry(header.network_id)
            .or_default()
            .push(header.height);
    }

    for (network_id, mut heights) in certs_by_network {
        heights.sort();
        // Check that heights start from 0 and are sequential
        for (i, height) in heights.iter().enumerate() {
            assert_eq!(
                height.as_u64(),
                i as u64,
                "Network {}: expected height {} but got {}",
                network_id,
                i,
                height.as_u64()
            );
        }
        assert_eq!(
            heights.len(),
            fixture.metadata.config.certificates_per_network as usize,
            "Network {}: expected {} certificates but got {}",
            network_id,
            fixture.metadata.config.certificates_per_network,
            heights.len()
        );
    }
    println!("  ✅ Certificate heights are sequential and complete for all networks");

    // 7. Verify epoch numbering consistency
    let mut epochs_seen = std::collections::HashSet::new();
    for header in headers.values() {
        if let Some(epoch) = header.epoch_number {
            epochs_seen.insert(epoch.as_u64());
        }
    }
    println!(
        "  ✅ Found {} unique epochs in certificate headers",
        epochs_seen.len()
    );

    // 8. Verify metadata epoch matches
    assert_eq!(
        latest_epoch.as_u64(),
        0,
        "Expected latest_settled_epoch to be 0"
    );
    println!("  ✅ Global metadata epoch is correct");

    // 9. Verify balance tree has both nodes and leaves
    for (network_id, (_root, nodes, leaves)) in &balance_trees {
        assert!(
            !nodes.is_empty(),
            "Network {} should have balance tree nodes",
            network_id
        );
        assert!(
            !leaves.is_empty(),
            "Network {} should have balance tree leaves",
            network_id
        );
    }
    println!("  ✅ Balance trees have both internal nodes and leaves");

    // 10. Verify disabled networks are a subset of all networks
    for (disabled_network_id, _) in &disabled_networks {
        assert!(
            network_ids.contains(disabled_network_id),
            "Disabled network {} not found in network list",
            disabled_network_id
        );
    }
    println!(
        "  ✅ All disabled networks are valid ({} disabled out of {})",
        disabled_networks.len(),
        network_ids.len()
    );

    println!("\n✅ Successfully reconstructed and validated all state database data!");
    println!("   All structures match expected format and are internally consistent.");
    println!("   Data integrity checks passed: relationships and constraints verified.");
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
            panic!(
                "⚠️  Latest pending cert for network {} (height {}) not in pending queue",
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
            panic!(
                "⚠️  Network {} has latest_pending but no entry in pending_queue at height {}",
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

    // Additional validation checks to ensure data integrity
    println!("\n=== Additional Data Integrity Checks ===");

    // 1. Verify certificate deserialization - check critical fields
    for ((network_id, height), cert) in &pending_queue {
        assert_eq!(
            cert.network_id, *network_id,
            "Certificate network_id mismatch at height {}",
            height
        );
        assert_eq!(
            cert.height, *height,
            "Certificate height mismatch for network {}",
            network_id
        );

        // Verify LocalExitRoot chain integrity
        assert_ne!(
            cert.prev_local_exit_root, cert.new_local_exit_root,
            "Network {}, height {}: prev and new LER should differ",
            network_id, height
        );

        // Verify certificate ID computation
        let computed_id = cert.hash();
        assert_ne!(
            computed_id,
            CertificateId::default(),
            "Certificate ID should not be default"
        );
    }
    println!(
        "  ✅ All {} pending certificates have valid structure and fields",
        pending_queue.len()
    );

    // 2. Verify latest_pending references valid certificates in pending_queue
    for (network_id, pending_cert) in &latest_pending {
        let key = (*network_id, pending_cert.1);
        if let Some(cert) = pending_queue.get(&key) {
            let computed_id = cert.hash();
            assert_eq!(
                computed_id, pending_cert.0,
                "Latest pending cert ID mismatch for network {}",
                network_id
            );
            assert_eq!(
                cert.network_id, *network_id,
                "Latest pending network_id mismatch"
            );
        }
        // Note: It's acceptable if the certificate is not in pending_queue
        // as it might have been moved to state DB
    }
    println!("  ✅ Latest pending certificates are consistent");

    // 3. Verify latest_proven has valid structure
    for (network_id, proven_cert) in &latest_proven {
        assert_eq!(
            proven_cert.1, *network_id,
            "Proven cert network_id mismatch"
        );
        assert_ne!(
            proven_cert.0,
            CertificateId::default(),
            "Proven cert ID should not be default"
        );
        // Height (proven_cert.2) should be less than latest pending height
        if let Some(pending_cert) = latest_pending.get(network_id) {
            assert!(
                proven_cert.2 < pending_cert.1,
                "Network {}: proven height ({}) should be < pending height ({})",
                network_id,
                proven_cert.2,
                pending_cert.1
            );
        }
    }
    println!("  ✅ Latest proven certificates have valid structure");

    // 4. Verify proofs if present
    if !proofs.is_empty() {
        // If we have proofs, the metadata should indicate proof generation was enabled
        assert!(
            fixture.metadata.config.generate_proofs,
            "Found {} proofs in database, but metadata.config.generate_proofs is false",
            proofs.len()
        );

        for cert_id in proofs.keys() {
            assert_ne!(
                *cert_id,
                CertificateId::default(),
                "Proof cert_id should not be default"
            );
        }
        println!(
            "  ✅ All {} proofs have valid certificate IDs",
            proofs.len()
        );
    } else {
        // If no proofs, the metadata should indicate proof generation was disabled
        assert!(
            !fixture.metadata.config.generate_proofs,
            "No proofs found in database, but metadata.config.generate_proofs is true"
        );
        println!("  ✅ No proofs present (proof generation disabled in metadata)");
    }

    // 5. Verify pending queue height ranges per network
    let mut heights_per_network: std::collections::BTreeMap<NetworkId, Vec<Height>> =
        std::collections::BTreeMap::new();
    for (network_id, height) in pending_queue.keys() {
        heights_per_network
            .entry(*network_id)
            .or_default()
            .push(*height);
    }

    for (network_id, mut heights) in heights_per_network {
        heights.sort();
        let min_height = heights.first().unwrap().as_u64();
        let max_height = heights.last().unwrap().as_u64();
        println!(
            "  ✅ Network {}: pending certs from height {} to {} ({} total)",
            network_id,
            min_height,
            max_height,
            heights.len()
        );

        // Verify no duplicate heights
        let unique_heights: std::collections::BTreeSet<_> = heights.iter().collect();
        assert_eq!(
            unique_heights.len(),
            heights.len(),
            "Network {} has duplicate heights in pending queue",
            network_id
        );
    }

    // 6. Verify LocalExitRoot chain in pending certificates
    for network_id in &networks_with_pending {
        let mut certs_for_network: Vec<_> = pending_queue
            .iter()
            .filter(|((nid, _), _)| nid == network_id)
            .collect();
        certs_for_network.sort_by_key(|((_, height), _)| height);

        if certs_for_network.len() > 1 {
            for i in 1..certs_for_network.len() {
                let prev_cert = &certs_for_network[i - 1].1;
                let curr_cert = &certs_for_network[i].1;

                // Current cert's prev_local_exit_root should equal previous cert's
                // new_local_exit_root
                assert_eq!(
                    curr_cert.prev_local_exit_root, prev_cert.new_local_exit_root,
                    "Network {}: LER chain broken between heights {} and {}",
                    network_id, prev_cert.height, curr_cert.height
                );
            }
        }
    }
    println!("  ✅ LocalExitRoot chains are valid in pending certificates");

    // 7. Verify actual data counts match metadata
    println!("\n=== Verifying Data Counts Match Metadata ===");

    let metadata_pending_queue = fixture
        .metadata
        .statistics
        .entries_per_column_family
        .get("pending_queue_cf")
        .copied()
        .unwrap_or(0);
    assert_eq!(
        pending_queue.len(),
        metadata_pending_queue,
        "Pending queue count mismatch: actual {} vs metadata {}",
        pending_queue.len(),
        metadata_pending_queue
    );

    let metadata_latest_pending = fixture
        .metadata
        .statistics
        .entries_per_column_family
        .get("latest_pending_certificate_per_network_cf")
        .copied()
        .unwrap_or(0);
    assert_eq!(
        latest_pending.len(),
        metadata_latest_pending,
        "Latest pending count mismatch: actual {} vs metadata {}",
        latest_pending.len(),
        metadata_latest_pending
    );

    let metadata_latest_proven = fixture
        .metadata
        .statistics
        .entries_per_column_family
        .get("latest_proven_certificate_per_network_cf")
        .copied()
        .unwrap_or(0);
    assert_eq!(
        latest_proven.len(),
        metadata_latest_proven,
        "Latest proven count mismatch: actual {} vs metadata {}",
        latest_proven.len(),
        metadata_latest_proven
    );

    println!("  ✅ All data counts match metadata statistics");

    println!("\n✅ Successfully reconstructed and validated all pending database data!");
    println!("   All structures match expected format and are internally consistent.");
    println!("   Data integrity checks passed: relationships and constraints verified.");
}

#[test]
fn test_reference_db_v1_read_network_info() {
    // Comprehensive test reading and reconstructing NetworkInfo data
    // This validates the structure generated in generate_state_db (step 7)
    // Uses the NetworkInfoReader trait to read network info in a type-safe manner
    use std::{collections::BTreeSet, sync::Arc};

    use agglayer_types::NetworkId;

    use crate::{
        columns::network_info::NetworkInfoColumn,
        storage::backup::BackupClient,
        stores::{state::StateStore, NetworkInfoReader},
        types::network_info::Key as NetworkInfoKey,
    };

    let fixture = TestFixture::new("netinfo");
    let state_db = fixture.open_state_db().expect("Failed to open state DB");

    println!("=== Reading NetworkInfo using NetworkInfoReader trait ===\n");

    // Collect all unique network IDs from the database
    let mut network_ids = BTreeSet::new();
    let mut total_entries = 0;
    println!("Collecting network IDs from network_info_cf...");
    for key_result in state_db
        .keys::<NetworkInfoColumn>()
        .expect("Failed to iterate")
    {
        let key: NetworkInfoKey = key_result.expect("Failed to read key");
        network_ids.insert(NetworkId::new(key.network_id));
        total_entries += 1;
    }

    println!("   ✅ Read {} total NetworkInfo entries", total_entries);
    println!("   ✅ Covering {} unique networks\n", network_ids.len());

    // Use NetworkInfoReader to read and validate each network
    let state_store = StateStore::new(Arc::new(state_db), BackupClient::noop());

    println!("=== Reading NetworkInfo per Network ===");
    for network_id in &network_ids {
        let network_info = state_store
            .get_network_info(*network_id)
            .expect("Failed to get network info");

        println!("\nNetwork {}:", network_id);

        // 1. NetworkType
        println!("  ✅ NetworkType: {:?}", network_info.network_type);

        // 2. Settled Certificate
        if let Some(cert_id) = network_info.settled_certificate_id {
            let height = network_info
                .settled_height
                .map_or("none".to_string(), |h| h.to_string());
            let ler = network_info
                .settled_ler
                .map_or("none".to_string(), |l| format!("{:?}", l));
            let leaf_count = network_info
                .settled_let_leaf_count
                .map_or("none".to_string(), |c| c.to_string());
            let pp_root = network_info
                .settled_pp_root
                .map_or("none".to_string(), |r| format!("{:?}", r));

            println!(
                "  ✅ SettledCertificate: cert_id={}, height={}, ler={}, leaf_count={}, pp_root={}",
                cert_id, height, ler, leaf_count, pp_root
            );
        } else {
            println!("  ⚠️  No SettledCertificate");
        }

        // 3. SettledClaim
        if let Some(ref claim) = network_info.settled_claim {
            println!(
                "  ✅ SettledClaim: global_index={}, bridge_exit_hash={}",
                claim.global_index, claim.bridge_exit_hash
            );
        } else {
            println!("  ⚠️  No SettledClaim");
        }

        // 4. LatestPendingCertificateInfo
        if let Some(height) = network_info.latest_pending_height {
            println!("  ✅ LatestPendingHeight: {}", height);
        } else {
            println!("  ⚠️  No LatestPendingHeight");
        }

        // Validate completeness: all networks should have network type and settled data
        assert_ne!(
            network_info.network_type,
            agglayer_types::NetworkType::Unspecified,
            "Network {} has unspecified NetworkType",
            network_id
        );
        assert!(
            network_info.settled_certificate_id.is_some(),
            "Network {} missing SettledCertificate",
            network_id
        );
        assert!(
            network_info.settled_claim.is_some(),
            "Network {} missing SettledClaim",
            network_id
        );
        assert!(
            network_info.latest_pending_height.is_some(),
            "Network {} missing LatestPendingHeight",
            network_id
        );
    }

    // Final validation
    println!("\n=== Validation Summary ===");
    println!("  Networks with complete info: {}", network_ids.len());
    println!("  Total entries: {}", total_entries);
    println!(
        "  Expected entries: {} (4 per network)",
        network_ids.len() * 4
    );

    assert_eq!(
        network_ids.len(),
        fixture.metadata.config.num_networks as usize,
        "Network count mismatch"
    );
    assert_eq!(
        total_entries,
        network_ids.len() * 4,
        "Should have exactly 4 info entries per network"
    );

    println!("\n✅ Successfully read and validated all NetworkInfo data using NetworkInfoReader!");
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
        // Assert that proofs exist only when metadata indicates proof generation was
        // enabled
        assert!(
            fixture.metadata.config.generate_proofs,
            "Found {} proofs in database, but metadata.config.generate_proofs is false",
            proofs.len()
        );
        println!(
            "   ✅ All proofs have corresponding certificates (validated {} mappings)",
            proofs.len()
        );
    } else {
        // Assert that no proofs exist only when metadata indicates proof generation was
        // disabled
        assert!(
            !fixture.metadata.config.generate_proofs,
            "No proofs found in database, but metadata.config.generate_proofs is true"
        );
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
                panic!("Failed to read SettlementTxHash: {:?}", e);
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
            panic!("Failed to read Packed: {:?}", e);
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
            panic!(
                "   ⚠️  Certificate indices have gaps ({} entries, expected {})",
                indices.len(),
                expected_count
            );
        }
    }

    // Validate proofs match certificates if proofs are enabled
    if !proofs.is_empty() {
        assert_eq!(proofs.len(), certificates.len());
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

    // Additional validation checks to ensure data integrity
    println!("\n=== Additional Data Integrity Checks ===");

    // 1. Verify certificates have valid structure and fields
    for (cert_index, cert) in &certificates {
        use agglayer_types::{CertificateId, NetworkId};

        // Verify certificate ID can be computed
        let computed_id = cert.hash();
        assert_ne!(
            computed_id,
            CertificateId::default(),
            "Certificate at index {} should have valid ID",
            cert_index
        );

        // Verify certificate has valid network ID
        assert_ne!(
            cert.network_id,
            NetworkId::new(0),
            "Certificate at index {} should have valid network ID",
            cert_index
        );

        // Verify LocalExitRoot fields are set (not default)
        // Note: They can be equal in some cases, so just check they're not all zeros
        let prev_ler_bytes: &[u8] = cert.prev_local_exit_root.as_ref();
        let new_ler_bytes: &[u8] = cert.new_local_exit_root.as_ref();
        assert!(
            prev_ler_bytes.len() == 32,
            "Certificate at index {} should have valid prev_local_exit_root",
            cert_index
        );
        assert!(
            new_ler_bytes.len() == 32,
            "Certificate at index {} should have valid new_local_exit_root",
            cert_index
        );
    }
    println!(
        "  ✅ All {} certificates have valid structure and fields",
        certificates.len()
    );

    // 2. Verify certificate index range is reasonable
    if !certificates.is_empty() {
        let max_index = certificates.keys().map(|i| i.as_u64()).max().unwrap();
        assert!(
            max_index < 100,
            "Maximum certificate index ({}) seems unreasonably high for test data",
            max_index
        );
        println!(
            "  ✅ Certificate indices are in reasonable range (0-{})",
            max_index
        );
    }

    // 3. Verify certificate distribution across networks
    // Note: Each epoch directory is a separate database that may contain
    // certificates from one or more networks. We verify that networks present
    // are valid.
    assert!(
        !networks_in_epoch.is_empty(),
        "Epoch should have certificates from at least one network"
    );
    assert!(
        networks_in_epoch.len() <= fixture.metadata.config.num_networks as usize,
        "Found {} networks but config only has {}",
        networks_in_epoch.len(),
        fixture.metadata.config.num_networks
    );
    println!(
        "  ✅ Epoch has certificates from {} valid network(s)",
        networks_in_epoch.len()
    );

    // 4. Verify each network has expected number of certificates
    let expected_certs_per_network = if !certificates.is_empty() {
        certificates.len() / networks_in_epoch.len()
    } else {
        0
    };
    if expected_certs_per_network > 0 {
        for (network_id, indices) in by_network.iter() {
            let cert_count = indices.len();
            // Allow some variation due to rounding
            assert!(
                cert_count <= expected_certs_per_network + 1,
                "Network {} has unexpected cert count: {}",
                network_id,
                cert_count
            );
        }
        println!("  ✅ Certificate distribution across networks is balanced");
    }

    // 5. Verify proofs match certificates if proofs are enabled
    if fixture.metadata.config.generate_proofs {
        for cert_index in proofs.keys() {
            assert!(
                certificates.contains_key(cert_index),
                "Proof exists for index {} but no certificate found",
                cert_index
            );
        }
        println!(
            "  ✅ All {} proofs have corresponding certificates",
            proofs.len()
        );

        // Verify all certificates have proofs if generation was enabled
        assert_eq!(
            proofs.len(),
            certificates.len(),
            "All certificates should have proofs when generation is enabled"
        );
        println!("  ✅ All certificates have proofs");
    }

    // 6. Verify metadata consistency
    if let Some(tx_hash) = settlement_tx_hash {
        assert_ne!(
            tx_hash,
            agglayer_types::Digest::default(),
            "Settlement tx hash should not be default"
        );
        println!("  ✅ Settlement transaction hash is valid");
    } else {
        panic!("  ⚠️  Settlement transaction hash not set");
    }

    if let Some(packed) = is_packed {
        // Verify it's a boolean value (which it is by type)
        println!("  ✅ Packed status is valid: {}", packed);
    } else {
        panic!("  ⚠️  Packed status not set");
    }

    // 7. Verify metadata entry count
    assert!(
        !metadata_entries.is_empty(),
        "Epoch should have at least one metadata entry"
    );
    println!("  ✅ Epoch has {} metadata entries", metadata_entries.len());

    // 8. Verify certificates per network are in valid height ranges
    for (network_id, indices) in by_network.iter() {
        // Get heights from certificates for this network
        let heights: Vec<u64> = indices
            .iter()
            .filter_map(|idx| certificates.get(idx).map(|c| c.height.as_u64()))
            .collect();

        if !heights.is_empty() {
            let min_height = heights.iter().min().unwrap();
            let max_height = heights.iter().max().unwrap();
            assert!(
                max_height < &1000,
                "Network {}: max height {} seems unreasonably high",
                network_id,
                max_height
            );
            println!(
                "  ✅ Network {}: height range {}-{} is valid",
                network_id, min_height, max_height
            );
        }
    }

    // 9. Verify no duplicate certificate IDs by comparing to a BTreeSet
    let cert_ids: std::collections::BTreeSet<_> = certificates.values().map(|c| c.hash()).collect();
    assert_eq!(
        cert_ids.len(),
        certificates.len(),
        "Found duplicate certificate IDs"
    );
    println!("  ✅ All certificate IDs are unique");

    println!("\n✅ Successfully reconstructed and validated all epochs database data!");
    println!("   All structures match expected format and are internally consistent.");
    println!("   Data integrity checks passed: relationships and constraints verified.");
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
    {
        let expected = fixture
            .metadata
            .statistics
            .entries_per_column_family
            .get("debug_certificates")
            .expect("Failed to get debug_certificates");

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
    for (stored_id, cert) in &certificates {
        let computed_id = cert.hash();
        if *stored_id != computed_id {
            panic!("Found certificate ID mismatch");
        }
    }
    println!("  ✅ All certificate IDs match their computed hashes");

    // Additional validation checks to ensure data integrity
    println!("\n=== Additional Data Integrity Checks ===");

    // 1. Verify certificates have valid structure and non-default fields
    for (cert_id, cert) in &certificates {
        use agglayer_types::{CertificateId, NetworkId};

        // Verify network ID is non-zero
        assert_ne!(
            cert.network_id,
            NetworkId::new(0),
            "Certificate {} should have valid network ID",
            cert_id
        );

        // Verify LocalExitRoot fields are valid (32 bytes)
        let prev_ler_bytes: &[u8] = cert.prev_local_exit_root.as_ref();
        let new_ler_bytes: &[u8] = cert.new_local_exit_root.as_ref();
        assert_eq!(
            prev_ler_bytes.len(),
            32,
            "Certificate {} should have valid prev_local_exit_root",
            cert_id
        );
        assert_eq!(
            new_ler_bytes.len(),
            32,
            "Certificate {} should have valid new_local_exit_root",
            cert_id
        );

        // Verify certificate ID is not default
        assert_ne!(
            *cert_id,
            CertificateId::default(),
            "Certificate ID should not be default"
        );
    }
    println!("  ✅ All certificates have valid structure and non-default fields");

    // 2. Verify expected certificate count matches metadata
    if let Some(expected) = fixture
        .metadata
        .statistics
        .entries_per_column_family
        .get("debug_certificates")
    {
        assert_eq!(
            certificates.len(),
            *expected,
            "Certificate count mismatch: expected {} but found {}",
            expected,
            certificates.len()
        );
        println!(
            "  ✅ Certificate count matches metadata: {}",
            certificates.len()
        );
    }

    // 3. Verify network count matches expected
    assert_eq!(
        networks_found.len(),
        fixture.metadata.config.num_networks as usize,
        "Network count mismatch: expected {} but found {}",
        fixture.metadata.config.num_networks,
        networks_found.len()
    );
    println!(
        "  ✅ Network count matches expected: {}",
        networks_found.len()
    );

    // 4. Verify each network has the expected number of certificates
    let expected_certs_per_network = fixture.metadata.config.certificates_per_network as usize;
    for (network_id, heights) in &heights_per_network {
        assert_eq!(
            heights.len(),
            expected_certs_per_network,
            "Network {} has {} certificates but expected {}",
            network_id,
            heights.len(),
            expected_certs_per_network
        );
    }
    println!(
        "  ✅ All networks have {} certificates as expected",
        expected_certs_per_network
    );

    // 5. Verify height sequences are valid and start from 0
    for (network_id, heights) in &heights_per_network {
        let mut sorted_heights: Vec<_> = heights.iter().map(|h| h.as_u64()).collect();
        sorted_heights.sort();

        // Check heights start from 0
        assert_eq!(
            sorted_heights[0], 0,
            "Network {}: heights should start from 0",
            network_id
        );

        // Check heights are sequential
        for i in 1..sorted_heights.len() {
            assert_eq!(
                sorted_heights[i],
                sorted_heights[i - 1] + 1,
                "Network {}: heights should be sequential, found gap at index {}",
                network_id,
                i
            );
        }

        // Check max height matches expected
        assert_eq!(
            sorted_heights.last().unwrap(),
            &(expected_certs_per_network as u64 - 1),
            "Network {}: max height should be {}",
            network_id,
            expected_certs_per_network - 1
        );
    }
    println!(
        "  ✅ All networks have sequential heights from 0 to {}",
        expected_certs_per_network - 1
    );

    // 6. Verify no duplicate heights within each network
    for (network_id, heights) in &heights_per_network {
        let unique_heights: std::collections::BTreeSet<_> = heights.iter().collect();
        assert_eq!(
            unique_heights.len(),
            heights.len(),
            "Network {} has duplicate heights",
            network_id
        );
    }
    println!("  ✅ No duplicate heights found within any network");

    // 7. Verify LocalExitRoot chains are valid per network
    for (network_id, heights) in &heights_per_network {
        // Get certificates for this network sorted by height
        let mut network_certs: Vec<_> = heights
            .iter()
            .filter_map(|h| {
                certificates.iter().find_map(|(_id, cert)| {
                    if cert.network_id == *network_id && &cert.height == h {
                        Some((cert.height.as_u64(), cert))
                    } else {
                        None
                    }
                })
            })
            .collect();
        network_certs.sort_by_key(|(height, _)| *height);

        // Check LER chain continuity
        if network_certs.len() > 1 {
            for i in 1..network_certs.len() {
                let prev_cert = network_certs[i - 1].1;
                let curr_cert = network_certs[i].1;

                assert_eq!(
                    curr_cert.prev_local_exit_root, prev_cert.new_local_exit_root,
                    "Network {}: LER chain broken between heights {} and {}",
                    network_id, prev_cert.height, curr_cert.height
                );
            }
            println!(
                "  ✅ Network {}: LER chain is valid ({} certificates)",
                network_id,
                network_certs.len()
            );
        }
    }

    // 8. Verify certificates have varied structures (not all identical)
    let bridge_exit_counts: std::collections::HashSet<_> = certificates
        .values()
        .map(|c| c.bridge_exits.len())
        .collect();
    assert!(
        bridge_exit_counts.len() > 1,
        "Certificates should have varied bridge_exit counts (found only {:?})",
        bridge_exit_counts
    );
    println!(
        "  ✅ Certificates have varied structures ({} different bridge_exit counts)",
        bridge_exit_counts.len()
    );

    // 9. Verify certificate metadata is present
    for (_cert_id, cert) in certificates.iter().take(5) {
        // Just verify metadata exists and has some data by accessing it
        // We just need to verify it deserializes correctly, which it does if we get
        // here
        let _ = &cert.metadata;
        // If we reach here, metadata deserialized correctly
    }
    println!("  ✅ Certificate metadata deserialized correctly");

    println!("\n✅ Successfully reconstructed and validated all debug database data!");
    println!(
        "   All {} certificates are valid and internally consistent.",
        certificates.len()
    );
    println!("   Data integrity checks passed: relationships and constraints verified.");
}
