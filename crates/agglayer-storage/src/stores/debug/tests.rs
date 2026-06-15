use std::path::PathBuf;

use agglayer_types::{Certificate, CertificateId};
use pessimistic_proof_test_suite::sample_data;
use prost::Message as _;

use super::DebugStore;
use crate::{
    columns::debug_certificates::{DebugCertificatesColumn, DebugCertificatesProtoColumn},
    schema::{Codec as _, ColumnSchema as _},
    stores::{DebugReader as _, DebugWriter as _},
    tests::TempDBDir,
    types::generated::agglayer::storage::v0,
};

fn store() -> (TempDBDir, DebugStore) {
    let tmp = TempDBDir::new();
    let store = DebugStore::new_with_path(&tmp.path).unwrap();
    (tmp, store)
}

fn read_raw_legacy_certificate_bytes(store: &DebugStore, certificate_id: CertificateId) -> Vec<u8> {
    let DebugStore::Enabled(store) = store else {
        panic!("expected enabled debug store");
    };
    let key = certificate_id.encode().unwrap();
    let cf = store
        .db
        .raw_rocksdb()
        .cf_handle(DebugCertificatesColumn::COLUMN_FAMILY_NAME)
        .unwrap();

    store
        .db
        .raw_rocksdb()
        .get_cf(&cf, key)
        .unwrap()
        .unwrap()
        .to_vec()
}

fn read_raw_proto_certificate_bytes(store: &DebugStore, certificate_id: CertificateId) -> Vec<u8> {
    let DebugStore::Enabled(store) = store else {
        panic!("expected enabled debug store");
    };
    let key = certificate_id.encode().unwrap();
    let cf = store
        .db
        .raw_rocksdb()
        .cf_handle(DebugCertificatesProtoColumn::COLUMN_FAMILY_NAME)
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
    certificate_id: CertificateId,
    value: Vec<u8>,
) {
    let db = crate::storage::DB::open_cf(path, super::cf_definitions::DEBUG_DB_V0).unwrap();
    let key = certificate_id.encode().unwrap();
    let cf = db
        .raw_rocksdb()
        .cf_handle(DebugCertificatesColumn::COLUMN_FAMILY_NAME)
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
fn add_certificate_writes_proto_bytes() {
    let (_tmp, store) = store();
    let certificate = Certificate::new_for_test(
        agglayer_types::NetworkId::new(1),
        agglayer_types::Height::ZERO,
    );

    store.add_certificate(&certificate).unwrap();

    let raw = read_raw_proto_certificate_bytes(&store, certificate.hash());
    let proto = v0::Certificate::decode(raw.as_slice())
        .expect("debug certificates should be stored as storage proto");
    let decoded = Certificate::try_from(proto).unwrap();

    assert_eq!(decoded, certificate);
}

#[test]
fn get_certificate_does_not_read_legacy_v0_rows_after_open() {
    let (_tmp, store) = store();
    let expected = sample_data::load_certificate("n15-cert_h0.json");
    let certificate_id = expected.hash();
    let value = load_v0_certificate_bytes("v0-n15-cert_h0.hex");
    let DebugStore::Enabled(store_ref) = &store else {
        panic!("expected enabled debug store");
    };
    let cf = store_ref
        .db
        .raw_rocksdb()
        .cf_handle(DebugCertificatesColumn::COLUMN_FAMILY_NAME)
        .unwrap();

    // Manually drop a legacy row into the legacy CF after the migration step
    // already ran on this fresh store. Runtime debug reads must not fall back
    // to the legacy CF: the migration boundary owns the legacy → proto copy.
    store_ref
        .db
        .raw_rocksdb()
        .put_cf(&cf, certificate_id.encode().unwrap(), value)
        .unwrap();

    assert_eq!(store.get_certificate(&certificate_id).unwrap(), None);
}

#[test]
fn reopening_debug_store_migrates_legacy_rows_to_proto() {
    let tmp = TempDBDir::new();
    let expected = sample_data::load_certificate("n15-cert_h0.json");
    let certificate_id = expected.hash();
    let legacy = load_v0_certificate_bytes("v0-n15-cert_h0.hex");

    write_raw_certificate_bytes(&tmp.path, certificate_id, legacy.clone());

    let store = DebugStore::new_with_path(&tmp.path).unwrap();

    assert_eq!(
        store.get_certificate(&certificate_id).unwrap(),
        Some(expected.clone())
    );

    let raw = read_raw_proto_certificate_bytes(&store, certificate_id);
    let proto = v0::Certificate::decode(raw.as_slice())
        .expect("legacy debug rows should be copied to the proto CF on reopen");
    let decoded = Certificate::try_from(proto).unwrap();

    assert_eq!(decoded, expected);
    assert_eq!(
        read_raw_legacy_certificate_bytes(&store, certificate_id),
        legacy
    );
}

#[test]
fn reopening_debug_store_skips_unparsable_legacy_rows() {
    use agglayer_types::Digest;

    // The good row will round-trip into the proto CF; the corrupt row will
    // be skipped (logged via tracing::error inside the helper) and the
    // migration must still succeed for the good row.
    let tmp = TempDBDir::new();
    let expected = sample_data::load_certificate("n15-cert_h0.json");
    let good_id = expected.hash();
    let good_bytes = load_v0_certificate_bytes("v0-n15-cert_h0.hex");

    let bad_id = CertificateId::from(Digest([0xAB; 32]));
    // First byte != 0/1 enters the proto path; the rest is garbage that
    // prost rejects as a malformed message.
    let bad_bytes: Vec<u8> = vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF];

    write_raw_certificate_bytes(&tmp.path, good_id, good_bytes);
    write_raw_certificate_bytes(&tmp.path, bad_id, bad_bytes);

    // Migration completes despite the corrupt row.
    let store = DebugStore::new_with_path(&tmp.path).expect("migration should not abort");

    // Good row reaches the proto CF and is readable through the runtime
    // codec.
    assert_eq!(
        store.get_certificate(&good_id).unwrap(),
        Some(expected),
        "the good row should be migrated to the proto CF"
    );
    // Bad row is absent from the proto CF; runtime reads return None.
    assert_eq!(
        store.get_certificate(&bad_id).unwrap(),
        None,
        "the corrupt row should be skipped, not migrated"
    );
}

fn create_raw_debug_v0(path: &std::path::Path) {
    let mut options = rocksdb::Options::default();
    options.create_if_missing(true);
    options.create_missing_column_families(true);
    let descriptors = super::cf_definitions::DEBUG_DB_V0
        .iter()
        .map(|cf| rocksdb::ColumnFamilyDescriptor::new(cf.name(), rocksdb::Options::default()));
    let db = rocksdb::DB::open_cf_descriptors(&options, path, descriptors).unwrap();
    drop(db);
}

#[test]
fn migrated_or_create_debug_creates_missing_storage() {
    let tmp = TempDBDir::new();
    let path = tmp.path.join("debug");

    let db = DebugStore::open_migrated_or_create_db(&path).unwrap();
    drop(db);

    let cfs = rocksdb::DB::list_cf(&rocksdb::Options::default(), &path).unwrap();
    assert!(cfs.contains(&DebugCertificatesProtoColumn::COLUMN_FAMILY_NAME.to_string()));
}

#[test]
fn migrated_or_create_debug_rejects_legacy_storage_without_mutating_it() {
    let tmp = TempDBDir::new();
    create_raw_debug_v0(&tmp.path);

    let error = match DebugStore::open_migrated_or_create_db(&tmp.path) {
        Ok(_) => panic!("migrated-or-create open should reject legacy debug storage"),
        Err(error) => error,
    };

    assert!(matches!(
        error,
        crate::storage::DBOpenError::StorageNeedsMigration { .. }
    ));
    let cfs = rocksdb::DB::list_cf(&rocksdb::Options::default(), &tmp.path).unwrap();
    assert!(!cfs.contains(&DebugCertificatesProtoColumn::COLUMN_FAMILY_NAME.to_string()));
}

#[test]
fn migrated_or_create_debug_roundtrips_as_current() {
    // Guards this store's DECLARED_MIGRATION_STEPS against init_db's recorded
    // step count; see crate::tests::assert_storage_gate_roundtrips.
    crate::tests::assert_storage_gate_roundtrips(DebugStore::open_migrated_or_create_db);
}
