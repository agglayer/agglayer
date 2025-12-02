//! Database Generator for Regression Testing
//!
//! This module provides utilities to generate populated RocksDB databases
//! for regression testing across version upgrades (e.g., alloy 0.14 -> 1.0).
//!
//! The generated databases contain realistic test data across all column
//! families and can be used as artifacts for deserialization regression tests.
//!
//! Data generated is random, using mock proofs, semantically incorrect so it
//! should be used only for database read/write/serialization/deserialization
//! testing.

use std::path::Path;

use agglayer_tries::roots::LocalExitRoot;
use agglayer_types::{
    testutils::AggchainDataType, Certificate, CertificateHeader, CertificateId, CertificateIndex,
    CertificateStatus, Height, NetworkId, Proof,
};
use pessimistic_proof::{
    core::commitment::SignatureCommitmentVersion, local_exit_tree::LocalExitTree,
};
use rand::{Rng, SeedableRng};
use strum::IntoEnumIterator;

use crate::{
    columns::{
        balance_tree_per_network::BalanceTreePerNetworkColumn,
        certificate_header::CertificateHeaderColumn,
        certificate_per_network::{CertificatePerNetworkColumn, Key as CertPerNetKey},
        debug_certificates::DebugCertificatesColumn,
        disabled_networks::DisabledNetworksColumn,
        epochs::{
            certificates::CertificatePerIndexColumn, metadata::PerEpochMetadataColumn,
            proofs::ProofPerIndexColumn,
        },
        latest_pending_certificate_per_network::{
            LatestPendingCertificatePerNetworkColumn, PendingCertificate,
        },
        latest_proven_certificate_per_network::{
            LatestProvenCertificatePerNetworkColumn, ProvenCertificate,
        },
        latest_settled_certificate_per_network::{
            LatestSettledCertificatePerNetworkColumn, SettledCertificate,
        },
        local_exit_tree_per_network::LocalExitTreePerNetworkColumn,
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
    types::{
        network_info::{Key as NetworkInfoKey, Value as NetworkInfoValue},
        MetadataKey, MetadataValue, PerEpochMetadataKey, PerEpochMetadataValue,
    },
};

/// Number of certificates per epoch for testing purposes
const CERTIFICATES_PER_EPOCH: u64 = 3;

/// Configuration for database generation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

/// Metadata structure for generated database artifacts
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DatabaseMetadata {
    /// Version identifier for this database artifact
    pub version: String,
    /// Generation timestamp
    pub timestamp: String,
    /// Configuration used to generate the databases
    pub config: GeneratorConfig,
    /// Statistics about generated data
    pub statistics: GenerationStatistics,
    /// Database paths relative to the metadata file
    pub database_paths: DatabasePaths,
}

/// Statistics about the generated database
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GenerationStatistics {
    pub total_networks: usize,
    pub total_certificates: usize,
    pub entries_per_column_family: std::collections::HashMap<String, usize>,
}

/// Database paths structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DatabasePaths {
    pub state: String,
    pub pending: String,
    pub epochs: String,
    pub debug: String,
}

/// Result of database generation
#[derive(Debug, Clone)]
pub struct GenerationResult {
    /// Number of entries written to each column family
    pub entries_per_cf: std::collections::HashMap<String, usize>,
    /// List of network IDs that were generated
    pub network_ids: Vec<NetworkId>,
    /// List of certificate IDs that were generated
    pub certificate_ids: Vec<CertificateId>,
    /// Map of (network_id, height_as_u64) to Certificate for reuse across
    /// databases
    pub certificates: std::collections::HashMap<(NetworkId, u64), Certificate>,
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
        certificates: std::collections::HashMap::new(),
    };

    // Generate state database
    let state_path = base_path.join("state");
    let state_result = generate_state_db(&state_path, config)?;
    result
        .entries_per_cf
        .extend(state_result.entries_per_cf.clone());
    result.network_ids = state_result.network_ids.clone();
    result.certificate_ids = state_result.certificate_ids.clone();
    result.certificates = state_result.certificates.clone();

    // Generate pending database
    let pending_path = base_path.join("pending");
    let pending_result = generate_pending_db(&pending_path, config, &state_result)?;
    result.entries_per_cf.extend(pending_result.entries_per_cf);

    // Generate epochs database
    let epochs_path = base_path.join("epochs");
    let epochs_result = generate_epochs_db(&epochs_path, config, &state_result)?;
    result.entries_per_cf.extend(epochs_result.entries_per_cf);

    // Generate debug database
    let debug_path = base_path.join("debug");
    let debug_result = generate_debug_db(&debug_path, config, &state_result)?;
    result.entries_per_cf.extend(debug_result.entries_per_cf);

    Ok(result)
}

/// Generate state database with all column families
pub fn generate_state_db(
    path: &Path,
    config: &GeneratorConfig,
) -> Result<GenerationResult, crate::storage::DBError> {
    let db = DB::open_cf(path, state_db_cf_definitions())?;
    let mut rng = rand::rngs::StdRng::seed_from_u64(config.seed);

    let mut result = GenerationResult {
        entries_per_cf: std::collections::HashMap::new(),
        network_ids: Vec::new(),
        certificate_ids: Vec::new(),
        certificates: std::collections::HashMap::new(),
    };

    // Generate data for each network
    for network_idx in 0..config.num_networks {
        let network_id = NetworkId::new(network_idx + 1);
        result.network_ids.push(network_id);

        // Initialize the first local exit root for this network
        let initial_ler: LocalExitRoot = LocalExitTree::<32>::default().get_root().into();
        let mut prev_local_exit_root = initial_ler;

        // Track the latest settled certificate for this network
        let mut latest_settled_cert: Option<(CertificateId, Height, CertificateHeader)> = None;

        // Generate certificates for this network
        for height in 0..config.certificates_per_network {
            let height = Height::from(height);

            // Use RNG with deterministic seed based on network and height for consistent
            // results
            let param_seed = config
                .seed
                .wrapping_add(network_id.to_u32() as u64)
                .wrapping_add(height.as_u64());
            let mut param_rng = rand::rngs::StdRng::seed_from_u64(param_seed);

            // Vary parameters based on height to create diverse test data
            let num_bridge_exits = param_rng.random_range(0..5); // 0-4 bridge exits

            // Generate aggchain_data_type using the generate_for_test function
            let aggchain_data_type = AggchainDataType::generate_for_test(param_seed);

            // Generate version using the generate_for_test function
            let version = SignatureCommitmentVersion::generate_for_test(param_seed);

            let certificate = Certificate::new_for_test_custom(
                network_id,
                height,
                prev_local_exit_root,
                num_bridge_exits,
                aggchain_data_type,
                version,
            );

            // Update prev_local_exit_root for the next certificate
            prev_local_exit_root = certificate.new_local_exit_root;

            let cert_id = certificate.hash();
            result.certificate_ids.push(cert_id);

            // Store certificate for reuse in other databases
            result
                .certificates
                .insert((network_id, height.as_u64()), certificate.clone());

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
            let header = CertificateHeader::generate_for_test(
                param_seed,
                network_id,
                height,
                cert_id,
                certificate.prev_local_exit_root,
                certificate.new_local_exit_root,
            );
            db.put::<CertificateHeaderColumn>(&cert_id, &header)?;
            *result
                .entries_per_cf
                .entry("certificate_header_cf".to_string())
                .or_insert(0) += 1;

            // 3. Track the latest settled certificate for this network
            if matches!(header.status, CertificateStatus::Settled) {
                latest_settled_cert = Some((cert_id, height, header.clone()));
            }
        }

        // After processing all certificates, write the latest settled certificate if
        // found
        if let Some((cert_id, height, header)) = latest_settled_cert {
            let settled_cert = SettledCertificate(
                cert_id,
                height,
                header
                    .epoch_number
                    .expect("Settled certificate should have epoch_number"),
                header
                    .certificate_index
                    .expect("Settled certificate should have certificate_index"),
            );
            db.put::<LatestSettledCertificatePerNetworkColumn>(&network_id, &settled_cert)?;
            *result
                .entries_per_cf
                .entry("latest_settled_certificate_per_network_cf".to_string())
                .or_insert(0) += 1;
        }

        // 4. LocalExitTree: Generate a small exit tree with a few leaves
        let let_seed = config
            .seed
            .wrapping_add(network_id.to_u32() as u64)
            .wrapping_add(2000);
        let num_leaves = rng.random_range(2..5);
        let let_entries =
            crate::types::testutils::generate_let_for_test(let_seed, network_id, num_leaves);
        for (key, value) in let_entries {
            db.put::<LocalExitTreePerNetworkColumn>(&key, &value)?;
            *result
                .entries_per_cf
                .entry("local_exit_tree_per_network_cf".to_string())
                .or_insert(0) += 1;
        }

        // 5. BalanceTree: Generate sparse merkle tree nodes
        let balance_tree_seed = config.seed.wrapping_add(network_id.to_u32() as u64);
        let balance_tree_entries = crate::types::testutils::generate_smt_for_test(
            balance_tree_seed,
            network_id,
            4, // num_leaves (will create 3 internal nodes + 1 root)
        );
        for (key, value) in balance_tree_entries {
            db.put::<BalanceTreePerNetworkColumn>(&key, &value)?;
            *result
                .entries_per_cf
                .entry("balance_tree_per_network_cf".to_string())
                .or_insert(0) += 1;
        }

        // 6. NullifierTree: Generate sparse merkle tree nodes
        let nullifier_tree_seed = config
            .seed
            .wrapping_add(network_id.to_u32() as u64)
            .wrapping_add(1000);
        let nullifier_tree_entries = crate::types::testutils::generate_smt_for_test(
            nullifier_tree_seed,
            network_id,
            4, // num_leaves (will create 3 internal nodes + 1 root)
        );
        for (key, value) in nullifier_tree_entries {
            db.put::<NullifierTreePerNetworkColumn>(&key, &value)?;
            *result
                .entries_per_cf
                .entry("nullifier_tree_per_network_cf".to_string())
                .or_insert(0) += 1;
        }

        // 7. NetworkInfo: Store network information

        // Get the last settled certificate for this network to use in NetworkInfo
        let last_cert_height = Height::from(config.certificates_per_network - 1);
        let last_certificate = result
            .certificates
            .get(&(network_id, last_cert_height.as_u64()))
            .expect("Last certificate should exist");
        let last_cert_id = last_certificate.hash();

        for kind in crate::types::network_info::v0::network_info_value::ValueDiscriminants::iter() {
            let key = NetworkInfoKey {
                network_id: network_id.to_u32(),
                kind,
            };

            // Create appropriate value based on discriminant with realistic random data
            let value = match kind {
                crate::types::network_info::v0::network_info_value::ValueDiscriminants::NetworkType => {
                    // Generate network type using seed for deterministic results
                    let network_type_seed = config
                        .seed
                        .wrapping_add(network_id.to_u32() as u64)
                        .wrapping_add(5000); // Add offset to differentiate from other random values
                    let network_type = agglayer_types::NetworkType::generate_for_test(network_type_seed);
                    // Convert to storage type
                    let storage_network_type: crate::types::network_info::v0::NetworkType = network_type.into();
                    NetworkInfoValue {
                        value: Some(crate::types::network_info::v0::network_info_value::Value::NetworkType(
                            storage_network_type as i32,
                        )),
                    }
                }
                crate::types::network_info::v0::network_info_value::ValueDiscriminants::SettledCertificate => {
                    // Use random data for settled certificate fields
                    let pp_root_bytes: Vec<u8> = rng.random::<[u8; 32]>().to_vec();
                    let ler_bytes: Vec<u8> = rng.random::<[u8; 32]>().to_vec();
                    NetworkInfoValue {
                        value: Some(crate::types::network_info::v0::network_info_value::Value::SettledCertificate(
                            crate::types::network_info::v0::SettledCertificate {
                                certificate_id: Some(crate::types::network_info::v0::SettledCertificateId {
                                    id: prost::bytes::Bytes::from(last_cert_id.as_ref().to_vec()),
                                }),
                                pp_root: Some(crate::types::network_info::v0::SettledPessimisticProofRoot {
                                    root: prost::bytes::Bytes::from(pp_root_bytes),
                                }),
                                let_leaf_count: Some(crate::types::network_info::v0::SettledLocalExitTreeLeafCount {
                                    settled_let_leaf_count: num_leaves as u64,
                                }),
                                ler: Some(crate::types::network_info::v0::SettledLocalExitRoot {
                                    root: prost::bytes::Bytes::from(ler_bytes),
                                }),
                            },
                        )),
                    }
                }
                crate::types::network_info::v0::network_info_value::ValueDiscriminants::SettledClaim => {
                    // Generate random but realistic hashes
                    let random_global_index: Vec<u8> = rng.random::<[u8; 32]>().to_vec();
                    let random_bridge_exit_hash: Vec<u8> = rng.random::<[u8; 32]>().to_vec();

                    NetworkInfoValue {
                        value: Some(crate::types::network_info::v0::network_info_value::Value::SettledClaim(
                            crate::types::network_info::v0::SettledClaim {
                                global_index: Some(crate::types::network_info::v0::GlobalIndex {
                                    value: prost::bytes::Bytes::from(random_global_index),
                                }),
                                bridge_exit_hash: Some(crate::types::network_info::v0::BridgeExitHash {
                                    bridge_exit_hash: prost::bytes::Bytes::from(random_bridge_exit_hash),
                                }),
                            },
                        )),
                    }
                }
                crate::types::network_info::v0::network_info_value::ValueDiscriminants::LatestPendingCertificateInfo => {
                    // Use the last certificate as the latest pending
                    NetworkInfoValue {
                        value: Some(crate::types::network_info::v0::network_info_value::Value::LatestPendingCertificateInfo(
                            crate::types::network_info::v0::LatestPendingCertificateInfo {
                                height: Some(crate::types::network_info::v0::LatestPendingCertificateHeight {
                                    height: last_cert_height.as_u64(),
                                }),
                                id: Some(crate::types::network_info::v0::LatestPendingCertificateId {
                                    id: prost::bytes::Bytes::from(last_cert_id.as_ref().to_vec()),
                                }),
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

    // 9. DisabledNetworks: Randomly disable some networks
    // Disable approximately 30% of networks for testing purposes
    for network_id in &result.network_ids {
        if rng.random_bool(0.3) {
            use crate::types::disabled_network::{DisabledBy, Value as DisabledNetwork};

            let disabled_network = DisabledNetwork {
                disabled_at: Some(prost_types::Timestamp {
                    seconds: 1700000000i64 + rng.random_range(0..10000000),
                    nanos: rng.random_range(0..1000000000),
                }),
                disabled_by: if rng.random_bool(0.5) {
                    DisabledBy::Admin as i32
                } else {
                    DisabledBy::Unspecified as i32
                },
            };
            db.put::<DisabledNetworksColumn>(network_id, &disabled_network)?;
            *result
                .entries_per_cf
                .entry("disabled_networks_cf".to_string())
                .or_insert(0) += 1;
        }
    }

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
        certificates: std::collections::HashMap::new(),
    };

    // Generate data for each network
    for network_id in &state_result.network_ids {
        // Get the last certificate from state_result to continue the chain
        let last_state_height = config.certificates_per_network - 1;
        let last_state_cert = state_result
            .certificates
            .get(&(*network_id, last_state_height))
            .expect("Certificate should exist from state generation");
        let mut prev_local_exit_root = last_state_cert.new_local_exit_root;

        // 1. PendingQueue: Add a few new pending certificates (continuing the chain)
        let num_pending = config.certificates_per_network.min(2);
        let mut last_pending_cert_id = None;
        let mut last_pending_height = Height::ZERO;

        for i in 0..num_pending {
            let height = Height::from(config.certificates_per_network + i);

            // Vary parameters based on height
            let num_bridge_exits = ((height.as_u64() + 1) % 5) as usize;

            // Use deterministic seed based on network and height for consistent results
            let param_seed = config
                .seed
                .wrapping_add(network_id.to_u32() as u64)
                .wrapping_add(height.as_u64());

            // Generate aggchain_data_type using the generate_for_test function
            let aggchain_data_type = AggchainDataType::generate_for_test(param_seed);

            // Generate version using the generate_for_test function
            let version = SignatureCommitmentVersion::generate_for_test(param_seed);

            let certificate = Certificate::new_for_test_custom(
                *network_id,
                height,
                prev_local_exit_root,
                num_bridge_exits,
                aggchain_data_type,
                version,
            );

            // Update prev_local_exit_root for the next certificate
            prev_local_exit_root = certificate.new_local_exit_root;

            // Save last certificate info for LatestPendingCertificatePerNetwork
            last_pending_cert_id = Some(certificate.hash());
            last_pending_height = height;

            let key = PendingQueueKey(*network_id, height);
            db.put::<PendingQueueColumn>(&key, &certificate)?;
            *result
                .entries_per_cf
                .entry("pending_queue_cf".to_string())
                .or_insert(0) += 1;
        }

        // 2. LatestPendingCertificatePerNetwork (use the last pending certificate from
        //    the loop above)
        if let Some(cert_id) = last_pending_cert_id {
            let pending_cert = PendingCertificate(cert_id, last_pending_height);
            db.put::<LatestPendingCertificatePerNetworkColumn>(network_id, &pending_cert)?;
            *result
                .entries_per_cf
                .entry("latest_pending_certificate_per_network_cf".to_string())
                .or_insert(0) += 1;
        }

        // 3. LatestProvenCertificatePerNetwork (use one of the state certificates)
        if config.certificates_per_network > 1 {
            let proven_height = Height::from(config.certificates_per_network - 2);
            let certificate = state_result
                .certificates
                .get(&(*network_id, proven_height.as_u64()))
                .expect("Certificate should exist from state generation")
                .clone();
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

/// Generate epochs database using the same certificates from generate_state_db
pub fn generate_epochs_db(
    path: &Path,
    config: &GeneratorConfig,
    state_result: &GenerationResult,
) -> Result<GenerationResult, crate::storage::DBError> {
    let db = DB::open_cf(path, epochs_db_cf_definitions())?;
    let mut rng = rand::rngs::StdRng::seed_from_u64(config.seed);

    let mut result = GenerationResult {
        entries_per_cf: std::collections::HashMap::new(),
        network_ids: state_result.network_ids.clone(),
        certificate_ids: Vec::new(),
        certificates: std::collections::HashMap::new(),
    };

    // Calculate how many complete epochs we'll have based on
    // certificates_per_network
    let num_epochs = config
        .certificates_per_network
        .div_ceil(CERTIFICATES_PER_EPOCH);

    // Generate data for each network
    for network_id in &state_result.network_ids {
        // Generate certificates grouped by epochs
        for epoch_idx in 0..num_epochs {
            // Calculate how many certificates are in this epoch
            let certs_in_this_epoch = std::cmp::min(
                config
                    .certificates_per_network
                    .saturating_sub(epoch_idx * CERTIFICATES_PER_EPOCH),
                CERTIFICATES_PER_EPOCH,
            );

            // Generate certificates for this epoch
            for cert_idx in 0..certs_in_this_epoch {
                let certificate_index = CertificateIndex::new(cert_idx);
                let height = Height::from(epoch_idx * CERTIFICATES_PER_EPOCH + cert_idx);

                // Reuse the certificate generated in state_db
                let certificate = state_result
                    .certificates
                    .get(&(*network_id, height.as_u64()))
                    .expect("Certificate should exist from state generation")
                    .clone();
                let cert_id = certificate.hash();
                result.certificate_ids.push(cert_id);

                // 1. CertificatePerIndex: certificate_index -> certificate
                db.put::<CertificatePerIndexColumn>(&certificate_index, &certificate)?;
                *result
                    .entries_per_cf
                    .entry("per_epoch_certificates_cf".to_string())
                    .or_insert(0) += 1;

                // 2. ProofPerIndex: certificate_index -> proof (if enabled)
                if config.generate_proofs {
                    let proof = Proof::dummy();
                    db.put::<ProofPerIndexColumn>(&certificate_index, &proof)?;
                    *result
                        .entries_per_cf
                        .entry("per_epoch_proofs_cf".to_string())
                        .or_insert(0) += 1;
                }
            }

            // 3. PerEpochMetadata: Store metadata for this epoch
            // Store a random settlement tx hash
            let settlement_tx_hash_key = PerEpochMetadataKey::SettlementTxHash;
            let settlement_tx_hash_value = PerEpochMetadataValue::SettlementTxHash(
                agglayer_types::Digest(rng.random::<[u8; 32]>()),
            );
            db.put::<PerEpochMetadataColumn>(&settlement_tx_hash_key, &settlement_tx_hash_value)?;
            *result
                .entries_per_cf
                .entry("per_epoch_metadata_cf".to_string())
                .or_insert(0) += 1;

            // Store packed status
            let packed_key = PerEpochMetadataKey::Packed;
            let packed_value = PerEpochMetadataValue::Packed(true);
            db.put::<PerEpochMetadataColumn>(&packed_key, &packed_value)?;
            *result
                .entries_per_cf
                .entry("per_epoch_metadata_cf".to_string())
                .or_insert(0) += 1;
        }
    }

    Ok(result)
}

/// Generate debug database using the same certificates from generate_state_db
pub fn generate_debug_db(
    path: &Path,
    _config: &GeneratorConfig,
    state_result: &GenerationResult,
) -> Result<GenerationResult, crate::storage::DBError> {
    let db = DB::open_cf(path, debug_db_cf_definitions())?;

    let mut result = GenerationResult {
        entries_per_cf: std::collections::HashMap::new(),
        network_ids: state_result.network_ids.clone(),
        certificate_ids: Vec::new(),
        certificates: std::collections::HashMap::new(),
    };

    // Store all certificates from state_result in the debug database
    // The debug database maps CertificateId -> Certificate
    for ((_network_id, _height), certificate) in &state_result.certificates {
        let cert_id = certificate.hash();
        result.certificate_ids.push(cert_id);

        // Store certificate in debug database
        db.put::<DebugCertificatesColumn>(&cert_id, certificate)?;
        *result
            .entries_per_cf
            .entry("debug_certificates".to_string())
            .or_insert(0) += 1;
    }

    Ok(result)
}
