use agglayer_interop_types::LocalExitRoot;
use agglayer_types::{CertificateId, CertificateStatus, Height, Metadata};
use criterion::{criterion_group, criterion_main, Criterion};
use unified_bridge::NetworkId;

fn bench_parse_certificate_header(c: &mut Criterion) {
    let bytes = agglayer_bincode::default()
        .serialize(&agglayer_types::CertificateHeader {
            network_id: NetworkId::ETH_L1,
            height: Height::default(),
            epoch_number: None,
            certificate_index: None,
            certificate_id: CertificateId::default(),
            prev_local_exit_root: LocalExitRoot::default(),
            new_local_exit_root: LocalExitRoot::default(),
            metadata: Metadata::default(),
            status: CertificateStatus::Pending,
            settlement_job_id: None,
            settlement_tx_hash: None,
        })
        .unwrap();
    c.bench_function("default cert header", |b| {
        b.iter(|| -> agglayer_types::CertificateHeader {
            agglayer_bincode::default()
                .deserialize(std::hint::black_box(&bytes))
                .unwrap()
        });
    });
}

criterion_group!(benches, bench_parse_certificate_header);
criterion_main!(benches);
