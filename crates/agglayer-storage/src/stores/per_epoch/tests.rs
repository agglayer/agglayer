use std::{
    collections::{BTreeMap, VecDeque},
    path::PathBuf,
    sync::Arc,
};

use agglayer_config::Config;
use agglayer_types::{
    Certificate, CertificateIndex, CertificateStatus, EpochNumber, Height, NetworkId, Proof,
};
use parking_lot::RwLock;
use pessimistic_proof_test_suite::sample_data;
use prost::Message as _;
use rstest::{fixture, rstest};
use tracing::info;

use crate::{
    backup::BackupClient,
    columns::epochs::{
        certificates::{CertificatePerIndexColumn, CertificatePerIndexProtoColumn},
        end_checkpoint::EndCheckpointColumn,
        start_checkpoint::StartCheckpointColumn,
    },
    error::Error,
    schema::{Codec as _, ColumnSchema as _},
    stores::{
        epochs::EpochsStore,
        interfaces::writer::{EpochStoreWriter as _, PerEpochWriter, StateWriter},
        pending::PendingStore,
        per_epoch::PerEpochStore,
        state::StateStore,
        EpochStoreReader as _, PendingCertificateWriter as _, PerEpochReader as _, StateReader,
    },
    tests::TempDBDir,
    types::generated::agglayer::storage::v0,
};

#[fixture]
fn store() -> PerEpochStore<PendingStore, StateStore> {
    let tmp = TempDBDir::new();
    let config = Arc::new(Config::new(&tmp.path));
    let pending_store =
        Arc::new(PendingStore::new_with_path(&config.storage.pending_db_path).unwrap());
    let state_store = Arc::new(
        StateStore::new_with_path(&config.storage.state_db_path, BackupClient::noop()).unwrap(),
    );

    let backup_client = BackupClient::noop();
    PerEpochStore::try_open(
        config,
        EpochNumber::ZERO,
        pending_store,
        state_store,
        None,
        backup_client,
    )
    .unwrap()
}

fn read_raw_legacy_epoch_certificate_bytes(
    store: &PerEpochStore<PendingStore, StateStore>,
    index: CertificateIndex,
) -> Vec<u8> {
    let key = index.encode().unwrap();
    let cf = store
        .db
        .raw_rocksdb()
        .cf_handle(CertificatePerIndexColumn::COLUMN_FAMILY_NAME)
        .unwrap();

    store
        .db
        .raw_rocksdb()
        .get_cf(&cf, key)
        .unwrap()
        .unwrap()
        .to_vec()
}

fn read_raw_proto_epoch_certificate_bytes(
    store: &PerEpochStore<PendingStore, StateStore>,
    index: CertificateIndex,
) -> Vec<u8> {
    let key = index.encode().unwrap();
    let cf = store
        .db
        .raw_rocksdb()
        .cf_handle(CertificatePerIndexProtoColumn::COLUMN_FAMILY_NAME)
        .unwrap();

    store
        .db
        .raw_rocksdb()
        .get_cf(&cf, key)
        .unwrap()
        .unwrap()
        .to_vec()
}

fn load_v0_certificate_bytes(name: &str) -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src/types/certificate/tests/encoded")
        .join(name);
    let hex = std::fs::read_to_string(path).unwrap();
    hex::decode(hex.trim()).unwrap()
}

fn write_raw_epoch_certificate_bytes(
    path: &std::path::Path,
    index: CertificateIndex,
    value: Vec<u8>,
) {
    let db = crate::storage::DB::open_cf(path, super::cf_definitions::EPOCHS_DB_V0).unwrap();
    let key = index.encode().unwrap();
    let cf = db
        .raw_rocksdb()
        .cf_handle(CertificatePerIndexColumn::COLUMN_FAMILY_NAME)
        .unwrap();

    db.raw_rocksdb().put_cf(&cf, key, value).unwrap();
}

fn seed_epoch_checkpoints(path: &std::path::Path, network_id: NetworkId, height: Height) {
    let db = crate::storage::DB::open_cf(path, super::cf_definitions::EPOCHS_DB_V0).unwrap();
    db.put::<StartCheckpointColumn>(&network_id, &height)
        .unwrap();
    db.put::<EndCheckpointColumn>(&network_id, &height).unwrap();
}

#[rstest]
fn add_certificate_writes_proto_bytes_to_epoch_store(
    store: PerEpochStore<PendingStore, StateStore>,
) {
    let network = NetworkId::new(1);
    let height = Height::ZERO;
    let certificate = Certificate::new_for_test(network, height);
    let certificate_id = certificate.hash();
    let pending_store = store.pending_store.clone();
    let state_store = store.state_store.clone();

    state_store
        .insert_certificate_header(&certificate, CertificateStatus::Proven)
        .unwrap();

    pending_store
        .insert_pending_certificate(network, height, &certificate)
        .unwrap();

    pending_store
        .insert_generated_proof(&certificate_id, &Proof::dummy())
        .unwrap();

    let (_, index) = store
        .add_certificate(certificate_id, agglayer_types::ExecutionMode::Default)
        .unwrap();

    let raw = read_raw_proto_epoch_certificate_bytes(&store, index);
    let proto = v0::Certificate::decode(raw.as_slice())
        .expect("epoch certificates should be stored as storage proto");
    let decoded = Certificate::try_from(proto).unwrap();

    assert_eq!(decoded, certificate);
}

#[rstest]
fn get_certificate_at_index_does_not_read_legacy_v0_rows_after_open(
    store: PerEpochStore<PendingStore, StateStore>,
) {
    let index = CertificateIndex::ZERO;
    let key = index.encode().unwrap();
    let value = load_v0_certificate_bytes("v0-n15-cert_h0.hex");
    let cf = store
        .db
        .raw_rocksdb()
        .cf_handle(CertificatePerIndexColumn::COLUMN_FAMILY_NAME)
        .unwrap();

    store.db.raw_rocksdb().put_cf(&cf, key, value).unwrap();

    assert_eq!(store.get_certificate_at_index(index).unwrap(), None);
}

#[test]
fn reopening_epoch_store_migrates_legacy_certificate_rows_to_proto() {
    let tmp = TempDBDir::new();
    let config = Arc::new(Config::new(&tmp.path));
    let epoch_number = EpochNumber::ZERO;
    let epoch_path = config.storage.epoch_db_path(epoch_number);
    let pending_store =
        Arc::new(PendingStore::new_with_path(&config.storage.pending_db_path).unwrap());
    let state_store = Arc::new(
        StateStore::new_with_path(&config.storage.state_db_path, BackupClient::noop()).unwrap(),
    );
    let expected = sample_data::load_certificate("n15-cert_h0.json");
    let legacy = load_v0_certificate_bytes("v0-n15-cert_h0.hex");
    let index = CertificateIndex::ZERO;
    let network_id = NetworkId::new(15);
    let height = Height::ZERO;

    seed_epoch_checkpoints(&epoch_path, network_id, height);
    write_raw_epoch_certificate_bytes(&epoch_path, index, legacy.clone());

    let db = Arc::new(PerEpochStore::<PendingStore, StateStore>::init_db(&epoch_path).unwrap());
    let store = PerEpochStore::try_open_with_db(
        db,
        epoch_number,
        pending_store,
        state_store,
        Some(std::iter::once((network_id, height)).collect()),
        BackupClient::noop(),
        false,
    )
    .unwrap();

    assert_eq!(
        store.get_certificate_at_index(index).unwrap(),
        Some(expected.clone())
    );

    let raw = read_raw_proto_epoch_certificate_bytes(&store, index);
    let proto = v0::Certificate::decode(raw.as_slice())
        .expect("legacy epoch certificate rows should be copied to the proto CF on reopen");
    let decoded = Certificate::try_from(proto).unwrap();

    assert_eq!(decoded, expected);
    assert_eq!(
        read_raw_legacy_epoch_certificate_bytes(&store, index),
        legacy
    );
}

#[test]
fn readonly_epoch_access_rejects_legacy_schema_when_proto_cf_absent() {
    let tmp = TempDBDir::new();
    let config = Arc::new(Config::new(&tmp.path));
    let epoch_number = EpochNumber::ZERO;
    let epoch_path = config.storage.epoch_db_path(epoch_number);
    let pending_store =
        Arc::new(PendingStore::new_with_path(&config.storage.pending_db_path).unwrap());
    let state_store = Arc::new(
        StateStore::new_with_path(&config.storage.state_db_path, BackupClient::noop()).unwrap(),
    );
    let epochs_store =
        EpochsStore::new(config, pending_store, state_store, BackupClient::noop()).unwrap();
    let legacy = load_v0_certificate_bytes("v0-n15-cert_h0.hex");
    let index = CertificateIndex::ZERO;

    write_raw_epoch_certificate_bytes(&epoch_path, index, legacy);

    let error = epochs_store
        .get_certificate(epoch_number, index)
        .expect_err("readonly epoch access should require migration for legacy storage");

    assert!(matches!(
        error,
        Error::DBOpenError(crate::storage::DBOpenError::StorageNeedsMigration { .. })
    ));

    let cfs = rocksdb::DB::list_cf(&rocksdb::Options::default(), &epoch_path).unwrap();
    assert!(!cfs.contains(&CertificatePerIndexProtoColumn::COLUMN_FAMILY_NAME.to_string()));
}

fn create_raw_epoch_v0(path: &std::path::Path) {
    let mut options = rocksdb::Options::default();
    options.create_if_missing(true);
    options.create_missing_column_families(true);
    let descriptors = super::cf_definitions::EPOCHS_DB_V0
        .iter()
        .map(|cf| rocksdb::ColumnFamilyDescriptor::new(cf.name(), rocksdb::Options::default()));
    let db = rocksdb::DB::open_cf_descriptors(&options, path, descriptors).unwrap();
    drop(db);
}

#[test]
fn migrated_or_create_epoch_creates_missing_storage() {
    let tmp = TempDBDir::new();
    let path = tmp.path.join("epoch-7");

    let db = PerEpochStore::<PendingStore, StateStore>::open_migrated_or_create_db(&path).unwrap();
    drop(db);

    let cfs = rocksdb::DB::list_cf(&rocksdb::Options::default(), &path).unwrap();
    assert!(cfs.contains(&CertificatePerIndexProtoColumn::COLUMN_FAMILY_NAME.to_string()));
}

#[test]
fn epochs_store_open_creates_missing_epoch_with_current_schema() {
    let tmp = TempDBDir::new();
    let config = Arc::new(Config::new(&tmp.path));
    let epoch_number = EpochNumber::ZERO;
    let epoch_path = config.storage.epoch_db_path(epoch_number);
    let pending_store =
        Arc::new(PendingStore::open_migrated_or_create(&config.storage.pending_db_path).unwrap());
    let state_store = Arc::new(
        StateStore::open_migrated_or_create(&config.storage.state_db_path, BackupClient::noop())
            .unwrap(),
    );
    let epochs_store =
        EpochsStore::new(config, pending_store, state_store, BackupClient::noop()).unwrap();

    assert!(!epoch_path.exists());

    let epoch_store = epochs_store.open(epoch_number).unwrap();
    drop(epoch_store);

    let cfs = rocksdb::DB::list_cf(&rocksdb::Options::default(), &epoch_path).unwrap();
    assert!(cfs.contains(&CertificatePerIndexProtoColumn::COLUMN_FAMILY_NAME.to_string()));
}

#[test]
fn migrated_or_create_epoch_rejects_legacy_storage_without_mutating_it() {
    let tmp = TempDBDir::new();
    create_raw_epoch_v0(&tmp.path);

    let error =
        match PerEpochStore::<PendingStore, StateStore>::open_migrated_or_create_db(&tmp.path) {
            Ok(_) => panic!("migrated-or-create open should reject legacy epoch storage"),
            Err(error) => error,
        };

    assert!(matches!(
        error,
        crate::storage::DBOpenError::StorageNeedsMigration { .. }
    ));
    let cfs = rocksdb::DB::list_cf(&rocksdb::Options::default(), &tmp.path).unwrap();
    assert!(!cfs.contains(&CertificatePerIndexProtoColumn::COLUMN_FAMILY_NAME.to_string()));
}

#[test]
fn migrated_or_create_epoch_roundtrips_as_current() {
    // Guards this store's DECLARED_MIGRATION_STEPS against init_db's recorded
    // step count; see crate::tests::assert_storage_gate_roundtrips.
    crate::tests::assert_storage_gate_roundtrips(
        PerEpochStore::<PendingStore, StateStore>::open_migrated_or_create_db,
    );
}

#[test]
fn migrated_readonly_epoch_reports_inspection_failure_for_unreadable_storage() {
    // The read-only gate must report an inspection failure distinctly, like the
    // read-write gate, rather than collapsing it into StorageNeedsMigration.
    let tmp = TempDBDir::new();
    let path = tmp.path.join("not-rocksdb");
    std::fs::write(&path, b"not a rocksdb directory").unwrap();

    let error = match PerEpochStore::<PendingStore, StateStore>::open_migrated_readonly_db(&path) {
        Ok(_) => panic!("unreadable storage must not be opened read-only"),
        Err(error) => error,
    };

    assert!(
        matches!(
            error,
            crate::storage::DBOpenError::StorageInspectionFailed { .. }
        ),
        "expected StorageInspectionFailed, got {error:?}"
    );
}

#[rstest]
fn can_start_packing_an_unpacked_epoch(store: PerEpochStore<PendingStore, StateStore>) {
    assert!(store.start_packing().is_ok());
}

#[rstest]
fn cant_start_packing_a_packed_epoch(store: PerEpochStore<PendingStore, StateStore>) {
    let mut lock = store.packing_lock.write();
    *lock = true;

    drop(lock);

    assert!(store.start_packing().is_err());
}

enum CheckpointState {
    Empty,
    WithCheckpoint(Vec<(NetworkId, Height)>),
}

impl From<CheckpointState> for BTreeMap<NetworkId, Height> {
    fn from(value: CheckpointState) -> Self {
        match value {
            CheckpointState::Empty => BTreeMap::new(),
            CheckpointState::WithCheckpoint(checkpoint) => checkpoint.into_iter().collect(),
        }
    }
}

type StartCheckpointState = CheckpointState;
type EndCheckpointState = CheckpointState;

#[rstest]
#[case::when_state_are_empty(
    StartCheckpointState::Empty,
    EndCheckpointState::Empty,
    |result: Result<_, Error>| result.is_ok(),
    Height::ZERO, Some(EpochNumber::ZERO), Some(CertificateIndex::ZERO))]
#[case::when_state_is_incorrect(
    StartCheckpointState::WithCheckpoint(vec![(NetworkId::new(0), Height::ZERO)]),
    EndCheckpointState::Empty,
    |result: Result<_, Error>| matches!(result, Err(Error::Unexpected(_))),
    Height::ZERO, None, None)]
#[case::when_certificate_is_unexpected(
    StartCheckpointState::WithCheckpoint(vec![(NetworkId::new(0), Height::new(1))]),
    EndCheckpointState::WithCheckpoint(vec![(NetworkId::new(0), Height::new(1))]),
    |result: Result<_, Error>| {
        matches!(
            result,
            Err(Error::CertificateCandidateError(crate::error::CertificateCandidateError::UnexpectedHeight(_, Height::ZERO, h1)))
                if h1 == Height::new(1)
        )
    },
    Height::ZERO, None, None)]
#[case::when_certificate_is_already_present(
    StartCheckpointState::WithCheckpoint(vec![(NetworkId::new(0), Height::new(1))]),
    EndCheckpointState::WithCheckpoint(vec![(NetworkId::new(0), Height::new(1))]),
    |result: Result<_, Error>| {
        matches!(
            result,
            Err(Error::CertificateCandidateError(crate::error::CertificateCandidateError::UnexpectedHeight(_, h1, h1_)))
                if h1 == Height::new(1) && h1_ == Height::new(1)
        )
    },
    Height::new(1), None, None)]
#[case::when_there_is_a_gap(
    StartCheckpointState::WithCheckpoint(vec![(NetworkId::new(0), Height::new(1))]),
    EndCheckpointState::WithCheckpoint(vec![(NetworkId::new(0), Height::new(1))]),
    |result: Result<_, Error>| {
        matches!(
            result,
            Err(Error::CertificateCandidateError(crate::error::CertificateCandidateError::UnexpectedHeight(_, h3, h1)))
                if h1 == Height::new(1) && h3 == Height::new(3)
        )
    },
    Height::new(3), None, None)]
fn adding_a_certificate(
    mut store: PerEpochStore<PendingStore, StateStore>,
    #[case] start_checkpoint: StartCheckpointState,
    #[case] end_checkpoint: EndCheckpointState,
    #[case] expected_result: impl FnOnce(Result<(EpochNumber, CertificateIndex), Error>) -> bool,
    #[case] height: Height,
    #[case] expected_epoch_number: Option<EpochNumber>,
    #[case] expected_certificate_index: Option<CertificateIndex>,
) {
    use agglayer_types::CertificateStatus;

    let network_id = 0.into();
    let certificate = Certificate::new_for_test(network_id, height);
    let certificate_id = certificate.hash();
    let pending_store = store.pending_store.clone();
    let state_store = store.state_store.clone();

    state_store
        .insert_certificate_header(&certificate, CertificateStatus::Proven)
        .unwrap();

    pending_store
        .insert_pending_certificate(network_id, height, &certificate)
        .unwrap();

    pending_store
        .insert_generated_proof(&certificate_id, &Proof::dummy())
        .unwrap();

    store.start_checkpoint = start_checkpoint.into();
    store.end_checkpoint = RwLock::new(end_checkpoint.into());

    assert!(expected_result(store.add_certificate(
        certificate_id,
        agglayer_types::ExecutionMode::Default
    )));

    let header = state_store
        .get_certificate_header(&certificate_id)
        .unwrap()
        .unwrap();
    assert_eq!(header.epoch_number, expected_epoch_number);
    assert_eq!(header.certificate_index, expected_certificate_index);
    assert_eq!(
        header.status,
        if expected_certificate_index.is_some() {
            CertificateStatus::Settled
        } else {
            CertificateStatus::Proven
        }
    );
}

#[rstest]
#[case::when_state_are_empty(
    StartCheckpointState::Empty,
    EndCheckpointState::Empty,
    VecDeque::from([|result: Result<_, Error>| result.is_ok(), |result: Result<_, Error>| result.is_err()]),
    Height::ZERO)]
#[case::when_state_are_empty_and_starting_at_wrong_height(
    StartCheckpointState::Empty,
    EndCheckpointState::Empty,
    VecDeque::from([|result: Result<_, Error>| result.is_err()]),
    Height::new(1))]
#[case::when_state_is_already_full(
    StartCheckpointState::Empty,
    EndCheckpointState::WithCheckpoint(vec![(NetworkId::new(0), Height::ZERO)]),
    VecDeque::from([|result: Result<_, Error>| result.is_err()]),
    Height::new(1))]
#[case::when_state_contains_other_network(
    StartCheckpointState::Empty,
    EndCheckpointState::WithCheckpoint(vec![(NetworkId::new(1), Height::ZERO)]),
    VecDeque::from([|result: Result<_, Error>| result.is_ok()]),
    Height::ZERO)]
#[case::when_state_contains_other_network_but_wrong_height(
    StartCheckpointState::Empty,
    EndCheckpointState::WithCheckpoint(vec![(NetworkId::new(1), Height::ZERO)]),
    VecDeque::from([|result: Result<_, Error>| result.is_err()]),
    Height::new(1))]
fn adding_multiple_certificates(
    mut store: PerEpochStore<PendingStore, StateStore>,
    #[case] start_checkpoint: StartCheckpointState,
    #[case] end_checkpoint: EndCheckpointState,
    #[case] mut expected_results: VecDeque<
        impl FnOnce(Result<(EpochNumber, CertificateIndex), Error>) -> bool,
    >,
    #[case] mut height: Height,
) {
    use agglayer_types::Certificate;
    use parking_lot::RwLock;

    use crate::stores::PendingCertificateWriter as _;

    let pending_store = store.pending_store.clone();
    let state_store = store.state_store.clone();
    let network = 0.into();

    store.start_checkpoint = start_checkpoint.into();
    store.end_checkpoint = RwLock::new(end_checkpoint.into());

    while let Some(expected_result) = expected_results.pop_front() {
        let certificate = Certificate::new_for_test(network, height);
        pending_store
            .insert_pending_certificate(network, height, &certificate)
            .unwrap();
        state_store
            .insert_certificate_header(&certificate, CertificateStatus::Pending)
            .unwrap();

        pending_store
            .insert_generated_proof(&certificate.hash(), &Proof::dummy())
            .unwrap();

        assert!(
            expected_result(
                store.add_certificate(certificate.hash(), agglayer_types::ExecutionMode::Default)
            ),
            "{network}:{height} failed to pass the test"
        );

        height.increment();
    }
}

#[test_log::test(rstest)]
fn adding_certificate_and_restart() {
    let tmp = TempDBDir::new();
    let config = Arc::new(Config::new(&tmp.path));
    let pending_store =
        Arc::new(PendingStore::new_with_path(&config.storage.pending_db_path).unwrap());
    let state_store = Arc::new(
        StateStore::new_with_path(&config.storage.state_db_path, BackupClient::noop()).unwrap(),
    );

    info!("Opening the epoch store for the first time");
    let backup_client = BackupClient::noop();
    let store = PerEpochStore::try_open(
        config.clone(),
        EpochNumber::ZERO,
        pending_store,
        state_store,
        None,
        backup_client.clone(),
    )
    .unwrap();

    let network = 1.into();
    let height = Height::ZERO;

    let certificate = Certificate::new_for_test(network, height);
    let certificate_id = certificate.hash();
    let pending_store = store.pending_store.clone();
    let state_store = store.state_store.clone();

    state_store
        .insert_certificate_header(&certificate, CertificateStatus::Proven)
        .unwrap();

    pending_store
        .insert_pending_certificate(network, height, &certificate)
        .unwrap();

    pending_store
        .insert_generated_proof(&certificate_id, &Proof::dummy())
        .unwrap();

    assert!(
        store
            .add_certificate(certificate.hash(), agglayer_types::ExecutionMode::Default)
            .is_ok(),
        "{network}:{height} failed to pass the test"
    );

    drop(store);

    info!("Opening the epoch store for the second time");
    let store = PerEpochStore::try_open(
        config,
        EpochNumber::ZERO,
        pending_store,
        state_store,
        None,
        backup_client,
    )
    .unwrap();

    let network = 2.into();
    let height = Height::ZERO;

    let certificate = Certificate::new_for_test(network, height);
    let certificate_id = certificate.hash();
    let pending_store = store.pending_store.clone();
    let state_store = store.state_store.clone();

    state_store
        .insert_certificate_header(&certificate, CertificateStatus::Proven)
        .unwrap();

    pending_store
        .insert_pending_certificate(network, height, &certificate)
        .unwrap();

    pending_store
        .insert_generated_proof(&certificate_id, &Proof::dummy())
        .unwrap();

    assert!(
        store
            .add_certificate(certificate.hash(), agglayer_types::ExecutionMode::Default)
            .is_ok(),
        "{network}:{height} failed to pass the test"
    );

    let first = store
        .get_certificate_at_index(CertificateIndex::ZERO)
        .unwrap()
        .unwrap();
    assert!(
        first.network_id == 1.into(),
        "Network ID mismatch {} != {}",
        first.network_id,
        1
    );

    let second = store
        .get_certificate_at_index(CertificateIndex::new(1))
        .unwrap()
        .unwrap();
    assert!(
        second.network_id == 2.into(),
        "Network ID mismatch {} != {}",
        second.network_id,
        2
    );
}

#[rstest]
fn can_retrieve_proof_at_index() {
    let tmp = TempDBDir::new();
    let config = Arc::new(Config::new(&tmp.path));
    let pending_store =
        Arc::new(PendingStore::new_with_path(&config.storage.pending_db_path).unwrap());
    let state_store = Arc::new(
        StateStore::new_with_path(&config.storage.state_db_path, BackupClient::noop()).unwrap(),
    );

    let backup_client = BackupClient::noop();
    let store = PerEpochStore::try_open(
        config,
        EpochNumber::ZERO,
        pending_store,
        state_store,
        None,
        backup_client,
    )
    .unwrap();

    let network = 1.into();
    let height = Height::ZERO;

    let certificate = Certificate::new_for_test(network, height);
    let certificate_id = certificate.hash();
    let pending_store = store.pending_store.clone();
    let state_store = store.state_store.clone();

    state_store
        .insert_certificate_header(&certificate, CertificateStatus::Proven)
        .unwrap();

    pending_store
        .insert_pending_certificate(network, height, &certificate)
        .unwrap();

    pending_store
        .insert_generated_proof(&certificate_id, &Proof::dummy())
        .unwrap();

    // Add the certificate to the epoch store
    assert!(
        store
            .add_certificate(certificate.hash(), agglayer_types::ExecutionMode::Default)
            .is_ok(),
        "Failed to add certificate to epoch store"
    );

    // Verify that we can retrieve the proof at index 0
    let proof = store.get_proof_at_index(CertificateIndex::ZERO).unwrap();
    assert!(proof.is_some(), "Should retrieve a proof");

    // Verify that retrieving proof at non-existent index returns None
    let non_existent_proof = store.get_proof_at_index(CertificateIndex::new(1)).unwrap();
    assert!(
        non_existent_proof.is_none(),
        "Should return None for non-existent index"
    );
}
