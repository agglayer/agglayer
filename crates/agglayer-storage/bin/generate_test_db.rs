//! CLI tool to generate RocksDB test databases for regression testing
//!
//! This tool generates populated RocksDB databases that can be used as artifacts
//! for regression testing across version upgrades (e.g., alloy 0.14 -> 1.0).
//!
//! Usage:
//!   cargo run --bin generate-test-db --features testutils -- --output-dir ./test-artifacts
//!
//! The tool will create:
//! - Four database directories (state, pending, epochs, debug)
//! - A metadata.json file describing the generated data
//! - Optionally, a compressed tarball of all databases

use std::{fs, path::PathBuf};

use clap::Parser;
use serde::{Deserialize, Serialize};

use agglayer_storage::tests::db_generator::{generate_all_databases, GeneratorConfig};

#[derive(Parser, Debug)]
#[command(
    name = "generate-test-db",
    about = "Generate RocksDB test databases for regression testing",
    version
)]
struct Args {
    /// Output directory where databases will be generated
    #[arg(short, long, default_value = "./test-dbs")]
    output_dir: PathBuf,

    /// Number of different networks to generate data for
    #[arg(short, long, default_value_t = 3)]
    num_networks: u32,

    /// Number of certificates per network
    #[arg(short, long, default_value_t = 5)]
    certificates_per_network: u64,

    /// Generate proofs for certificates (slower but more complete)
    #[arg(short, long, default_value_t = true)]
    generate_proofs: bool,

    /// Random seed for reproducibility
    #[arg(short, long, default_value_t = 42)]
    seed: u64,

    /// Create a compressed tarball of the databases
    #[arg(short, long, default_value_t = false)]
    tarball: bool,

    /// Name for the tarball (only used if --tarball is set)
    #[arg(long, default_value = "reference_db_v1.tar.gz")]
    tarball_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct DatabaseMetadata {
    /// Version identifier for this database artifact
    version: String,
    /// Generation timestamp
    timestamp: String,
    /// Configuration used to generate the databases
    config: GeneratorConfigMetadata,
    /// Statistics about generated data
    statistics: GenerationStatistics,
    /// Database paths relative to the metadata file
    database_paths: DatabasePaths,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeneratorConfigMetadata {
    num_networks: u32,
    certificates_per_network: u64,
    generate_proofs: bool,
    seed: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct GenerationStatistics {
    total_networks: usize,
    total_certificates: usize,
    entries_per_column_family: std::collections::HashMap<String, usize>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DatabasePaths {
    state: String,
    pending: String,
    epochs: String,
    debug: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("🚀 Generating RocksDB test databases...");
    println!("Output directory: {}", args.output_dir.display());
    println!("Configuration:");
    println!("  - Networks: {}", args.num_networks);
    println!("  - Certificates per network: {}", args.certificates_per_network);
    println!("  - Generate proofs: {}", args.generate_proofs);
    println!("  - Seed: {}", args.seed);
    println!();

    // Create output directory
    fs::create_dir_all(&args.output_dir)?;

    // Configure generator
    let config = GeneratorConfig {
        num_networks: args.num_networks,
        certificates_per_network: args.certificates_per_network,
        generate_proofs: args.generate_proofs,
        seed: args.seed,
    };

    // Generate databases
    println!("📝 Generating databases...");
    let result = generate_all_databases(&args.output_dir, &config)?;

    println!("✅ Database generation complete!");
    println!();
    println!("Statistics:");
    println!("  - Networks generated: {}", result.network_ids.len());
    println!(
        "  - Certificates generated: {}",
        result.certificate_ids.len()
    );
    println!("  - Column families populated: {}", result.entries_per_cf.len());
    println!();
    println!("Entries per column family:");
    for (cf, count) in &result.entries_per_cf {
        println!("  - {}: {} entries", cf, count);
    }
    println!();

    // Create metadata file
    let metadata = DatabaseMetadata {
        version: "1.0.0".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        config: GeneratorConfigMetadata {
            num_networks: args.num_networks,
            certificates_per_network: args.certificates_per_network,
            generate_proofs: args.generate_proofs,
            seed: args.seed,
        },
        statistics: GenerationStatistics {
            total_networks: result.network_ids.len(),
            total_certificates: result.certificate_ids.len(),
            entries_per_column_family: result.entries_per_cf.clone(),
        },
        database_paths: DatabasePaths {
            state: "state".to_string(),
            pending: "pending".to_string(),
            epochs: "epochs".to_string(),
            debug: "debug".to_string(),
        },
    };

    let metadata_path = args.output_dir.join("metadata.json");
    let metadata_json = serde_json::to_string_pretty(&metadata)?;
    fs::write(&metadata_path, metadata_json)?;
    println!("📄 Metadata written to: {}", metadata_path.display());

    // Create tarball if requested
    if args.tarball {
        println!();
        println!("📦 Creating tarball...");
        create_tarball(&args.output_dir, &args.tarball_name)?;
        println!("✅ Tarball created: {}", args.tarball_name);
    }

    println!();
    println!("🎉 All done! Databases are ready for use in regression tests.");

    Ok(())
}

fn create_tarball(db_dir: &PathBuf, tarball_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::process::Command;

    // Get the parent directory and the db directory name
    let parent_dir = db_dir
        .parent()
        .ok_or("Cannot get parent directory")?
        .to_path_buf();
    let db_name = db_dir
        .file_name()
        .ok_or("Cannot get directory name")?
        .to_str()
        .ok_or("Invalid directory name")?;

    // Create tarball in the parent directory
    let tarball_path = parent_dir.join(tarball_name);

    // Use tar command to create compressed archive
    let output = Command::new("tar")
        .arg("-czf")
        .arg(&tarball_path)
        .arg("-C")
        .arg(&parent_dir)
        .arg(db_name)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to create tarball: {}", stderr).into());
    }

    // Print tarball size
    let metadata = fs::metadata(&tarball_path)?;
    let size_mb = metadata.len() as f64 / (1024.0 * 1024.0);
    println!("  Tarball size: {:.2} MB", size_mb);
    println!("  Location: {}", tarball_path.display());

    Ok(())
}

