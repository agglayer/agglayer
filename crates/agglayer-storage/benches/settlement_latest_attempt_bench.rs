use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::{rngs::StdRng, seq::SliceRandom, Rng, SeedableRng};
use rocksdb::{ColumnFamily, ColumnFamilyDescriptor, Options, DB};

const ATTEMPTS_CF: &str = "bench_settlement_attempts_cf";
const LATEST_ATTEMPT_CF: &str = "bench_settlement_latest_attempt_cf";

const MAX_ATTEMPTS_PER_JOB: u64 = 25;
const RNG_SEED: u64 = 42;

#[derive(Clone, Copy, Debug)]
enum Strategy {
    PrefixScan,
    LatestCf,
}

impl Strategy {
    fn as_str(self) -> &'static str {
        match self {
            Strategy::PrefixScan => "strategy_prefix_scan",
            Strategy::LatestCf => "strategy_latest_cf",
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Workload {
    WriteOnly,
    WriteHeavy90_10,
    ReadOnlyLatestHit,
    ReadOnlyLatestMiss,
}

impl Workload {
    fn as_str(self) -> &'static str {
        match self {
            Workload::WriteOnly => "wl_write_only",
            Workload::WriteHeavy90_10 => "wl_write_heavy_90_10",
            Workload::ReadOnlyLatestHit => "wl_read_only_latest_hit",
            Workload::ReadOnlyLatestMiss => "wl_read_only_latest_miss",
        }
    }
}

#[derive(Clone)]
struct Dataset {
    name: String,
    insert_ops: Vec<(u64, u64)>,
    read_hit_jobs: Vec<u64>,
    read_miss_jobs: Vec<u64>,
}

fn benchmark_path(seed: u64) -> PathBuf {
    let mut path = std::env::temp_dir();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Unable to read wall clock");
    path.push(format!(
        "agglayer_bench_settlement_latest_attempt_{}_{}_{}",
        std::process::id(),
        now.as_nanos(),
        seed
    ));
    path
}

fn create_db(path: &Path) -> DB {
    let mut opts = Options::default();
    opts.create_if_missing(true);
    opts.create_missing_column_families(true);

    let descriptors = vec![
        ColumnFamilyDescriptor::new(ATTEMPTS_CF, Options::default()),
        ColumnFamilyDescriptor::new(LATEST_ATTEMPT_CF, Options::default()),
    ];

    DB::open_cf_descriptors(&opts, path, descriptors).expect("Unable to open benchmark RocksDB")
}

fn cf<'a>(db: &'a DB, name: &str) -> &'a ColumnFamily {
    db.cf_handle(name)
        .unwrap_or_else(|| panic!("Missing ColumnFamily '{name}'"))
}

fn encode_attempt_key(job_id: u64, attempt_seq: u64) -> [u8; 16] {
    let mut bytes = [0u8; 16];
    bytes[0..8].copy_from_slice(&job_id.to_be_bytes());
    bytes[8..16].copy_from_slice(&attempt_seq.to_be_bytes());
    bytes
}

fn decode_attempt_key(key: &[u8]) -> Option<(u64, u64)> {
    if key.len() != 16 {
        return None;
    }

    let mut job_id = [0u8; 8];
    let mut attempt_seq = [0u8; 8];
    job_id.copy_from_slice(&key[0..8]);
    attempt_seq.copy_from_slice(&key[8..16]);

    Some((u64::from_be_bytes(job_id), u64::from_be_bytes(attempt_seq)))
}

fn decode_u64(value: &[u8]) -> Option<u64> {
    if value.len() != 8 {
        return None;
    }
    let mut arr = [0u8; 8];
    arr.copy_from_slice(value);
    Some(u64::from_be_bytes(arr))
}

fn insert_attempt(db: &DB, strategy: Strategy, job_id: u64, attempt_seq: u64) {
    let attempts_cf = cf(db, ATTEMPTS_CF);
    let key = encode_attempt_key(job_id, attempt_seq);
    let value = attempt_seq.to_be_bytes();
    db.put_cf(attempts_cf, key, value)
        .expect("Unable to insert attempt");

    if matches!(strategy, Strategy::LatestCf) {
        let latest_cf = cf(db, LATEST_ATTEMPT_CF);
        db.put_cf(latest_cf, job_id.to_be_bytes(), attempt_seq.to_be_bytes())
            .expect("Unable to update latest-attempt index");
    }
}

fn read_latest_attempt_prefix_scan(db: &DB, job_id: u64) -> Option<u64> {
    let attempts_cf = cf(db, ATTEMPTS_CF);
    let mut iter = db.raw_iterator_cf(attempts_cf);
    let seek_key = encode_attempt_key(job_id, u64::MAX);
    iter.seek_for_prev(seek_key);

    if !iter.valid() {
        return None;
    }

    let key = iter.key()?;
    let (key_job_id, attempt_seq) = decode_attempt_key(key)?;
    if key_job_id == job_id {
        Some(attempt_seq)
    } else {
        None
    }
}

fn read_latest_attempt(db: &DB, strategy: Strategy, job_id: u64) -> Option<u64> {
    match strategy {
        Strategy::PrefixScan => read_latest_attempt_prefix_scan(db, job_id),
        Strategy::LatestCf => {
            let latest_cf = cf(db, LATEST_ATTEMPT_CF);
            let value = db
                .get_cf(latest_cf, job_id.to_be_bytes())
                .expect("Unable to read latest-attempt index")?;
            decode_u64(&value)
        }
    }
}

fn sample_attempt_count(rng: &mut StdRng) -> u64 {
    let bucket = rng.random_range(0..100u64);
    match bucket {
        0..=69 => rng.random_range(1..=3),
        70..=94 => rng.random_range(4..=10),
        _ => rng.random_range(11..=MAX_ATTEMPTS_PER_JOB),
    }
}

fn generate_dataset(name: &str, job_count: usize) -> Dataset {
    let mut rng = StdRng::seed_from_u64(RNG_SEED ^ (job_count as u64));
    let attempts_per_job: Vec<u64> = (0..job_count)
        .map(|_| sample_attempt_count(&mut rng))
        .collect();

    let mut insert_ops = Vec::new();
    for attempt_seq in 1..=MAX_ATTEMPTS_PER_JOB {
        for (job_id, max_attempts) in attempts_per_job.iter().enumerate() {
            if *max_attempts >= attempt_seq {
                insert_ops.push((job_id as u64, attempt_seq));
            }
        }
    }

    let mut read_hit_jobs: Vec<u64> = (0..job_count as u64).collect();
    read_hit_jobs.shuffle(&mut rng);
    if read_hit_jobs.len() > 10_000 {
        read_hit_jobs.truncate(10_000);
    }

    let miss_count = read_hit_jobs.len().max(1);
    let read_miss_jobs: Vec<u64> =
        (job_count as u64..job_count as u64 + miss_count as u64).collect();

    Dataset {
        name: name.to_string(),
        insert_ops,
        read_hit_jobs,
        read_miss_jobs,
    }
}

fn run_workload(dataset: &Dataset, strategy: Strategy, workload: Workload) -> Duration {
    let path = benchmark_path(dataset.insert_ops.len() as u64);
    fs::create_dir_all(&path).expect("Unable to create benchmark directory");

    let db = create_db(&path);
    let elapsed = match workload {
        Workload::WriteOnly => {
            let now = Instant::now();
            for (job_id, attempt_seq) in dataset.insert_ops.iter().copied() {
                insert_attempt(&db, strategy, job_id, attempt_seq);
            }
            now.elapsed()
        }
        Workload::WriteHeavy90_10 => {
            let mut read_cursor = 0usize;
            let now = Instant::now();
            for (i, (job_id, attempt_seq)) in dataset.insert_ops.iter().copied().enumerate() {
                insert_attempt(&db, strategy, job_id, attempt_seq);

                if i % 10 == 0 {
                    let read_job = dataset.read_hit_jobs[read_cursor % dataset.read_hit_jobs.len()];
                    let latest = read_latest_attempt(&db, strategy, read_job);
                    std::hint::black_box(latest);
                    read_cursor = read_cursor.wrapping_add(1);
                }
            }
            now.elapsed()
        }
        Workload::ReadOnlyLatestHit => {
            for (job_id, attempt_seq) in dataset.insert_ops.iter().copied() {
                insert_attempt(&db, strategy, job_id, attempt_seq);
            }

            let now = Instant::now();
            for job_id in dataset.read_hit_jobs.iter().copied() {
                let latest = read_latest_attempt(&db, strategy, job_id);
                std::hint::black_box(latest);
            }
            now.elapsed()
        }
        Workload::ReadOnlyLatestMiss => {
            for (job_id, attempt_seq) in dataset.insert_ops.iter().copied() {
                insert_attempt(&db, strategy, job_id, attempt_seq);
            }

            let now = Instant::now();
            for job_id in dataset.read_miss_jobs.iter().copied() {
                let latest = read_latest_attempt(&db, strategy, job_id);
                std::hint::black_box(latest);
            }
            now.elapsed()
        }
    };

    drop(db);
    fs::remove_dir_all(&path).expect("Unable to clean benchmark directory");
    elapsed
}

fn bench_dataset_workload(
    c: &mut Criterion,
    dataset: &Dataset,
    workload: Workload,
    strategy: Strategy,
) {
    let mut group = c.benchmark_group("settlement_latest");
    let id = BenchmarkId::new(
        format!(
            "{}/{}/{}",
            dataset.name,
            workload.as_str(),
            strategy.as_str()
        ),
        "",
    );

    group.bench_with_input(id, dataset, |b, ds| {
        b.iter_custom(|iters| {
            let mut total = Duration::ZERO;
            for _ in 0..iters {
                total += run_workload(ds, strategy, workload);
            }
            total
        });
    });

    group.finish();
}

fn bench_settlement_latest_attempt(c: &mut Criterion) {
    let datasets = [
        generate_dataset("ds_1k_realistic_1_25", 1_000),
        generate_dataset("ds_10k_realistic_1_25", 10_000),
        generate_dataset("ds_50k_realistic_1_25", 50_000),
    ];

    let strategies = [Strategy::PrefixScan, Strategy::LatestCf];
    let workloads = [
        Workload::WriteOnly,
        Workload::WriteHeavy90_10,
        Workload::ReadOnlyLatestHit,
        Workload::ReadOnlyLatestMiss,
    ];

    for dataset in datasets.iter() {
        for workload in workloads {
            for strategy in strategies {
                bench_dataset_workload(c, dataset, workload, strategy);
            }
        }
    }
}

criterion_group! {
    name = benches_settlement_latest_attempt;
    config = Criterion::default().sample_size(10);
    targets = bench_settlement_latest_attempt
}

criterion_main!(benches_settlement_latest_attempt);
