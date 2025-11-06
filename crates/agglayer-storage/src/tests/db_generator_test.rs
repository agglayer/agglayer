//! Tests for the database generator

use super::{
    db_generator::{generate_all_databases, GeneratorConfig},
    TempDBDir,
};

#[test]
fn test_generate_databases() {
    let temp_dir = TempDBDir::new();
    let config = GeneratorConfig {
        num_networks: 2,
        certificates_per_network: 3,
        generate_proofs: true,
        seed: 42,
    };

    let result =
        generate_all_databases(&temp_dir.path, &config).expect("Failed to generate databases");

    // Verify we have the expected number of networks
    assert_eq!(result.network_ids.len(), 2);

    // Verify we have certificates
    assert_eq!(result.certificate_ids.len(), 6); // 2 networks * 3 certs

    // Verify we have entries in column families
    assert!(!result.entries_per_cf.is_empty());

    // Print statistics for manual verification
    println!("Generated {} networks", result.network_ids.len());
    println!("Generated {} certificates", result.certificate_ids.len());
    println!("Column family entries:");
    for (cf, count) in &result.entries_per_cf {
        println!("  {}: {} entries", cf, count);
    }
}

#[test]
fn test_generate_minimal_database() {
    let temp_dir = TempDBDir::new();
    let config = GeneratorConfig {
        num_networks: 1,
        certificates_per_network: 1,
        generate_proofs: false,
        seed: 123,
    };

    let result =
        generate_all_databases(&temp_dir.path, &config).expect("Failed to generate databases");

    assert_eq!(result.network_ids.len(), 1);
    assert_eq!(result.certificate_ids.len(), 1);
}
