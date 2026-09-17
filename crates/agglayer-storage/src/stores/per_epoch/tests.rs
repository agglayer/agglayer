use std::{
    collections::{BTreeMap, VecDeque},
    path::PathBuf,
    sync::Arc,
};

use agglayer_config::Config;
use agglayer_types::{
    Certificate, CertificateIndex, CertificateStatus, EpochNumber, Height, Metadata, NetworkId,
    Proof,
};
use parking_lot::RwLock;
use pessimistic_proof_test_suite::sample_data;
use prost::Message as _;
use rocksdb::WriteBatch;
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
        interfaces::writer::{PerEpochWriter, StateWriter},
        pending::PendingStore,
        per_epoch::PerEpochStore,
        state::StateStore,
        EpochStoreReader as _, PendingCertificateReader as _, PendingCertificateWriter as _,
        PerEpochReader as _, StateReader,
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

fn persist_epoch_phase(
    store: &PerEpochStore<PendingStore, StateStore>,
    certificate: &Certificate,
    proof: &Proof,
    index: CertificateIndex,
) {
    let mut batch = WriteBatch::default();
    let network_id = certificate.network_id;
    let height = certificate.height;

    store
        .db
        .multi_insert_batch::<CertificatePerIndexProtoColumn>(
            [(&index, certificate)],
            &mut batch,
        )
        .unwrap();
    store
        .db
        .multi_insert_batch::<crate::columns::epochs::proofs::ProofPerIndexColumn>(
            [(&index, proof)],
            &mut batch,
        )
        .unwrap();
    store
        .db
        .multi_insert_batch::<EndCheckpointColumn>(
            [(&network_id, &height)],
            &mut batch,
        )
        .unwrap();
    store.db.write_batch(batch).unwrap();
}

#[test]
fn add_certificate_recovers_committed_epoch_phase_after_restart() {
    let tmp = TempDBDir::new();
    let config = Arc::new(Config::new(&tmp.path));
    let pending_store =
        Arc::new(PendingStore::new_with_path(&config.storage.pending_db_path).unwrap());
    let state_store = Arc::new(
        StateStore::new_with_path(&config.storage.state_db_path, BackupClient::noop()).unwrap(),
    );
    let store = PerEpochStore::try_open(
        config.clone(),
        EpochNumber::ZERO,
        pending_store.clone(),
        state_store.clone(),
        None,
        BackupClient::noop(),
    )
    .unwrap();

    let certificate = Certificate::new_for_test(NetworkId::new(1), Height::ZERO);
    let certificate_id = certificate.hash();
    let proof = Proof::dummy();

    state_store
        .insert_certificate_header(&certificate, CertificateStatus::Proven)
        .unwrap();
    pending_store
        .insert_pending_certificate(certificate.network_id, certificate.height, &certificate)
        .unwrap();
    pending_store
        .insert_generated_proof(&certificate_id, &proof)
        .unwrap();

    persist_epoch_phase(&store, &certificate, &proof, CertificateIndex::ZERO);

    // A later write for another network makes the incomplete certificate no
    // longer the last row in the epoch. Recovery must search by certificate
    // id instead of assuming the previous index is the one to resume.
    let later = Certificate::new_for_test(NetworkId::new(2), Height::ZERO);
    persist_epoch_phase(
        &store,
        &later,
        &Proof::dummy(),
        CertificateIndex::new(1),
    );

    drop(store);

    let store = PerEpochStore::try_open(
        config,
        EpochNumber::ZERO,
        pending_store.clone(),
        state_store.clone(),
        None,
        BackupClient::noop(),
    )
    .unwrap();

    assert_eq!(
        store
            .add_certificate(certificate_id, agglayer_types::ExecutionMode::Default)
            .unwrap(),
        (EpochNumber::ZERO, CertificateIndex::ZERO)
    );

    let header = state_store
        .get_certificate_header(&certificate_id)
        .unwrap()
        .unwrap();
    assert_eq!(header.epoch_number, Some(EpochNumber::ZERO));
    assert_eq!(header.certificate_index, Some(CertificateIndex::ZERO));
    assert_eq!(header.status, CertificateStatus::Settled);
    assert!(pending_store
        .get_certificate(certificate.network_id, certificate.height)
        .unwrap()
        .is_none());
    assert!(pending_store.get_proof(certificate_id).unwrap().is_none());
    assert!(store
        .get_certificate_at_index(CertificateIndex::new(2))
        .unwrap()
        .is_none());
}

#[test]
fn add_certificate_finishes_pending_cleanup_after_restart() {
    let tmp = TempDBDir::new();
    let config = Arc::new(Config::new(&tmp.path));
    let pending_store =
        Arc::new(PendingStore::new_with_path(&config.storage.pending_db_path).unwrap());
    let state_store = Arc::new(
        StateStore::new_with_path(&config.storage.state_db_path, BackupClient::noop()).unwrap(),
    );
    let store = PerEpochStore::try_open(
        config.clone(),
        EpochNumber::ZERO,
        pending_store.clone(),
        state_store.clone(),
        None,
        BackupClient::noop(),
    )
    .unwrap();

    let certificate = Certificate::new_for_test(NetworkId::new(1), Height::ZERO);
    let certificate_id = certificate.hash();
    let proof = Proof::dummy();
    let index = CertificateIndex::ZERO;

    state_store
        .insert_certificate_header(&certificate, CertificateStatus::Proven)
        .unwrap();
    pending_store
        .insert_pending_certificate(certificate.network_id, certificate.height, &certificate)
        .unwrap();
    pending_store
        .insert_generated_proof(&certificate_id, &proof)
        .unwrap();

    persist_epoch_phase(&store, &certificate, &proof, index);
    state_store
        .assign_certificate_to_epoch(&certificate_id, &EpochNumber::ZERO, &index)
        .unwrap();

    // Simulate a crash after the first cleanup delete.
    pending_store.remove_generated_proof(&certificate_id).unwrap();
    drop(store);

    let store = PerEpochStore::try_open(
        config,
        EpochNumber::ZERO,
        pending_store.clone(),
        state_store.clone(),
        None,
        BackupClient::noop(),
    )
    .unwrap();

    assert_eq!(
        store
            .add_certificate(certificate_id, agglayer_types::ExecutionMode::Default)
            .unwrap(),
        (EpochNumber::ZERO, index)
    );
    assert!(pending_store
        .get_certificate(certificate.network_id, certificate.height)
        .unwrap()
        .is_none());
    assert!(pending_store.get_proof(certificate_id).unwrap().is_none());
    assert!(store
        .get_certificate_at_index(CertificateIndex::new(1))
        .unwrap()
        .is_none());
}

#[test]
fn recovered_cleanup_does_not_delete_replaced_pending_certificate() {
    let tmp = TempDBDir::new();
    let config = Arc::new(Config::new(&tmp.path));
    let pending_store =
        Arc::new(PendingStore::new_with_path(&config.storage.pending_db_path).unwrap());
    let state_store = Arc::new(
        StateStore::new_with_path(&config.storage.state_db_path, BackupClient::noop()).unwrap(),
    );
    let store = PerEpochStore::try_open(
        config,
        EpochNumber::ZERO,
        pending_store.clone(),
        state_store.clone(),
        None,
        BackupClient::noop(),
    )
    .unwrap();

    let certificate = Certificate::new_for_test(NetworkId::new(1), Height::ZERO);
    let certificate_id = certificate.hash();
    let proof = Proof::dummy();
    let index = CertificateIndex::ZERO;

    state_store
        .insert_certificate_header(&certificate, CertificateStatus::Proven)
        .unwrap();
    pending_store
        .insert_pending_certificate(certificate.network_id, certificate.height, &certificate)
        .unwrap();
    pending_store
        .insert_generated_proof(&certificate_id, &proof)
        .unwrap();

    persist_epoch_phase(&store, &certificate, &proof, index);
    state_store
        .assign_certificate_to_epoch(&certificate_id, &EpochNumber::ZERO, &index)
        .unwrap();

    let mut replacement = Certificate::new_for_test(certificate.network_id, certificate.height);
    replacement.metadata = Metadata::new([1; 32].into());
    assert_ne!(replacement.hash(), certificate_id);
    pending_store
        .insert_pending_certificate(replacement.network_id, replacement.height, &replacement)
        .unwrap();

    assert_eq!(
        store
            .add_certificate(certificate_id, agglayer_types::ExecutionMode::Default)
            .unwrap(),
        (EpochNumber::ZERO, index)
    );

    assert_eq!(
        pending_store
            .get_certificate(replacement.network_id, replacement.height)
            .unwrap(),
        Some(replacement)
    );
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

    let store = PerEpochStore::try_open(
        config,
        epoch_number,
        pending_store,
        state_store,
        Some(std::iter::once((network_id, height)).collect()),
        BackupClient::noop(),
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
fn readonly_epoch_access_falls_back_to_legacy_cf_when_proto_cf_absent() {
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
    let expected = sample_data::load_certificate("n15-cert_h0.json");
    let legacy = load_v0_certificate_bytes("v0-n15-cert_h0.hex");
    let index = CertificateIndex::ZERO;

    write_raw_epoch_certificate_bytes(&epoch_path, index, legacy);

    // The read must fall back to the still-present legacy CF instead of failing
    // with `ColumnFamilyNotFound`.
    assert_eq!(
        epochs_store.get_certificate(epoch_number, index).unwrap(),
        Some(expected)
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
    VecDeque::from([|result: Result<_, Error>| result.is_ok(), |result: Result<_, Error>| result.is_ok()]),
    Height::ZERO)]
#[case::when_state_are_empty_and_starting_at_wrong_height(
    StartCheckpointState::Empty,
    EndCheckpointState::Empty,
    VecDeque::from([|result: Result<_, Error>| result.is_err()]),
    Height::new(1))]
#[case::when_state_is_already_full(
    StartCheckpointState::Empty,
    EndCheckpointState::WithCheckpoint(vec![(NetworkId::new(0), Height::ZERO)]),
    VecDeque::from([|result: Result<_, Error>| result.is_ok()]),
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

#[rstest]
#[case::five_certificates(5)]
#[case::ten_certificates(10)]
#[case::fifty_certificates(50)]
fn can_add_multiple_certificates_per_epoch(
    store: PerEpochStore<PendingStore, StateStore>,
    #[case] num_certificates: u64,
) {
    let pending_store = store.pending_store.clone();
    let state_store = store.state_store.clone();
    let network = NetworkId::new(0);

    // Add multiple certificates sequentially
    for i in 0..num_certificates {
        let height = Height::new(i);
        let certificate = Certificate::new_for_test(network, height);
        let certificate_id = certificate.hash();

        state_store
            .insert_certificate_header(&certificate, CertificateStatus::Proven)
            .unwrap();

        pending_store
            .insert_pending_certificate(network, height, &certificate)
            .unwrap();

        pending_store
            .insert_generated_proof(&certificate_id, &Proof::dummy())
            .unwrap();

        let result = store.add_certificate(certificate_id, agglayer_types::ExecutionMode::Default);
        assert!(
            result.is_ok(),
            "Failed to add certificate at height {}: {:?}",
            i,
            result
        );

        let (epoch_number, certificate_index) = result.unwrap();
        assert_eq!(epoch_number, EpochNumber::ZERO);
        assert_eq!(certificate_index, CertificateIndex::new(i));
    }

    // Verify all certificates can be retrieved
    for i in 0..num_certificates {
        let certificate = store
            .get_certificate_at_index(CertificateIndex::new(i))
            .unwrap();
        assert!(
            certificate.is_some(),
            "Certificate at index {} should exist",
            i
        );

        let cert = certificate.unwrap();
        assert_eq!(cert.network_id, network);
        assert_eq!(cert.height, Height::new(i));
    }

    // Verify end checkpoint is updated correctly
    let end_checkpoint = store.get_end_checkpoint();
    assert_eq!(
        end_checkpoint.get(&network),
        Some(&Height::new(num_certificates - 1))
    );
}

#[rstest]
fn multiple_networks_multiple_certificates_per_epoch(
    store: PerEpochStore<PendingStore, StateStore>,
) {
    let pending_store = store.pending_store.clone();
    let state_store = store.state_store.clone();

    // Add 10 certificates for 3 different networks
    let networks = [NetworkId::new(0), NetworkId::new(1), NetworkId::new(2)];
    let certificates_per_network = 10;

    for network in networks.iter() {
        for i in 0..certificates_per_network {
            let height = Height::new(i);
            let certificate = Certificate::new_for_test(*network, height);
            let certificate_id = certificate.hash();

            state_store
                .insert_certificate_header(&certificate, CertificateStatus::Proven)
                .unwrap();

            pending_store
                .insert_pending_certificate(*network, height, &certificate)
                .unwrap();

            pending_store
                .insert_generated_proof(&certificate_id, &Proof::dummy())
                .unwrap();

            let result =
                store.add_certificate(certificate_id, agglayer_types::ExecutionMode::Default);
            assert!(
                result.is_ok(),
                "Failed to add certificate for network {} at height {}: {:?}",
                network,
                i,
                result
            );
        }
    }

    // Verify end checkpoints for all networks
    let end_checkpoint = store.get_end_checkpoint();
    for network in networks.iter() {
        assert_eq!(
            end_checkpoint.get(network),
            Some(&Height::new(certificates_per_network - 1)),
            "Network {} should have end checkpoint at height {}",
            network,
            certificates_per_network - 1
        );
    }

    // Verify total number of certificates
    let mut count = 0;
    for i in 0..(networks.len() as u64 * certificates_per_network) {
        if store
            .get_certificate_at_index(CertificateIndex::new(i))
            .unwrap()
            .is_some()
        {
            count += 1;
        }
    }
    assert_eq!(
        count,
        networks.len() as u64 * certificates_per_network,
        "Should have {} total certificates",
        networks.len() * certificates_per_network as usize
    );
}

#[rstest]
fn sequential_validation_still_enforced(store: PerEpochStore<PendingStore, StateStore>) {
    let pending_store = store.pending_store.clone();
    let state_store = store.state_store.clone();
    let network = NetworkId::new(0);

    // Add first certificate at height 0
    let height0 = Height::ZERO;
    let certificate0 = Certificate::new_for_test(network, height0);
    let certificate_id0 = certificate0.hash();

    pending_store
        .insert_pending_certificate(network, height0, &certificate0)
        .unwrap();

    let height1 = Height::new(1);
    let certificate1 = Certificate::new_for_test(network, height1);
    let certificate_id1 = certificate1.hash();

    pending_store
        .insert_pending_certificate(network, height1, &certificate1)
        .unwrap();

    let height2 = Height::new(2);
    let certificate2 = Certificate::new_for_test(network, height2);
    let certificate_id2 = certificate2.hash();

    pending_store
        .insert_pending_certificate(network, height2, &certificate2)
        .unwrap();

    state_store
        .insert_certificate_header(&certificate0, CertificateStatus::Proven)
        .unwrap();

    pending_store
        .insert_generated_proof(&certificate_id0, &Proof::dummy())
        .unwrap();

    assert!(store
        .add_certificate(certificate_id0, agglayer_types::ExecutionMode::Default)
        .is_ok());

    state_store
        .insert_certificate_header(&certificate2, CertificateStatus::Proven)
        .unwrap();

    pending_store
        .insert_generated_proof(&certificate_id2, &Proof::dummy())
        .unwrap();

    let result = store.add_certificate(certificate_id2, agglayer_types::ExecutionMode::Default);
    assert!(
        result.is_err(),
        "Should reject certificate with gap in heights"
    );
    assert!(matches!(
        result,
        Err(Error::CertificateCandidateError(
            crate::error::CertificateCandidateError::UnexpectedHeight(_, _, _)
        ))
    ));

    state_store
        .insert_certificate_header(&certificate1, CertificateStatus::Proven)
        .unwrap();

    pending_store
        .insert_generated_proof(&certificate_id1, &Proof::dummy())
        .unwrap();

    assert!(store
        .add_certificate(certificate_id1, agglayer_types::ExecutionMode::Default)
        .is_ok());
}
