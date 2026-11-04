use std::{
    ops::Add,
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use agglayer_storage::{
    backup::BackupClient,
    stores::{state::StateStore, NetworkSettlement, StateReader as _, StateWriter as _},
};
use agglayer_types::{CertificateId, CertificateIndex, Digest, EpochNumber, Height, NetworkType};
use criterion::{criterion_group, criterion_main, Criterion};
use rand::Rng;

fn bench_latest_certificate(c: &mut Criterion) {
    let mut path = std::env::temp_dir();

    let folder_name = "bench_latest_certificate";
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Failed to get time since epoch");

    let mut rng = rand::rng();

    path.push(format!(
        "{}/{}_{}",
        folder_name,
        time.as_nanos(),
        rng.random::<u64>()
    ));

    std::fs::create_dir_all(path.clone()).expect("Failed to create temp dir");

    let dir_path = path;

    fn run(dir_path: std::path::PathBuf, expected: u32) -> Duration {
        std::fs::remove_dir_all(dir_path.clone()).unwrap();

        let db = Arc::new(StateStore::init_db(dir_path.as_path(), None).unwrap());
        let store = StateStore::new(db.clone(), BackupClient::noop());

        for i in 1..=expected {
            store
                .set_latest_settled_certificate_for_network(
                    &i.into(),
                    &NetworkSettlement {
                        certificate_id: CertificateId::new([0; 32].into()),
                        height: Height::ZERO,
                        epoch_number: EpochNumber::ZERO,
                        certificate_index: CertificateIndex::new(u64::from(i - 1)),
                        new_pp_root: Digest::ZERO,
                        new_local_exit_root: Digest::ZERO.into(),
                        settled_let_leaf_count: 0,
                        network_type: NetworkType::Ecdsa,
                        settled_claim: None,
                    },
                )
                .expect("Unable to put certificate into storage");
        }
        let now = Instant::now();
        let iterator = store.get_active_networks().expect("Unable to get keys");
        let elapsed = now.elapsed();

        assert_eq!(iterator.len(), expected as usize);

        elapsed
    }

    c.bench_function(
        "last_certificate_per_network iterator with small set",
        |b| {
            b.iter_custom(|iters| {
                let mut total = Duration::new(0, 0);

                (0..iters).for_each(|_| {
                    let elapsed = run(dir_path.clone(), 10);

                    total = total.add(elapsed);
                });

                total
            });
        },
    );

    c.bench_function(
        "last_certificate_per_network iterator with medium set",
        |b| {
            b.iter_custom(|iters| {
                let mut total = Duration::new(0, 0);

                (0..iters).for_each(|_| {
                    let elapsed = run(dir_path.clone(), 100);
                    total = total.add(elapsed);
                });

                total
            });
        },
    );

    c.bench_function(
        "last_certificate_per_network iterator with large set",
        |b| {
            b.iter_custom(|iters| {
                let mut total = Duration::new(0, 0);

                (0..iters).for_each(|_| {
                    let elapsed = run(dir_path.clone(), 10_000);
                    total = total.add(elapsed);
                });

                total
            });
        },
    );

    c.bench_function(
        "last_certificate_per_network iterator with very large set",
        |b| {
            b.iter_custom(|iters| {
                let mut total = Duration::new(0, 0);

                (0..iters).for_each(|_| {
                    let elapsed = run(dir_path.clone(), 100_000);
                    total = total.add(elapsed);
                });

                total
            });
        },
    );
}

criterion_group! {
  name = benches_latest_certificate;
  config = Criterion::default().sample_size(10);
  targets = bench_latest_certificate
}

criterion_main!(benches_latest_certificate);
