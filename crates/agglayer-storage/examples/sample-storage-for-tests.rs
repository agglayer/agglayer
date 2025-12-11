//! Sample storage for tests
//!
//! This tool takes a production RocksDB database and creates a database with
//! the same schema for testing purposes. It optionally randomly samples
//! a user-specified number of entries from each column family.

use std::path::{Path, PathBuf};

use clap::Parser;
use eyre::Context;
use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};
use rocksdb::{IteratorMode, Options, DB};
use tracing::{debug, info, instrument};

#[derive(Parser, Debug)]
#[command(name = "sample-storage-for-tests")]
#[command(about = "Sample a RocksDB database for testing purposes", long_about = None)]
struct Args {
    /// Path to the input database
    #[arg(short, long)]
    input: PathBuf,

    /// Path to the output database
    #[arg(short, long)]
    output: PathBuf,

    /// Number of entries to sample per column family
    #[arg(long, default_value = "0")]
    sample_size: usize,

    /// Random seed for deterministic sampling
    #[arg(long, default_value = "42")]
    seed: u64,
}

fn main() -> eyre::Result<()> {
    // Initialize tracing subscriber with environment filter
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    debug!("Logging initialized");

    let Args {
        input,
        output,
        sample_size,
        seed,
    } = Args::try_parse()?;

    run(input.as_path(), output.as_path(), sample_size, seed)
}

fn run(input: &Path, output: &Path, sample_size: usize, seed: u64) -> eyre::Result<()> {
    info!(?input, ?output, %sample_size, %seed, "Running...");

    // Initialize RNG with seed for deterministic sampling
    let mut rng = StdRng::seed_from_u64(seed);

    // List all column families in the input database
    let (options, cf_descriptors) = Options::load_latest(
        input,
        rocksdb::Env::new()?,
        true,
        rocksdb::Cache::new_lru_cache(256 * 1024),
    )?;
    let cf_names: Vec<_> = cf_descriptors
        .iter()
        .map(|d| d.name().to_string())
        .collect();
    info!("Found column families {cf_names:?}");

    // Open input database in read-only mode
    info!("Opening input database...");
    let input_db = DB::open_cf_for_read_only(&Options::default(), input, &cf_names, false)?;

    // Create output database with the same column families
    info!("Creating output database...");
    let output_db = {
        let mut output_opts = options;
        output_opts.create_if_missing(true);
        output_opts.create_missing_column_families(true);

        DB::open_cf_descriptors(&output_opts, output, cf_descriptors)?
    };

    // Process each column family
    for cf_name in &cf_names {
        info!("Processing column family: {cf_name:?}");
        sample_column_family(&input_db, &output_db, cf_name, sample_size, &mut rng)
            .with_context(|| format!("Processing column family {cf_name:?} failed"))?;
    }

    info!("Output database: {}", output.display());

    Ok(())
}

#[instrument(skip_all, fields(?cf_name))]
fn sample_column_family(
    input_db: &DB,
    output_db: &DB,
    cf_name: &str,
    sample_size: usize,
    rng: &mut StdRng,
) -> eyre::Result<()> {
    info!("Sampling column family {cf_name:?}");

    let cf_handle = input_db
        .cf_handle(cf_name)
        .ok_or_else(|| eyre::eyre!("Column family not found: {cf_name:?}"))?;

    let iter = input_db.iterator_cf(cf_handle, IteratorMode::Start);

    // Collect all keys from the column family
    let mut all_keys: Vec<Vec<u8>> = Vec::new();
    for item in iter {
        let (key, _value) = item?;
        all_keys.push(key.to_vec());
    }

    let total_entries = all_keys.len();
    debug!("Total entries: {}", total_entries);

    // Sample keys deterministically
    let sample_count = sample_size.min(total_entries);
    all_keys.shuffle(rng);
    let sampled_keys = &all_keys[..sample_count];

    // Copy sampled entries to output database
    for key in sampled_keys {
        let value = input_db
            .get_cf(cf_handle, key)?
            .ok_or_else(|| eyre::eyre!("Key not found {key:?}"))?;
        output_db.put_cf(cf_handle, key, value)?;
    }

    debug!("Completed");
    Ok(())
}
