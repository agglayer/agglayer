use std::path::PathBuf;

use agglayer_types::{Certificate, Height, NetworkId};
use pessimistic_proof_test_suite::sample_data;
use prost::Message as _;

use super::PendingStore;
use crate::{
    columns::pending_queue::{PendingQueueColumn, PendingQueueKey, PendingQueueProtoColumn},
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
