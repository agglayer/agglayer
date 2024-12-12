use std::{
    collections::{BTreeMap, VecDeque},
    sync::Arc,
};

use agglayer_config::Config;
use agglayer_types::Certificate;
use agglayer_types::{Height, NetworkId, Proof};
use parking_lot::RwLock;
use rstest::{fixture, rstest};

use crate::stores::interfaces::writer::StateWriter;
use crate::stores::{PendingCertificateWriter as _, StateReader};
use crate::{
    error::Error,
    stores::{
        interfaces::writer::PerEpochWriter, pending::PendingStore, per_epoch::PerEpochStore,
        state::StateStore,
    },
    tests::TempDBDir,
};

#[fixture]
fn store() -> PerEpochStore<PendingStore, StateStore> {
    let tmp = TempDBDir::new();
    let config = Arc::new(Config::new(&tmp.path));
    let pending_store =
        Arc::new(PendingStore::new_with_path(&config.storage.pending_db_path).unwrap());
    let state_store = Arc::new(StateStore::new_with_path(&config.storage.state_db_path).unwrap());

    PerEpochStore::try_open(config, 0, pending_store, state_store, None).unwrap()
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
    0, Some(0), Some(0))]
#[case::when_state_is_incorrect(
    StartCheckpointState::WithCheckpoint(vec![(NetworkId::new(0), 0)]),
    EndCheckpointState::Empty,
    |result: Result<_, Error>| matches!(result, Err(Error::Unexpected(_))),
    0, None, None)]
#[case::when_certificate_is_unexpected(
    StartCheckpointState::WithCheckpoint(vec![(NetworkId::new(0), 1)]),
    EndCheckpointState::WithCheckpoint(vec![(NetworkId::new(0), 1)]),
    |result: Result<_, Error>| {
        matches!(result, Err(Error::CertificateCandidateError(crate::error::CertificateCandidateError::UnexpectedHeight(_, 0, 1))))
    }, 0, None, None)]
#[case::when_certificate_is_already_present(
    StartCheckpointState::WithCheckpoint(vec![(NetworkId::new(0), 1)]),
    EndCheckpointState::WithCheckpoint(vec![(NetworkId::new(0), 1)]),
    |result: Result<_, Error>| {
        matches!(result, Err(Error::CertificateCandidateError(crate::error::CertificateCandidateError::UnexpectedHeight(_, 1, 1))))
    }, 1, None, None)]
#[case::when_there_is_a_gap(
    StartCheckpointState::WithCheckpoint(vec![(NetworkId::new(0), 1)]),
    EndCheckpointState::WithCheckpoint(vec![(NetworkId::new(0), 1)]),
    |result: Result<_, Error>| {
        matches!(result, Err(Error::CertificateCandidateError(crate::error::CertificateCandidateError::UnexpectedHeight(_, 3, 1))))
    }, 3, None, None)]
fn adding_a_certificate(
    mut store: PerEpochStore<PendingStore, StateStore>,
    #[case] start_checkpoint: StartCheckpointState,
    #[case] end_checkpoint: EndCheckpointState,
    #[case] expected_result: impl FnOnce(Result<(u64, u64), Error>) -> bool,
    #[case] height: Height,
    #[case] expected_epoch_number: Option<u64>,
    #[case] expected_certificate_index: Option<u64>,
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
        network_id,
        height,
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
    0)]
#[case::when_state_are_empty_and_starting_at_wrong_height(
    StartCheckpointState::Empty,
    EndCheckpointState::Empty,
    VecDeque::from([|result: Result<_, Error>| result.is_err()]),
    1)]
#[case::when_state_is_already_full(
    StartCheckpointState::Empty,
    EndCheckpointState::WithCheckpoint(vec![(NetworkId::new(0), 0)]),
    VecDeque::from([|result: Result<_, Error>| result.is_err()]),
    1)]
#[case::when_state_contains_other_network(
    StartCheckpointState::Empty,
    EndCheckpointState::WithCheckpoint(vec![(NetworkId::new(1), 0)]),
    VecDeque::from([|result: Result<_, Error>| result.is_ok()]),
    0)]
#[case::when_state_contains_other_network_but_wrong_height(
    StartCheckpointState::Empty,
    EndCheckpointState::WithCheckpoint(vec![(NetworkId::new(1), 0)]),
    VecDeque::from([|result: Result<_, Error>| result.is_err()]),
    1)]
fn adding_multiple_certificates(
    mut store: PerEpochStore<PendingStore, StateStore>,
    #[case] start_checkpoint: StartCheckpointState,
    #[case] end_checkpoint: EndCheckpointState,
    #[case] mut expected_results: VecDeque<impl FnOnce(Result<(u64, u64), Error>) -> bool>,
    #[case] mut height: Height,
) {
    use agglayer_types::Certificate;
    use parking_lot::RwLock;

    use crate::stores::PendingCertificateWriter as _;

    let pending_store = store.pending_store.clone();
    let network = 0.into();

    store.start_checkpoint = start_checkpoint.into();
    store.end_checkpoint = RwLock::new(end_checkpoint.into());

    while let Some(expected_result) = expected_results.pop_front() {
        let certificate = Certificate::new_for_test(network, height);
        pending_store
            .insert_pending_certificate(network, height, &certificate)
            .unwrap();
        pending_store
            .insert_generated_proof(&certificate.hash(), &Proof::dummy())
            .unwrap();

        assert!(
            expected_result(store.add_certificate(
                network,
                height,
                agglayer_types::ExecutionMode::Default
            )),
            "{}:{} failed to pass the test",
            network,
            height
        );

        height += 1;
    }
}
