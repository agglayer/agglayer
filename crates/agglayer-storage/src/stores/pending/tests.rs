use std::path::PathBuf;

use agglayer_types::{Certificate, CertificateId, Height, NetworkId, Proof};
use pessimistic_proof_test_suite::sample_data;
use prost::Message as _;

use super::PendingStore;
use crate::{
    columns::{
        pending_queue::{PendingQueueColumn, PendingQueueKey, PendingQueueProtoColumn},
        proof_per_certificate::ProofPerCertificateColumn,
    },
    error::Error,
    schema::{Codec as _, ColumnSchema as _},
    stores::{PendingCertificateReader as _, PendingCertificateWriter as _},
    tests::TempDBDir,
    types::generated::agglayer::storage::v0,
};

fn store() -> (TempDBDir, PendingStore) {
    let tmp = TempDBDir::new();
    let store = PendingStore::new_with_path(&tmp.path).unwrap();
    (tmp, store)
}

fn read_raw_legacy_certificate_bytes(
    store: &PendingStore,
    network_id: NetworkId,
    height: Height,
) -> Vec<u8> {
    let key = PendingQueueKey(network_id, height).encode().unwrap();
    let cf = store
        .db
        .raw_rocksdb()
        .cf_handle(PendingQueueColumn::COLUMN_FAMILY_NAME)
        .unwrap();

    store
        .db
        .raw_rocksdb()
        .get_cf(&cf, key)
        .unwrap()
        .unwrap()
        .to_vec()
}

fn read_raw_proto_certificate_bytes(
    store: &PendingStore,
    network_id: NetworkId,
    height: Height,
) -> Vec<u8> {
    let key = PendingQueueKey(network_id, height).encode().unwrap();
    let cf = store
        .db
        .raw_rocksdb()
        .cf_handle(PendingQueueProtoColumn::COLUMN_FAMILY_NAME)
        .unwrap();

    store
        .db
        .raw_rocksdb()
        .get_cf(&cf, key)
        .unwrap()
        .unwrap()
        .to_vec()
}

fn write_raw_certificate_bytes(
    path: &std::path::Path,
    network_id: NetworkId,
    height: Height,
    value: Vec<u8>,
) {
    let db = crate::storage::DB::open_cf(path, super::cf_definitions::PENDING_DB_V0).unwrap();
    let key = PendingQueueKey(network_id, height).encode().unwrap();
    let cf = db
        .raw_rocksdb()
        .cf_handle(PendingQueueColumn::COLUMN_FAMILY_NAME)
        .unwrap();

    db.raw_rocksdb().put_cf(&cf, key, value).unwrap();
}

fn write_raw_proof_bytes(store: &PendingStore, certificate_id: CertificateId, value: Vec<u8>) {
    let key = certificate_id.encode().unwrap();
    let cf = store
        .db
        .raw_rocksdb()
        .cf_handle(ProofPerCertificateColumn::COLUMN_FAMILY_NAME)
        .unwrap();

    store.db.raw_rocksdb().put_cf(&cf, key, value).unwrap();
}

fn load_v0_certificate_bytes(name: &str) -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src/types/certificate/tests/encoded")
        .join(name);
    let hex = std::fs::read_to_string(path).unwrap();
    hex::decode(hex.trim()).unwrap()
}

#[test]
fn insert_pending_certificate_writes_proto_bytes() {
    let (_tmp, store) = store();
    let certificate = Certificate::new_for_test(NetworkId::new(1), Height::ZERO);

    store
        .insert_pending_certificate(certificate.network_id, certificate.height, &certificate)
        .unwrap();

    let raw = read_raw_proto_certificate_bytes(&store, certificate.network_id, certificate.height);
    let proto = v0::Certificate::decode(raw.as_slice())
        .expect("pending certificates should be stored as storage proto");
    let decoded = Certificate::try_from(proto).unwrap();

    assert_eq!(decoded, certificate);
}

#[test]
fn get_certificate_does_not_read_legacy_v0_rows_after_open() {
    let (_tmp, store) = store();
    let network_id = NetworkId::new(15);
    let height = Height::ZERO;
    let key = PendingQueueKey(network_id, height).encode().unwrap();
    let value = load_v0_certificate_bytes("v0-n15-cert_h0.hex");
    let cf = store
        .db
        .raw_rocksdb()
        .cf_handle(PendingQueueColumn::COLUMN_FAMILY_NAME)
        .unwrap();

    store.db.raw_rocksdb().put_cf(&cf, key, value).unwrap();

    assert_eq!(store.get_certificate(network_id, height).unwrap(), None);
}

#[test]
fn reopening_pending_store_migrates_legacy_rows_to_proto() {
    let tmp = TempDBDir::new();
    let network_id = NetworkId::new(15);
    let height = Height::ZERO;
    let expected = sample_data::load_certificate("n15-cert_h0.json");
    let legacy = load_v0_certificate_bytes("v0-n15-cert_h0.hex");

    write_raw_certificate_bytes(&tmp.path, network_id, height, legacy.clone());

    let store = PendingStore::new_with_path(&tmp.path).unwrap();

    assert_eq!(
        store.get_certificate(network_id, height).unwrap(),
        Some(expected.clone())
    );

    let raw = read_raw_proto_certificate_bytes(&store, network_id, height);
    let proto = v0::Certificate::decode(raw.as_slice())
        .expect("legacy pending rows should be copied to the proto CF on reopen");
    let decoded = Certificate::try_from(proto).unwrap();

    assert_eq!(decoded, expected);
    assert_eq!(
        read_raw_legacy_certificate_bytes(&store, network_id, height),
        legacy
    );
}

#[test]
fn get_proof_reports_unreadable_pending_proof() {
    let (_tmp, store) = store();
    let certificate_id = CertificateId::new([7; 32].into());

    write_raw_proof_bytes(&store, certificate_id, b"not a proof".to_vec());

    let err = store.get_proof(certificate_id).unwrap_err();

    assert!(matches!(err, Error::UnreadableProof { id, .. } if id == certificate_id));
}

#[test]
fn multi_get_proof_reports_unreadable_pending_proofs() {
    let (_tmp, store) = store();
    let valid_id = CertificateId::new([8; 32].into());
    let invalid_id = CertificateId::new([9; 32].into());

    store
        .insert_generated_proof(&valid_id, &Proof::dummy())
        .unwrap();
    write_raw_proof_bytes(&store, invalid_id, b"not a proof".to_vec());

    let err = store.multi_get_proof(&[valid_id, invalid_id]).unwrap_err();

    assert!(matches!(err, Error::UnreadableProof { id, .. } if id == invalid_id));
}

#[test]
fn get_current_pending_heights_returns_all_networks() {
    use std::collections::BTreeMap;

    use crate::columns::latest_pending_certificate_per_network::PendingCertificate;

    let (_tmp, store) = store();

    assert_eq!(store.get_current_pending_heights().unwrap(), vec![]);

    let network_1 = NetworkId::new(1);
    let network_15 = NetworkId::new(15);
    let certificate_1 = Certificate::new_for_test(network_1, Height::ZERO);
    let certificate_15 = Certificate::new_for_test(network_15, Height::new(3));

    store
        .insert_pending_certificate(network_1, Height::ZERO, &certificate_1)
        .unwrap();
    store
        .insert_pending_certificate(network_15, Height::new(3), &certificate_15)
        .unwrap();

    let heights: BTreeMap<_, _> = store
        .get_current_pending_heights()
        .unwrap()
        .into_iter()
        .collect();

    assert_eq!(heights.len(), 2);
    assert_eq!(
        heights[&network_1],
        PendingCertificate(certificate_1.hash(), Height::ZERO)
    );
    assert_eq!(
        heights[&network_15],
        PendingCertificate(certificate_15.hash(), Height::new(3))
    );

    // Inserting a newer pending certificate for the same network overwrites
    // the latest pointer instead of adding a new entry.
    let certificate_1_next = Certificate::new_for_test(network_1, Height::new(1));
    store
        .insert_pending_certificate(network_1, Height::new(1), &certificate_1_next)
        .unwrap();

    let heights: BTreeMap<_, _> = store
        .get_current_pending_heights()
        .unwrap()
        .into_iter()
        .collect();

    assert_eq!(heights.len(), 2);
    assert_eq!(
        heights[&network_1],
        PendingCertificate(certificate_1_next.hash(), Height::new(1))
    );
    assert_eq!(
        heights[&network_15],
        PendingCertificate(certificate_15.hash(), Height::new(3))
    );
}
