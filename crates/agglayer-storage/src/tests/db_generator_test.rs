//! Tests for the database generator

use super::{
    db_generator::{generate_all_databases, GeneratorConfig},
    TempDBDir,
};

#[test]
fn test_generate_databases() {
    let temp_dir = TempDBDir::new();
    let config = GeneratorConfig {
        num_networks: 3,
        certificates_per_network: 5,
        generate_proofs: true,
        seed: 42,
    };

    let result =
        generate_all_databases(&temp_dir.path, &config).expect("Failed to generate databases");

    // Verify we have the expected number of networks
    assert_eq!(result.network_ids.len(), config.num_networks as usize);

    // Verify we have certificates
    assert_eq!(
        result.certificate_ids.len(),
        (config.num_networks as u64 * config.certificates_per_network) as usize
    );

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

#[test]
fn test_certificate_chaining() {
    let temp_dir = TempDBDir::new();
    let config = GeneratorConfig {
        num_networks: 2,
        certificates_per_network: 5,
        generate_proofs: false,
        seed: 42,
    };

    let result =
        generate_all_databases(&temp_dir.path, &config).expect("Failed to generate databases");

    // Verify certificates are properly chained for each network
    for network_id in &result.network_ids {
        println!("\nVerifying certificate chain for network {:?}", network_id);

        // Check that each certificate's prev_local_exit_root matches the previous
        // cert's new_local_exit_root
        for height in 1..config.certificates_per_network {
            let prev_cert = result
                .certificates
                .get(&(*network_id, height - 1))
                .expect("Previous certificate should exist");
            let curr_cert = result
                .certificates
                .get(&(*network_id, height))
                .expect("Current certificate should exist");

            println!(
                "  Height {}: prev_ler matches = {}",
                height,
                curr_cert.prev_local_exit_root == prev_cert.new_local_exit_root
            );

            assert_eq!(
                curr_cert.prev_local_exit_root,
                prev_cert.new_local_exit_root,
                "Certificate at height {} should have prev_local_exit_root equal to the \
                 new_local_exit_root of certificate at height {}",
                height,
                height - 1
            );
        }

        println!(
            "  ✅ All certificates properly chained for network {:?}",
            network_id
        );
    }
}
