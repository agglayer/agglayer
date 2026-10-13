use std::{collections::BTreeMap, sync::Arc};

use agglayer_types::{
    aggchain_proof::{AggchainData, MultisigPayload},
    primitives::Hashable as _,
    Address, CertificateIndex, CertificateStatus, EpochNumber, Metadata, U256,
};
use pessimistic_proof::{
    nullifier_tree::NullifierKey,
    unified_bridge::{
        BridgeExit, Claim, ClaimFromMainnet, GlobalIndex, ImportedBridgeExit, L1InfoTreeLeaf,
        L1InfoTreeLeafInner, LeafType, MerkleProof, TokenInfo,
    },
};
use rstest::rstest;

use super::*;
use crate::{
    backup::BackupClient,
    columns::{
        network_info::NetworkInfoColumn, nullifier_tree_per_network::NullifierTreePerNetworkColumn,
    },
    schema::bincode_codec,
    storage::{DBMigrationError, DBMigrationErrorDetails, DBOpenError},
    stores::{
        pending::PendingStore, NetworkInfoReader, PendingCertificateWriter, StateReader,
        StateWriter,
    },
    tests::{init_state_db_before_network_info_backfill, mocks::MockEpochsStore, TempDBDir},
    types::{
        network_info::v0::{network_info_value::ValueDiscriminants, NetworkInfoValue},
        SmtKey, SmtKeyType, SmtValue,
    },
};

fn digest(seed: u8) -> Digest {
    Digest([seed; 32])
}

fn imported_exit(seed: u8) -> ImportedBridgeExit {
    ImportedBridgeExit {
        bridge_exit: BridgeExit {
            leaf_type: LeafType::Transfer,
            token_info: TokenInfo {
                origin_network: NetworkId::new(0),
                origin_token_address: Address::from([seed; 20]),
            },
            dest_network: NetworkId::new(1),
            dest_address: Address::from([seed; 20]),
            amount: U256::from(seed),
            metadata: None,
        },
        global_index: GlobalIndex::new(NetworkId::new(0), seed as u32),
        claim_data: Claim::Mainnet(Box::new(ClaimFromMainnet {
            proof_leaf_mer: MerkleProof::new(digest(seed), [digest(1); 32]),
            proof_ger_l1root: MerkleProof::new(digest(seed), [digest(2); 32]),
            l1_leaf: L1InfoTreeLeaf {
                l1_info_tree_index: 0,
                rer: digest(3),
                mer: digest(4),
                inner: L1InfoTreeLeafInner {
                    global_exit_root: digest(5),
                    block_hash: digest(6),
                    timestamp: 0,
                },
            },
        })),
    }
}

/// A test certificate carrying one imported bridge exit per seed.
fn certificate(network_id: u32, height: u64, claim_seeds: &[u8]) -> Certificate {
    let mut certificate =
        Certificate::new_for_test(NetworkId::new(network_id), Height::new(height));
    certificate.imported_bridge_exits = claim_seeds.iter().copied().map(imported_exit).collect();
    certificate
}

fn claim_of(certificate: &Certificate) -> Option<SettledClaim> {
    certificate.imported_bridge_exits.last().map(Into::into)
}

/// Each settled certificate gets its own epoch.
fn epoch_of(certificate: &Certificate) -> (EpochNumber, CertificateIndex) {
    (
        EpochNumber::new(
            u64::from(certificate.network_id.to_u32()) * 1_000 + certificate.height.as_u64(),
        ),
        CertificateIndex::ZERO,
    )
}

fn proof_with_public_values(public_values: &[u8]) -> Proof {
    let Proof::SP1(mut proof) = Proof::dummy();
    proof.public_values = sp1_sdk::SP1PublicValues::from(public_values);
    Proof::SP1(proof)
}

fn open_pre_backfill(path: &std::path::Path) -> StateStore {
    StateStore::new(
        Arc::new(init_state_db_before_network_info_backfill(path).unwrap()),
        BackupClient::noop(),
    )
}

#[derive(Clone, Copy)]
enum ProofReads {
    Stored,
    Undecodable,
    Unreadable,
}

/// A state database written by a build without the backfill, a real pending
/// store, and a mocked epoch store that counts every certificate read.
struct Fixture {
    state: Option<StateStore>,
    pending_db: Arc<DB>,
    pending: PendingStore,
    epochs: BTreeMap<(EpochNumber, CertificateIndex), Certificate>,
    epoch_proofs: BTreeMap<(EpochNumber, CertificateIndex), Proof>,
    /// Makes every epoch certificate read fail.
    unreadable_epochs: bool,
    /// What every epoch proof read returns.
    proof_reads: ProofReads,
    // Last, so the databases close before their directories are removed.
    state_dir: TempDBDir,
    _pending_dir: TempDBDir,
}

impl Fixture {
    fn new() -> Self {
        let state_dir = TempDBDir::new();
        let pending_dir = TempDBDir::new();
        let pending_db = Arc::new(PendingStore::init_db(&pending_dir.path).unwrap());

        Self {
            state: Some(open_pre_backfill(&state_dir.path)),
            pending: PendingStore::new(pending_db.clone()),
            pending_db,
            epochs: BTreeMap::new(),
            epoch_proofs: BTreeMap::new(),
            unreadable_epochs: false,
            proof_reads: ProofReads::Stored,
            state_dir,
            _pending_dir: pending_dir,
        }
    }

    fn state(&self) -> &StateStore {
        self.state.as_ref().unwrap()
    }

    /// Settle `certificates` in order the way the network task does, and point
    /// the legacy settled index at each in turn. The local state records one
    /// nullifier per imported bridge exit.
    fn settle(&mut self, certificates: &[Certificate]) {
        for certificate in certificates {
            let network_id = certificate.network_id;
            let (epoch, index) = epoch_of(certificate);
            let state = self.state();
            state
                .insert_certificate_header(certificate, CertificateStatus::Settled)
                .unwrap();
            state
                .assign_certificate_to_epoch(&certificate.hash(), &epoch, &index)
                .unwrap();

            let mut local_state = state
                .read_local_network_state(network_id)
                .unwrap()
                .unwrap_or_default();
            for exit in &certificate.imported_bridge_exits {
                local_state
                    .nullifier_tree
                    .insert(NullifierKey::from(exit.global_index), digest(1))
                    .unwrap();
            }
            let leaves: Vec<_> = certificate
                .bridge_exits
                .iter()
                .map(|exit| exit.hash())
                .collect();
            for leaf in &leaves {
                local_state.exit_tree.add_leaf(*leaf).unwrap();
            }
            state
                .write_local_network_state(&network_id, &local_state, &leaves)
                .unwrap();
            state
                .db
                .put::<LatestSettledCertificatePerNetworkColumn>(
                    &network_id,
                    &SettledCertificate(certificate.hash(), certificate.height, epoch, index),
                )
                .unwrap();
            self.epochs.insert((epoch, index), certificate.clone());
        }
    }

    /// Accept `certificate` as pending: its header and the legacy pointer.
    fn submit(&self, certificate: &Certificate) {
        self.state()
            .insert_certificate_header(certificate, CertificateStatus::Pending)
            .unwrap();
        self.pending
            .insert_pending_certificate(certificate.network_id, certificate.height, certificate)
            .unwrap();
    }

    fn prove(&self, certificate: &Certificate) {
        self.pending
            .set_latest_proven_certificate_per_network(
                &certificate.network_id,
                &certificate.height,
                &certificate.hash(),
            )
            .unwrap();
    }

    /// Store `certificate` in the pending store without its legacy pointer.
    fn store_pending_body(&self, height: Height, certificate: &Certificate) {
        self.pending_db
            .put::<crate::columns::pending_queue::PendingQueueProtoColumn>(
                &crate::columns::pending_queue::PendingQueueKey(certificate.network_id, height),
                certificate,
            )
            .unwrap();
    }

    /// An epoch store serving the settled certificates, which must be read
    /// exactly `certificate_reads` times.
    fn epoch_store(&self, certificate_reads: usize) -> MockEpochsStore {
        let epochs = self.epochs.clone();
        let unreadable = self.unreadable_epochs;
        let mut store = MockEpochsStore::new();
        store
            .expect_get_certificate()
            .times(certificate_reads)
            .returning(move |epoch, index| {
                if unreadable {
                    return Err(Error::Unexpected("unreadable epoch".to_string()));
                }
                Ok(epochs.get(&(epoch, index)).cloned())
            });
        let proofs = self.epoch_proofs.clone();
        let proof_reads = self.proof_reads;
        store
            .expect_get_proof()
            .returning(move |epoch, index| match proof_reads {
                ProofReads::Stored => Ok(proofs.get(&(epoch, index)).cloned()),
                ProofReads::Undecodable => {
                    let error = bincode_codec().deserialize::<Proof>(&[0xff]).unwrap_err();
                    Err(Error::DBError(DBError::CodecError(error.into())))
                }
                ProofReads::Unreadable => Err(Error::Unexpected("unreadable proof".to_string())),
            });
        store
    }

    /// Open the database with the current binary, as the node does, reading
    /// exactly `certificate_reads` certificates from epochs if it migrates.
    ///
    /// A failed migration leaves the database openable by the previous build.
    fn migrate(&mut self, certificate_reads: usize) -> Result<(), DBOpenError> {
        let epochs = self.epoch_store(certificate_reads);
        let result = self.migrate_with(&epochs);
        drop(epochs);
        result
    }

    /// Like [`Self::migrate`], reading epochs from `epochs`.
    fn migrate_with(&mut self, epochs: &dyn EpochStoreReader) -> Result<(), DBOpenError> {
        drop(self.state.take());
        let result = StateStore::init_db(
            &self.state_dir.path,
            Some((self.pending_db.as_ref(), epochs)),
        );

        match result {
            Ok(db) => {
                self.state = Some(StateStore::new(Arc::new(db), BackupClient::noop()));
                Ok(())
            }
            Err(error) => {
                self.state = Some(open_pre_backfill(&self.state_dir.path));
                Err(error)
            }
        }
    }

    fn put_row(&self, network_id: u32, value: Value) {
        let (key, value) = StateStore::network_info_row(NetworkId::new(network_id), value);
        self.state()
            .db
            .put::<NetworkInfoColumn>(&key, &value)
            .unwrap();
    }

    fn row(&self, network_id: u32, kind: ValueDiscriminants) -> Option<NetworkInfoValue> {
        self.state()
            .db
            .get::<NetworkInfoColumn>(&network_info::Key { network_id, kind })
            .unwrap()
    }

    fn rows(&self) -> usize {
        self.state().db.keys::<NetworkInfoColumn>().unwrap().count()
    }

    fn info(&self, network_id: u32) -> agglayer_types::NetworkInfo {
        self.state()
            .get_network_info(NetworkId::new(network_id))
            .unwrap()
    }

    fn cached_settled_id(&self, network_id: u32) -> Option<CertificateId> {
        self.state()
            .get_latest_settled_certificate_id(NetworkId::new(network_id))
            .unwrap()
    }
}

fn settled_row(certificate: &Certificate) -> Value {
    StateStore::settled_certificate_value(
        certificate.hash(),
        certificate.new_local_exit_root,
        None,
        None,
    )
}

/// The stored settled local exit root of `network_id`.
fn stored_settled_ler(fixture: &Fixture, network_id: u32) -> Option<Vec<u8>> {
    match fixture
        .row(network_id, ValueDiscriminants::SettledCertificate)?
        .value?
    {
        Value::SettledCertificate(settled) => Some(settled.ler?.root.to_vec()),
        _ => None,
    }
}

/// The backfill error behind a failed migration.
fn backfill_error(result: Result<(), DBOpenError>) -> Error {
    match result {
        Err(DBOpenError::Migration(DBMigrationError {
            details: DBMigrationErrorDetails::Custom(report),
            ..
        })) => report
            .downcast::<Error>()
            .expect("the migration error must come from the backfill"),
        other => panic!("expected a failed backfill, got {other:?}"),
    }
}

fn assert_backfill_error(result: Result<(), DBOpenError>, network_id: u32) {
    match backfill_error(result) {
        Error::NetworkInfoBackfill {
            network_id: failed, ..
        } => assert_eq!(failed, NetworkId::new(network_id)),
        other => panic!("expected a network backfill error, got {other:?}"),
    }
}

#[test_log::test]
fn empty_column_is_filled_from_all_three_indexes() {
    let mut fixture = Fixture::new();
    let history = [certificate(1, 0, &[]), certificate(1, 1, &[])];
    fixture.settle(&history);
    let proven = certificate(1, 2, &[]);
    fixture.submit(&proven);
    fixture.prove(&proven);
    let pending = certificate(1, 3, &[]);
    fixture.submit(&pending);

    // The type comes from the latest settled certificate, and the empty
    // nullifier tree skips the claim scan.
    fixture.migrate(1).unwrap();

    let info = fixture.info(1);
    assert_eq!(info.network_type, NetworkType::Ecdsa);
    assert_eq!(info.latest_pending_certificate_id, Some(pending.hash()));
    assert_eq!(info.latest_pending_height, Some(pending.height));
    assert_eq!(fixture.cached_settled_id(1), Some(history[1].hash()));
    assert_eq!(info.settled_claim, None);
    assert!(fixture.row(1, ValueDiscriminants::SettledClaim).is_none());
    assert_eq!(
        fixture
            .state()
            .get_latest_proven_certificate_id(NetworkId::new(1))
            .unwrap(),
        Some(proven.hash())
    );
    assert_eq!(
        stored_settled_ler(&fixture, 1),
        Some(Digest::from(history[1].new_local_exit_root).to_vec())
    );
}

#[test_log::test]
fn historical_claim_is_found_by_the_reverse_scan_and_persisted() {
    let mut fixture = Fixture::new();
    let history = [
        certificate(1, 0, &[10]),
        certificate(1, 1, &[20, 21]),
        certificate(1, 2, &[]),
    ];
    fixture.settle(&history);

    // The settled certificate, then height 1.
    fixture.migrate(2).unwrap();

    let info = fixture.info(1);
    assert_eq!(fixture.cached_settled_id(1), Some(history[2].hash()));
    assert_eq!(info.settled_claim, claim_of(&history[1]));
    assert!(fixture.row(1, ValueDiscriminants::SettledClaim).is_some());
}

#[test_log::test]
fn never_claimed_network_reads_no_epoch_for_claim_discovery() {
    let mut fixture = Fixture::new();
    let history: Vec<_> = (0..5).map(|height| certificate(1, height, &[])).collect();
    fixture.settle(&history);

    // A scan would read all five certificates; only the network type is read.
    fixture.migrate(1).unwrap();

    let info = fixture.info(1);
    assert_eq!(fixture.cached_settled_id(1), Some(history[4].hash()));
    assert_eq!(info.settled_claim, None);
    assert!(fixture.row(1, ValueDiscriminants::SettledClaim).is_none());
}

#[test_log::test]
fn non_empty_nullifier_tree_without_a_settled_claim_scans_to_genesis() {
    let mut fixture = Fixture::new();
    let history = [certificate(1, 0, &[]), certificate(1, 1, &[])];
    fixture.settle(&history);
    // The next certificate's claim is already in the stored nullifier tree,
    // but that certificate is not settled yet.
    let state = fixture.state();
    let mut local_state = state
        .read_local_network_state(NetworkId::new(1))
        .unwrap()
        .unwrap();
    local_state
        .nullifier_tree
        .insert(
            NullifierKey::from(imported_exit(10).global_index),
            digest(1),
        )
        .unwrap();
    state
        .write_local_network_state(&NetworkId::new(1), &local_state, &[])
        .unwrap();

    // The settled certificate, then height 0.
    fixture.migrate(2).unwrap();

    assert_eq!(fixture.cached_settled_id(1), Some(history[1].hash()));
    assert_eq!(fixture.info(1).settled_claim, None);
}

#[test_log::test]
fn settled_network_without_a_stored_nullifier_tree_scans_its_history() {
    let mut fixture = Fixture::new();
    let history = [certificate(1, 0, &[10]), certificate(1, 1, &[])];
    fixture.settle(&history);
    fixture
        .state()
        .db
        .delete::<NullifierTreePerNetworkColumn>(&SmtKey {
            network_id: 1,
            key_type: SmtKeyType::Root,
        })
        .unwrap();

    // The settled certificate, then height 0.
    fixture.migrate(2).unwrap();

    assert_eq!(fixture.info(1).settled_claim, claim_of(&history[0]));
}

#[test_log::test]
fn malformed_nullifier_tree_fails() {
    let mut fixture = Fixture::new();
    fixture.settle(&[certificate(1, 0, &[])]);
    fixture
        .state()
        .db
        .put::<NullifierTreePerNetworkColumn>(
            &SmtKey {
                network_id: 1,
                key_type: SmtKeyType::Root,
            },
            &SmtValue::Leaf(digest(1)),
        )
        .unwrap();

    assert!(matches!(
        backfill_error(fixture.migrate(1)),
        Error::WrongValueType
    ));
    assert_eq!(fixture.rows(), 0);
}

#[test_log::test]
fn optional_settled_aggregates_are_recovered_when_available() {
    let mut fixture = Fixture::new();
    let mut settled = certificate(1, 0, &[]);
    settled.bridge_exits = (0..2).map(|seed| imported_exit(seed).bridge_exit).collect();
    let mut exit_tree = pessimistic_proof::unified_bridge::LocalExitTree::<32>::default();
    for exit in &settled.bridge_exits {
        exit_tree.add_leaf(exit.hash()).unwrap();
    }
    settled.new_local_exit_root = exit_tree.get_root().into();
    let output = PessimisticProofOutput {
        prev_local_exit_root: digest(1).into(),
        prev_pessimistic_root: digest(2),
        l1_info_root: digest(3),
        origin_network: NetworkId::new(1),
        aggchain_hash: digest(4),
        new_local_exit_root: settled.new_local_exit_root,
        new_pessimistic_root: digest(5),
    };
    let proof = proof_with_public_values(
        &PessimisticProofOutput::bincode_codec()
            .serialize(&output)
            .unwrap(),
    );
    fixture.epoch_proofs.insert(epoch_of(&settled), proof);
    fixture.settle(std::slice::from_ref(&settled));

    fixture.migrate(1).unwrap();

    let info = fixture.info(1);
    assert_eq!(info.settled_pp_root, Some(digest(5)));
    // The stored exit tree is still the one the certificate settled.
    assert_eq!(info.settled_let_leaf_count, Some(2));
}

#[test_log::test]
fn undecodable_proof_output_leaves_the_root_unset() {
    let mut fixture = Fixture::new();
    let settled = certificate(1, 0, &[]);
    fixture
        .epoch_proofs
        .insert(epoch_of(&settled), proof_with_public_values(&[0xff]));
    fixture.settle(std::slice::from_ref(&settled));

    fixture.migrate(1).unwrap();

    let info = fixture.info(1);
    assert_eq!(fixture.cached_settled_id(1), Some(settled.hash()));
    assert_eq!(info.settled_pp_root, None);
}

#[test_log::test]
fn missing_optional_aggregates_stay_absent_without_hiding_the_claim() {
    let mut fixture = Fixture::new();
    let settled = certificate(1, 0, &[10, 11]);
    fixture.settle(std::slice::from_ref(&settled));
    // The next certificate's leaf is already in the stored exit tree, so its
    // count no longer describes the settled certificate. No proof is stored.
    let state = fixture.state();
    let mut local_state = state
        .read_local_network_state(NetworkId::new(1))
        .unwrap()
        .unwrap();
    local_state.exit_tree.add_leaf(digest(9)).unwrap();
    state
        .write_local_network_state(&NetworkId::new(1), &local_state, &[digest(9)])
        .unwrap();

    fixture.migrate(1).unwrap();

    let info = fixture.info(1);
    assert_eq!(fixture.cached_settled_id(1), Some(settled.hash()));
    assert_eq!(info.settled_pp_root, None);
    assert_eq!(info.settled_let_leaf_count, None);
    assert_eq!(info.settled_claim, claim_of(&settled));
}

#[rstest]
#[case::pending_only(false)]
#[case::proven_only(true)]
#[test_log::test]
fn unsettled_network_opens_no_epoch(#[case] proven: bool) {
    let mut fixture = Fixture::new();
    let certificate = certificate(1, 0, &[10]);
    fixture
        .state()
        .insert_certificate_header(&certificate, CertificateStatus::Pending)
        .unwrap();
    // Keep the certificate body without the legacy pending pointer for the
    // proven-only case.
    fixture.store_pending_body(certificate.height, &certificate);
    if proven {
        fixture.prove(&certificate);
    } else {
        fixture
            .pending
            .set_latest_pending_certificate_per_network(
                &certificate.network_id,
                &certificate.height,
                &certificate.hash(),
            )
            .unwrap();
    }

    let mut epochs = MockEpochsStore::new();
    epochs.expect_get_certificate().never();
    epochs.expect_get_proof().never();
    fixture.migrate_with(&epochs).unwrap();
    drop(epochs);

    // A network that never settled has no type yet.
    let info = fixture.info(1);
    assert_eq!(fixture.row(1, ValueDiscriminants::NetworkType), None);
    assert_eq!(info.network_type, NetworkType::Unspecified);
    assert_eq!(fixture.cached_settled_id(1), None);
    assert_eq!(
        fixture
            .state()
            .get_latest_proven_certificate_id(NetworkId::new(1))
            .unwrap()
            .is_some(),
        proven
    );
    assert_eq!(info.latest_pending_certificate_id.is_some(), !proven);
}

#[test_log::test]
fn failed_backfill_writes_nothing_and_reruns_from_an_empty_column() {
    let mut fixture = Fixture::new();
    let first = [certificate(1, 0, &[10]), certificate(1, 1, &[])];
    let second = [certificate(2, 0, &[30]), certificate(2, 1, &[])];
    fixture.settle(&first);
    fixture.settle(&second);
    // The second network's settled certificate is missing from its epoch.
    let missing = fixture.epochs.remove(&epoch_of(&second[1])).unwrap();

    // The first network reads two certificates, the second one before
    // failing. Nothing from the first network is written.
    assert_backfill_error(fixture.migrate(3), 2);
    assert_eq!(fixture.rows(), 0);

    // Once repaired, the next open runs the whole backfill again.
    fixture.epochs.insert(epoch_of(&second[1]), missing);
    fixture.migrate(4).unwrap();
    assert_eq!(fixture.info(1).settled_claim, claim_of(&first[0]));
    assert_eq!(fixture.info(2).settled_claim, claim_of(&second[0]));
}

#[test_log::test]
fn backfill_runs_once_and_never_reads_the_legacy_indexes_again() {
    let mut fixture = Fixture::new();
    let first = certificate(1, 0, &[]);
    fixture.submit(&first);
    fixture.migrate(0).unwrap();
    assert_eq!(
        fixture.info(1).latest_pending_certificate_id,
        Some(first.hash())
    );

    // Only the legacy index moves, and the later open must not notice it.
    let next = certificate(1, 1, &[]);
    fixture.submit(&next);
    fixture.migrate(0).unwrap();
    assert_eq!(
        fixture.info(1).latest_pending_certificate_id,
        Some(first.hash())
    );

    // A build without the backfill step refuses the migrated database.
    drop(fixture.state.take());
    assert!(matches!(
        init_state_db_before_network_info_backfill(&fixture.state_dir.path),
        Err(DBOpenError::FewerStepsDeclared { .. })
    ));
}

#[test_log::test]
fn prebackfill_row_with_a_matching_settled_id_is_reset_and_reconstructed() {
    let mut fixture = Fixture::new();
    let history = [certificate(1, 0, &[10]), certificate(1, 1, &[])];
    fixture.settle(&history);
    // A feature build settled height 1 without backfilling the older claim,
    // and left a row for a network no legacy index knows.
    fixture.put_row(1, settled_row(&history[1]));
    fixture.put_row(1, Value::NetworkType(v0::NetworkType::Generic as i32));
    fixture.put_row(99, Value::NetworkType(v0::NetworkType::Generic as i32));

    // The settled certificate at height 1, then height 0.
    fixture.migrate(2).unwrap();

    assert_eq!(fixture.cached_settled_id(1), Some(history[1].hash()));
    assert_eq!(fixture.info(1).settled_claim, claim_of(&history[0]));
    assert_eq!(fixture.info(1).network_type, NetworkType::Ecdsa);
    assert_eq!(fixture.row(99, ValueDiscriminants::NetworkType), None);
}

#[test_log::test]
fn proven_pointer_stored_under_another_network_is_skipped() {
    let mut fixture = Fixture::new();
    let settled = certificate(2, 0, &[]);
    fixture.settle(std::slice::from_ref(&settled));
    let proven = certificate(2, 1, &[]);
    fixture.submit(&proven);
    fixture.prove(&proven);
    // A legacy row under network 9 whose value names network 2's settled
    // certificate: it matches a real header, and network 9 is written after
    // network 2, so trusting the value would replace network 2's pointer.
    fixture
        .pending_db
        .put::<LatestProvenCertificatePerNetworkColumn>(
            &NetworkId::new(9),
            &ProvenCertificate(settled.hash(), NetworkId::new(2), Height::ZERO),
        )
        .unwrap();

    fixture.migrate(1).unwrap();

    assert_eq!(
        fixture
            .state()
            .get_latest_proven_certificate_id(NetworkId::new(2))
            .unwrap(),
        Some(proven.hash())
    );
    assert_eq!(
        fixture.row(9, ValueDiscriminants::LatestProvenCertificateInfo),
        None
    );
}

#[rstest]
#[case::absent_header(1, 0, false)]
#[case::foreign_header(2, 1, true)]
#[case::wrong_height(1, 5, true)]
#[test_log::test]
fn unsettled_pointer_without_a_matching_header_is_skipped(
    #[case] header_network: u32,
    #[case] pointer_height: u64,
    #[case] store_header: bool,
    #[values(false, true)] proven: bool,
) {
    let mut fixture = Fixture::new();
    let settled = certificate(1, 0, &[]);
    fixture.settle(std::slice::from_ref(&settled));
    let junk = certificate(header_network, 1, &[]);
    if store_header {
        fixture
            .state()
            .insert_certificate_header(&junk, CertificateStatus::Pending)
            .unwrap();
    }
    if proven {
        fixture
            .pending
            .set_latest_proven_certificate_per_network(
                &NetworkId::new(1),
                &Height::new(pointer_height),
                &junk.hash(),
            )
            .unwrap();
    } else {
        fixture
            .pending
            .set_latest_pending_certificate_per_network(
                &NetworkId::new(1),
                &Height::new(pointer_height),
                &junk.hash(),
            )
            .unwrap();
    }

    fixture.migrate(1).unwrap();

    assert_eq!(
        fixture.row(1, ValueDiscriminants::LatestPendingCertificateInfo),
        None
    );
    assert_eq!(
        fixture.row(1, ValueDiscriminants::LatestProvenCertificateInfo),
        None
    );
    assert_eq!(fixture.cached_settled_id(1), Some(settled.hash()));
    assert_eq!(fixture.info(1).network_type, NetworkType::Ecdsa);
}

#[test_log::test]
fn network_with_only_a_junk_pointer_gets_no_rows() {
    let mut fixture = Fixture::new();
    fixture.prove(&certificate(1, 0, &[]));

    fixture.migrate(0).unwrap();

    assert_eq!(fixture.rows(), 0);
}

#[rstest]
#[case::absent_header(1, 0, false)]
#[case::foreign_header(2, 0, true)]
#[case::wrong_height(1, 5, true)]
#[test_log::test]
fn settled_pointer_without_a_matching_header_fails(
    #[case] header_network: u32,
    #[case] pointer_height: u64,
    #[case] store_header: bool,
) {
    let mut fixture = Fixture::new();
    let certificate = certificate(header_network, 0, &[]);
    if store_header {
        fixture
            .state()
            .insert_certificate_header(&certificate, CertificateStatus::Settled)
            .unwrap();
    }
    fixture
        .state()
        .db
        .put::<LatestSettledCertificatePerNetworkColumn>(
            &NetworkId::new(1),
            &SettledCertificate(
                certificate.hash(),
                Height::new(pointer_height),
                EpochNumber::ZERO,
                CertificateIndex::ZERO,
            ),
        )
        .unwrap();

    assert_backfill_error(fixture.migrate(0), 1);
}

fn multisig(mut certificate: Certificate) -> Certificate {
    certificate.aggchain_data = AggchainData::MultisigOnly {
        multisig: MultisigPayload(Vec::new()),
    };
    certificate
}

#[test_log::test]
fn network_type_comes_from_the_latest_settled_certificate() {
    let mut fixture = Fixture::new();
    // The network moved from ECDSA to multisig, and newer certificates that
    // are only pending or proven do not decide its type.
    let history = [certificate(1, 0, &[]), multisig(certificate(1, 1, &[]))];
    fixture.settle(&history);
    let proven = certificate(1, 2, &[]);
    fixture.submit(&proven);
    fixture.prove(&proven);
    fixture.submit(&certificate(1, 3, &[]));

    fixture.migrate(1).unwrap();

    assert_eq!(fixture.info(1).network_type, NetworkType::MultisigOnly);
}

#[test_log::test]
fn another_certificate_in_the_settled_slot_fails() {
    let mut fixture = Fixture::new();
    let settled = certificate(1, 0, &[]);
    fixture.settle(std::slice::from_ref(&settled));
    let mut other = multisig(settled.clone());
    other.metadata = Metadata::new(digest(7));
    assert_ne!(other.hash(), settled.hash());
    fixture.epochs.insert(epoch_of(&settled), other);

    assert_backfill_error(fixture.migrate(1), 1);
}

#[test_log::test]
fn settled_certificate_missing_from_its_epoch_fails() {
    let mut fixture = Fixture::new();
    let settled = certificate(1, 0, &[]);
    fixture.settle(std::slice::from_ref(&settled));
    fixture.epochs.clear();

    assert_backfill_error(fixture.migrate(1), 1);
    assert_eq!(fixture.rows(), 0);
}

#[test_log::test]
fn unreadable_epoch_fails() {
    let mut fixture = Fixture::new();
    fixture.settle(&[certificate(1, 0, &[])]);
    fixture.unreadable_epochs = true;

    assert!(matches!(
        backfill_error(fixture.migrate(1)),
        Error::Unexpected(_)
    ));
    assert_eq!(fixture.rows(), 0);
}

#[test_log::test]
fn unreadable_local_exit_tree_fails() {
    use crate::columns::local_exit_tree_per_network::{self as LET, LocalExitTreePerNetworkColumn};

    let mut fixture = Fixture::new();
    fixture.settle(&[certificate(1, 0, &[])]);
    // A frontier layer holding a leaf count makes the exit tree unreadable.
    fixture
        .state()
        .db
        .put::<LocalExitTreePerNetworkColumn>(
            &LET::Key {
                network_id: 1,
                key_type: LET::KeyType::Frontier(0),
            },
            &LET::Value::LeafCount(0),
        )
        .unwrap();

    assert!(matches!(
        backfill_error(fixture.migrate(1)),
        Error::InconsistentFrontier
    ));
    assert_eq!(fixture.rows(), 0);
}

#[derive(Debug)]
enum HistoryGap {
    MissingHeight,
    NoEpochAssignment,
    MissingFromItsEpoch,
    ReplacedInItsEpoch,
}

#[rstest]
#[case::missing_height(HistoryGap::MissingHeight, 1)]
#[case::no_epoch_assignment(HistoryGap::NoEpochAssignment, 1)]
#[case::missing_from_its_epoch(HistoryGap::MissingFromItsEpoch, 2)]
#[case::replaced_in_its_epoch(HistoryGap::ReplacedInItsEpoch, 2)]
#[test_log::test]
fn gap_in_the_settled_history_fails_the_scan(
    #[case] gap: HistoryGap,
    #[case] certificate_reads: usize,
) {
    let mut fixture = Fixture::new();
    let history = [
        certificate(1, 0, &[10]),
        certificate(1, 1, &[]),
        certificate(1, 2, &[]),
    ];
    fixture.settle(&history);
    let broken = &history[1];
    let db = &fixture.state().db;
    match gap {
        HistoryGap::MissingHeight => db
            .delete::<CertificatePerNetworkColumn>(&certificate_per_network::Key {
                network_id: 1,
                height: broken.height,
            })
            .unwrap(),
        HistoryGap::NoEpochAssignment => {
            let mut header = db
                .get::<CertificateHeaderColumn>(&broken.hash())
                .unwrap()
                .unwrap();
            header.epoch_number = None;
            db.put::<CertificateHeaderColumn>(&broken.hash(), &header)
                .unwrap();
        }
        HistoryGap::MissingFromItsEpoch => {
            fixture.epochs.remove(&epoch_of(broken));
        }
        HistoryGap::ReplacedInItsEpoch => {
            let mut other = broken.clone();
            other.metadata = Metadata::new(digest(7));
            fixture.epochs.insert(epoch_of(broken), other);
        }
    }

    // The settled certificate at height 2 carries no claim; the scan stops at
    // the gap in height 1, before the claim at height 0.
    assert_backfill_error(fixture.migrate(certificate_reads), 1);
    assert_eq!(fixture.rows(), 0);
}

#[test_log::test]
fn scan_rejects_a_history_entry_of_another_network() {
    let mut fixture = Fixture::new();
    let history = [
        certificate(1, 0, &[10]),
        certificate(1, 1, &[]),
        certificate(1, 2, &[]),
    ];
    fixture.settle(&history);
    // The per-height index at height 1 names a certificate of network 2.
    let other = certificate(2, 1, &[]);
    fixture
        .state()
        .insert_certificate_header(&other, CertificateStatus::Pending)
        .unwrap();
    fixture
        .state()
        .db
        .put::<CertificatePerNetworkColumn>(
            &certificate_per_network::Key {
                network_id: 1,
                height: Height::new(1),
            },
            &other.hash(),
        )
        .unwrap();

    assert_backfill_error(fixture.migrate(1), 1);
}

#[test_log::test]
fn every_pointer_naming_the_settled_certificate_is_kept() {
    let mut fixture = Fixture::new();
    let settled = certificate(1, 0, &[]);
    fixture.settle(std::slice::from_ref(&settled));
    fixture
        .pending
        .set_latest_pending_certificate_per_network(
            &settled.network_id,
            &settled.height,
            &settled.hash(),
        )
        .unwrap();
    fixture.prove(&settled);

    fixture.migrate(1).unwrap();

    let info = fixture.info(1);
    assert_eq!(info.latest_pending_certificate_id, Some(settled.hash()));
    assert_eq!(fixture.cached_settled_id(1), Some(settled.hash()));
    assert_eq!(
        fixture
            .state()
            .get_latest_proven_certificate_id(NetworkId::new(1))
            .unwrap(),
        Some(settled.hash())
    );
}

#[test_log::test]
fn backfill_reads_real_epoch_databases() {
    use crate::{
        columns::epochs::certificates::CertificatePerIndexProtoColumn,
        stores::{epochs::EpochsStore, per_epoch::PerEpochStore},
    };

    let mut fixture = Fixture::new();
    let history = [
        multisig(certificate(1, 0, &[10])),
        multisig(certificate(1, 1, &[])),
        multisig(certificate(1, 2, &[])),
    ];
    fixture.settle(&history);
    let epochs_dir = TempDBDir::new();
    let config = Arc::new(agglayer_config::Config::new(&epochs_dir.path));
    for ((epoch, index), certificate) in &fixture.epochs {
        let path = config.storage.epoch_db_path(*epoch);
        std::fs::create_dir_all(&path).unwrap();
        PerEpochStore::<(), ()>::init_db(&path)
            .unwrap()
            .put::<CertificatePerIndexProtoColumn>(index, certificate)
            .unwrap();
    }

    // The type and the scan read three epochs, switching between them.
    fixture
        .migrate_with(&EpochsStore::read_only(config))
        .unwrap();

    let info = fixture.info(1);
    assert_eq!(info.network_type, NetworkType::MultisigOnly);
    assert_eq!(fixture.cached_settled_id(1), Some(history[2].hash()));
    assert_eq!(info.settled_claim, claim_of(&history[0]));
}

#[test_log::test]
fn undecodable_stored_proof_leaves_the_root_unset() {
    let mut fixture = Fixture::new();
    let settled = certificate(1, 0, &[]);
    fixture.settle(std::slice::from_ref(&settled));
    fixture.proof_reads = ProofReads::Undecodable;

    fixture.migrate(1).unwrap();

    assert_eq!(fixture.cached_settled_id(1), Some(settled.hash()));
    assert_eq!(fixture.info(1).settled_pp_root, None);
}

#[test_log::test]
#[should_panic(expected = "needs the pending and epoch stores")]
fn open_without_sources_panics_on_a_database_with_certificates() {
    let mut fixture = Fixture::new();
    fixture.settle(&[certificate(1, 0, &[10])]);
    drop(fixture.state.take());

    let _ = StateStore::init_db(&fixture.state_dir.path, None);
}

#[test_log::test]
fn unreadable_proof_fails() {
    let mut fixture = Fixture::new();
    fixture.settle(&[certificate(1, 0, &[])]);
    fixture.proof_reads = ProofReads::Unreadable;

    assert!(matches!(
        backfill_error(fixture.migrate(1)),
        Error::Unexpected(_)
    ));
    assert_eq!(fixture.rows(), 0);
}

#[test_log::test]
fn settled_network_without_a_stored_exit_tree_leaves_the_leaf_count_unset() {
    use crate::columns::local_exit_tree_per_network::{self as LET, LocalExitTreePerNetworkColumn};

    let mut fixture = Fixture::new();
    let settled = certificate(1, 0, &[]);
    fixture.settle(std::slice::from_ref(&settled));
    fixture
        .state()
        .db
        .delete::<LocalExitTreePerNetworkColumn>(&LET::Key {
            network_id: 1,
            key_type: LET::KeyType::LeafCount,
        })
        .unwrap();

    fixture.migrate(1).unwrap();

    assert_eq!(fixture.cached_settled_id(1), Some(settled.hash()));
    assert_eq!(fixture.info(1).settled_let_leaf_count, None);
}
