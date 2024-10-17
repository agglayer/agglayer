use std::{
    collections::BTreeMap,
    sync::{atomic::Ordering, Arc},
};

use agglayer_config::Config;
use agglayer_types::{Height, NetworkId, Proof};
use rstest::rstest;

use crate::{
    error::Error,
    storage::{pending_db_cf_definitions, DB},
    stores::{pending::PendingStore, per_epoch::PerEpochStore, PerEpochWriter as _},
    tests::TempDBDir,
};

#[test]
fn can_start_packing_an_unpacked_epoch() {
    let tmp = TempDBDir::new();
    let pending_db =
        Arc::new(DB::open_cf(tmp.path.as_path(), pending_db_cf_definitions()).unwrap());
    let config = Arc::new(Config::new_for_test());

    let store = PerEpochStore::try_open(config, 0, pending_db.clone()).unwrap();

    assert!(store.start_packing().is_ok());
}

#[test]
fn cant_start_packing_a_packed_epoch() {
    let tmp = TempDBDir::new();
    let pending_db =
        Arc::new(DB::open_cf(tmp.path.as_path(), pending_db_cf_definitions()).unwrap());
    let config = Arc::new(Config::new(&tmp.path));

    let store = PerEpochStore::try_open(config, 0, pending_db.clone()).unwrap();

    store.in_packing.store(true, Ordering::Relaxed);

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
    |result: Result<(), Error>| result.is_ok(),
    0)]
#[case::when_state_is_incorrect(
    StartCheckpointState::WithCheckpoint(vec![(NetworkId::new(0), 0)]),
    EndCheckpointState::Empty,
    |result: Result<(), Error>| matches!(result, Err(Error::Unexpected(_))),
    0)]
#[case::when_certificate_is_unexpected(
    StartCheckpointState::WithCheckpoint(vec![(NetworkId::new(0), 1)]),
    EndCheckpointState::WithCheckpoint(vec![(NetworkId::new(0), 1)]),
    |result: Result<(), Error>| {
        matches!(result, Err(Error::CertificateCandidateError(crate::error::CertificateCandidateError::UnexpectedHeight(_, 0, 1))))
    }, 0)]
#[case::when_certificate_is_already_present(
    StartCheckpointState::WithCheckpoint(vec![(NetworkId::new(0), 1)]),
    EndCheckpointState::WithCheckpoint(vec![(NetworkId::new(0), 1)]),
    |result: Result<(), Error>| {
        matches!(result, Err(Error::CertificateCandidateError(crate::error::CertificateCandidateError::UnexpectedHeight(_, 1, 1))))
    }, 1)]
#[case::when_there_is_a_gap(
    StartCheckpointState::WithCheckpoint(vec![(NetworkId::new(0), 1)]),
    EndCheckpointState::WithCheckpoint(vec![(NetworkId::new(0), 1)]),
    |result: Result<(), Error>| {
        matches!(result, Err(Error::CertificateCandidateError(crate::error::CertificateCandidateError::UnexpectedHeight(_, 3, 1))))
    }, 3)]
fn adding_a_certificate(
    #[case] start_checkpoint: StartCheckpointState,
    #[case] end_checkpoint: EndCheckpointState,
    #[case] expected_result: impl FnOnce(Result<(), Error>) -> bool,
    #[case] height: Height,
) {
    use agglayer_types::Certificate;
    use parking_lot::RwLock;

    use crate::stores::PendingCertificateWriter as _;

    let tmp = TempDBDir::new();
    let pending_db =
        Arc::new(DB::open_cf(tmp.path.as_path(), pending_db_cf_definitions()).unwrap());
    let config = Arc::new(Config::new(&tmp.path));

    let certificate = Certificate::new_for_test(0.into(), height);
    let pending_store = PendingStore::new(pending_db.clone());
    pending_store
        .insert_pending_certificate(0.into(), height, &certificate)
        .unwrap();

    pending_store
        .insert_generated_proof(&certificate.hash(), &Proof::new_for_test())
        .unwrap();

    let mut store = PerEpochStore::try_open(config, 0, pending_db.clone()).unwrap();

    store.start_checkpoint = start_checkpoint.into();
    store.end_checkpoint = RwLock::new(end_checkpoint.into());

    assert!(expected_result(store.add_certificate(0.into(), height)));
}
