use std::sync::Arc;

use agglayer_types::{
    aggchain_data::CertificateAggchainDataCtx, primitives::Hashable as _, Certificate,
    CertificateId, CertificateIndex, Digest, EpochNumber, Height, L1WitnessCtx,
    LocalNetworkStateData, NetworkId, PessimisticRootInput,
};
use pessimistic_proof::{
    core::{
        commitment::{PessimisticRootCommitmentVersion, SignatureCommitmentVersion},
        generate_pessimistic_proof,
    },
    LocalNetworkState,
};
use rstest::{fixture, rstest};
use tracing::info;

use super::cf_definitions;
use crate::{
    backup::BackupClient,
    columns::latest_settled_certificate_per_network::{
        LatestSettledCertificatePerNetworkColumn, SettledCertificate,
    },
    error::Error,
    schema::ColumnSchema as _,
    storage::DB,
    stores::{state::StateStore, StateReader as _, StateWriter as _},
    tests::TempDBDir,
};

mod disabled_networks;
mod metadata;
mod settlement;

#[test]
fn init_db_adds_legacy_missing_cfs() {
    use std::collections::BTreeSet;

    use crate::columns::{
        disabled_networks::DisabledNetworksColumn,
        settlement_attempt_per_wallet::SettlementAttemptPerWalletColumn,
        settlement_attempt_results::SettlementAttemptResultsColumn,
        settlement_attempts::SettlementAttemptsColumn,
        settlement_job_id_per_certificate_id::SettlementJobIdPerCertificateIdColumn,
        settlement_job_results::SettlementJobResultsColumn, settlement_jobs::SettlementJobsColumn,
    };

    // Simulate a pre-`disabled_networks_cf` snapshot: an existing rocksdb
    // directory whose schema is exactly STATE_DB_V0. The current binary
    // must open it without UnexpectedSchema and end up with every CF
    // declared in STATE_DB.
    let tmp = TempDBDir::new();
    {
        let _legacy = DB::open_cf(tmp.path.as_path(), cf_definitions::STATE_DB_V0)
            .expect("V0 schema initialization should succeed");
    }

    let db = StateStore::init_db(tmp.path.as_path())
        .expect("init_db should ensure the missing legacy CFs are added");

    // After init_db, every CF in STATE_DB_LEGACY_ADD_CFS exists on disk.
    let post_cfs: BTreeSet<&str> =
        rocksdb::DB::list_cf(&rocksdb::Options::default(), tmp.path.as_path())
            .expect("list_cf should succeed after init_db")
            .into_iter()
            .map(|s| Box::leak(s.into_boxed_str()) as &str)
            .collect();

    for expected in [
        DisabledNetworksColumn::COLUMN_FAMILY_NAME,
        SettlementJobIdPerCertificateIdColumn::COLUMN_FAMILY_NAME,
        SettlementJobsColumn::COLUMN_FAMILY_NAME,
        SettlementJobResultsColumn::COLUMN_FAMILY_NAME,
        SettlementAttemptsColumn::COLUMN_FAMILY_NAME,
        SettlementAttemptResultsColumn::COLUMN_FAMILY_NAME,
        SettlementAttemptPerWalletColumn::COLUMN_FAMILY_NAME,
    ] {
        assert!(
            post_cfs.contains(expected),
            "expected legacy-add CF {expected:?} to be present after init_db; got {post_cfs:?}"
        );
    }
    drop(db);
}

#[test]
fn init_db_adds_settlement_job_id_per_certificate_id_cf_to_v1_schema() {
    use std::collections::BTreeSet;

    use crate::columns::settlement_job_id_per_certificate_id::SettlementJobIdPerCertificateIdColumn;

    let tmp = TempDBDir::new();
    {
        let previous_schema = DB::builder(tmp.path.as_path(), cf_definitions::STATE_DB_V0)
            .expect("V0 schema initialization should succeed")
            .ensure_cfs(cf_definitions::STATE_DB_V1_ADDED_CFS)
            .expect("V1 schema migration should succeed")
            .finalize(cf_definitions::STATE_DB)
            .expect("V1 schema finalization should succeed");
        drop(previous_schema);
    }

    let db = StateStore::init_db(tmp.path.as_path())
        .expect("init_db should ensure the certificate settlement job CF is added");

    let post_cfs: BTreeSet<&str> =
        rocksdb::DB::list_cf(&rocksdb::Options::default(), tmp.path.as_path())
            .expect("list_cf should succeed after init_db")
            .into_iter()
            .map(|s| Box::leak(s.into_boxed_str()) as &str)
            .collect();

    assert!(
        post_cfs.contains(SettlementJobIdPerCertificateIdColumn::COLUMN_FAMILY_NAME),
        "expected CF {:?} to be present after init_db; got {post_cfs:?}",
        SettlementJobIdPerCertificateIdColumn::COLUMN_FAMILY_NAME
    );
    drop(db);
}

#[test]
fn init_db_creates_certificate_id_per_settlement_job_id_cf() {
    use crate::columns::certificate_id_per_settlement_job_id::CertificateIdPerSettlementJobIdColumn;

    let tmp = crate::tests::TempDBDir::new();
    let db = StateStore::init_db(tmp.path.as_path()).expect("init db");
    let cfs = rocksdb::DB::list_cf(&rocksdb::Options::default(), tmp.path.as_path())
        .expect("list cf names");
    assert!(
        cfs.contains(&CertificateIdPerSettlementJobIdColumn::COLUMN_FAMILY_NAME.to_string()),
        "expected reverse CF {} to exist",
        CertificateIdPerSettlementJobIdColumn::COLUMN_FAMILY_NAME
    );
    drop(db);
}

#[test]
fn init_db_is_idempotent_on_current_schema() {
    // Opening init_db twice on a fresh DB must succeed: after the first
    // open records the ensure_cfs step, the second call sees the current
    // schema and skips that already-recorded step.
    let tmp = TempDBDir::new();
    let db = StateStore::init_db(tmp.path.as_path()).expect("first init_db");
    drop(db);
    let db = StateStore::init_db(tmp.path.as_path())
        .expect("second init_db should be a no-op on the already-current schema");
    drop(db);
}

#[test]
fn can_retrieve_list_of_network() {
    let tmp = TempDBDir::new();
    let db = Arc::new(StateStore::init_db(tmp.path.as_path()).unwrap());
    let store = StateStore::new(db.clone(), BackupClient::noop());
    assert!(store.get_active_networks().unwrap().is_empty());

    db.put::<LatestSettledCertificatePerNetworkColumn>(
        &1.into(),
        &SettledCertificate(
            CertificateId::new([0; 32].into()),
            Height::ZERO,
            EpochNumber::ZERO,
            CertificateIndex::ZERO,
        ),
    )
    .expect("Unable to put certificate into storage");
    assert!(store.get_active_networks().unwrap().len() == 1);
}

fn equal_state(lhs: &LocalNetworkStateData, rhs: &LocalNetworkStateData) -> bool {
    // local exit tree
    assert_eq!(lhs.exit_tree.leaf_count(), rhs.exit_tree.leaf_count());
    assert_eq!(lhs.exit_tree.get_root(), rhs.exit_tree.get_root());

    // balance tree
    assert_eq!(lhs.balance_tree.root, rhs.balance_tree.root);
    assert_eq!(lhs.balance_tree.tree, rhs.balance_tree.tree);

    // nullifier tree
    assert_eq!(lhs.nullifier_tree.root, rhs.nullifier_tree.root);
    assert_eq!(lhs.nullifier_tree.tree, rhs.nullifier_tree.tree);

    true
}

#[fixture]
pub(crate) fn network_id() -> NetworkId {
    0.into()
}

#[fixture]
pub(crate) fn store() -> StateStore {
    let _ = test_log::tracing_subscriber::fmt()
        .with_test_writer()
        .with_env_filter(test_log::tracing_subscriber::EnvFilter::from_default_env())
        .try_init();

    tracing::info!("Setting up storage fixture");

    let tmp = TempDBDir::new();
    tracing::debug!(path = ?tmp.path, "Temporary directory created");
    let db = Arc::new(StateStore::init_db(tmp.path.as_path()).unwrap());

    StateStore::new(db.clone(), BackupClient::noop())
}

#[rstest]
fn can_handle_empty_state(#[from(network_id)] unknown_network_id: NetworkId, store: StateStore) {
    // return none for unknown network
    assert!(matches!(
        store.read_local_network_state(unknown_network_id),
        Ok(None)
    ));

    // can write one state from scratch
    assert!(store
        .write_local_network_state(&unknown_network_id, &LocalNetworkStateData::default(), &[])
        .is_ok());
}

#[rstest]
fn can_retrieve_state(network_id: NetworkId, store: StateStore) {
    // write arbitrary state
    let mut lns = LocalNetworkStateData::default();
    let leaves = (0..10).map(|_| Digest([5u8; 32])).collect::<Vec<_>>();
    for l in &leaves {
        lns.exit_tree.add_leaf(*l).unwrap();
    }

    assert!(store
        .write_local_network_state(&network_id, &lns, leaves.as_slice())
        .is_ok());

    // retrieve it
    assert!(
        matches!(store.read_local_network_state(network_id), Ok(Some(retrieved)) if equal_state(&lns, &retrieved))
    );
}

#[rstest]
fn can_update_existing_state(network_id: NetworkId, store: StateStore) {
    let mut lns = LocalNetworkStateData::default();

    // write initial state
    assert!(store
        .write_local_network_state(&network_id, &lns, &[])
        .is_ok());

    // update state
    let bridge_exit = [5u8; 32];
    lns.exit_tree.add_leaf(bridge_exit.into()).unwrap();

    // write new state
    assert!(store
        .write_local_network_state(&network_id, &lns, &[Digest(bridge_exit)])
        .is_ok());

    // retrieve new state
    assert!(
        matches!(store.read_local_network_state(network_id), Ok(Some(retrieved)) if equal_state(&lns, &retrieved))
    );
}

#[rstest]
fn can_detect_inconsistent_state(network_id: NetworkId, store: StateStore) {
    let mut lns = LocalNetworkStateData::default();

    // write initial state
    assert!(store
        .write_local_network_state(&network_id, &lns, &[])
        .is_ok());

    // update state
    let bridge_exit = [5u8; 32];
    lns.exit_tree.add_leaf(bridge_exit.into()).unwrap();

    // write new state with missing leaves
    assert!(matches!(
        store.write_local_network_state(&network_id, &lns, &[]),
        Err(Error::InconsistentState { .. })
    ));
}

use pessimistic_proof_test_suite::sample_data::{self as data};

#[rstest]
fn can_read(network_id: NetworkId, store: StateStore) {
    let certificates: Vec<Certificate> = [
        "n15-cert_h0.json",
        "n15-cert_h1.json",
        "n15-cert_h2.json",
        "n15-cert_h3.json",
    ]
    .iter()
    .map(|p| data::load_certificate(p))
    .collect();

    let mut leaves: Vec<Digest> = Vec::new();
    let mut lns = LocalNetworkStateData::default();

    for (idx, certificate) in certificates.iter().enumerate() {
        info!(
            "Certificate ({idx}|{}) | {}, nib:{} b:{}",
            certificate.height,
            certificate.hash(),
            certificate.imported_bridge_exits.len(),
            certificate.bridge_exits.len(),
        );

        let signer = certificate
            .retrieve_signer(SignatureCommitmentVersion::V2)
            .unwrap();
        let l1_info_root = certificate.l1_info_root().unwrap().unwrap_or_default();

        let ctx_from_l1 = L1WitnessCtx {
            l1_info_root,
            prev_pessimistic_root: PessimisticRootInput::Computed(
                PessimisticRootCommitmentVersion::V2,
            ),
            aggchain_data_ctx: CertificateAggchainDataCtx::LegacyEcdsa { signer },
        };

        let multi_batch_header = lns
            .make_multi_batch_header(certificate, ctx_from_l1.clone())
            .unwrap();

        info!("Certificate {idx}: successful witness generation");
        let initial_state = LocalNetworkState::from(lns.clone());

        generate_pessimistic_proof(initial_state.into(), &multi_batch_header).unwrap();
        info!("Certificate {idx}: successful native execution");

        for b in &certificate.bridge_exits {
            leaves.push(b.hash());
        }
        lns.apply_certificate(certificate, ctx_from_l1).unwrap();
        info!("Certificate {idx}: successful state transition, waiting for the next");
    }

    let mut before_going_through_disk = lns.clone();

    info!(
        "before DB | root: {}, nb nodes: {}",
        before_going_through_disk.balance_tree.root,
        before_going_through_disk.balance_tree.tree.len()
    );

    before_going_through_disk
        .balance_tree
        .traverse_and_prune()
        .unwrap();
    before_going_through_disk
        .nullifier_tree
        .traverse_and_prune()
        .unwrap();

    info!(
        "before DB (pruned) | root: {}, nb nodes: {}",
        before_going_through_disk.balance_tree.root,
        before_going_through_disk.balance_tree.tree.len()
    );

    // write state
    assert!(store
        .write_local_network_state(&network_id, &before_going_through_disk, leaves.as_slice())
        .is_ok());

    // read state
    let after_going_through_disk = store.read_local_network_state(network_id).unwrap().unwrap();

    info!(
        "after DB | root: {}, nb nodes: {}",
        after_going_through_disk.balance_tree.root,
        after_going_through_disk.balance_tree.tree.len()
    );

    // check that the read succeed and is equal to the state prior to passing by
    // the disk
    assert!(equal_state(
        &before_going_through_disk,
        &after_going_through_disk
    ));
}

#[test]
fn import_native_tokens() {
    let certificates: Vec<Certificate> = ["cert_h0.json", "cert_h1.json", "cert_h2.json"]
        .iter()
        .map(|p| data::load_certificate(p))
        .collect();

    let mut lns = LocalNetworkStateData::default();

    for (idx, certificate) in certificates.iter().enumerate() {
        info!(
            "Certificate ({idx}|{}) | {}, nib:{} b:{}",
            certificate.height,
            certificate.hash(),
            certificate.imported_bridge_exits.len(),
            certificate.bridge_exits.len(),
        );

        let signer = certificate
            .retrieve_signer(SignatureCommitmentVersion::V2)
            .unwrap();
        let l1_info_root = certificate.l1_info_root().unwrap().unwrap_or_default();

        let ctx_from_l1 = L1WitnessCtx {
            l1_info_root,
            prev_pessimistic_root: PessimisticRootInput::Computed(
                PessimisticRootCommitmentVersion::V2,
            ),
            aggchain_data_ctx: CertificateAggchainDataCtx::LegacyEcdsa { signer },
        };
        let multi_batch_header = lns
            .make_multi_batch_header(certificate, ctx_from_l1.clone())
            .unwrap();

        info!("Certificate {idx}: successful witness generation");
        let initial_state = LocalNetworkState::from(lns.clone());

        generate_pessimistic_proof(initial_state.into(), &multi_batch_header).unwrap();
        info!("Certificate {idx}: successful native execution");

        lns.apply_certificate(certificate, ctx_from_l1).unwrap();
        info!("Certificate {idx}: successful state transition, waiting for the next");
    }
}

#[rstest]
#[case("n15-cert_h0")]
#[case("n15-cert_h1")]
#[case("n15-cert_h2")]
#[case("n15-cert_h3")]
fn certificate_serialization(#[case] cert_name: &str) {
    use crate::schema::Codec;

    let certificate = data::load_certificate(&format!("{cert_name}.json"));
    let encoded = certificate.encode().unwrap();
    let hash = pessimistic_proof::keccak::keccak256(&encoded);
    insta::assert_debug_snapshot!(cert_name, hash);
}
