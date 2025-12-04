//! CLI tool to generate RocksDB test databases for regression testing
//!
//! This tool generates populated RocksDB databases with some mock data
//! that can be used as artifacts for regression testing across
//! version upgrades (e.g., alloy 0.14 -> 1.0).
//!
//! Usage:
//!   cargo run --bin generate-test-db --features testutils -- --output-dir
//! ./test-artifacts
//!
//! The tool will create:
//! - Four database directories (state, pending, epochs, debug)
//! - A metadata.json file describing the generated data
//! - Optionally, a compressed tarball of all databases

use std::{
    fs,
    path::{Path, PathBuf},
};

use agglayer_storage::tests::db_generator::{
    generate_all_databases, DatabaseMetadata, DatabasePaths, GenerationStatistics, GeneratorConfig,
};
use clap::Parser;

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
    #[arg(short, long, default_value_t = 10)]
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("🚀 Generating RocksDB test databases...");
    println!("Output directory: {}", args.output_dir.display());
    println!("Configuration:");
    println!("  - Networks: {}", args.num_networks);
    println!(
        "  - Certificates per network: {}",
        args.certificates_per_network
    );
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
    println!(
        "  - Column families populated: {}",
        result.entries_per_cf.len()
    );
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
        config: config.clone(),
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

fn create_tarball(db_dir: &Path, tarball_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    use flate2::{write::GzEncoder, Compression};
    use tar::Builder;

    // Get the parent directory and the db directory name
    let parent_dir = db_dir
        .parent()
        .ok_or("Cannot get parent directory")?
        .to_path_buf();

    // Compute the expected directory name from tarball_name (remove .tar.gz)
    let expected_dir_name = tarball_name
        .trim_end_matches(".tar.gz")
        .trim_end_matches(".tar");

    // Create tarball in the parent directory
    let tarball_path = parent_dir.join(tarball_name);

    // Create the compressed tar archive
    let tar_file = fs::File::create(&tarball_path)?;
    let encoder = GzEncoder::new(tar_file, Compression::default());
    let mut archive = Builder::new(encoder);

    // Add the directory to the archive with the expected name
    // This recursively adds all files and subdirectories
    archive.append_dir_all(expected_dir_name, db_dir)?;

    // Finish writing the archive
    archive.finish()?;

    // Print tarball size
    let metadata = fs::metadata(&tarball_path)?;
    let size_kb = metadata.len() as f64 / 1024.0;
    println!("  Tarball size: {:.2} KB", size_kb);
    println!("  Location: {}", tarball_path.display());

    Ok(())
}
