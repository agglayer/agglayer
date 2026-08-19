use std::{path::PathBuf, sync::Arc};

use agglayer_types::{
    primitives::Hashable as _, Address, Certificate, CertificateHeader, CertificateIndex,
    CertificateStatus, Digest, EpochNumber, Height, LocalNetworkStateData, NetworkId,
    SettlementTxHash, U256,
};
use pessimistic_proof::unified_bridge::{
    BridgeExit, Claim, ClaimFromMainnet, ClaimFromRollup, GlobalIndex, ImportedBridgeExit,
    L1InfoTreeLeaf, L1InfoTreeLeafInner, LeafType, MerkleProof, TokenInfo,
};

use super::{
    DebugCertificateFallbackReason, SettledCertificateSnapshot, TreeSnapshotError,
    TreeSnapshotReader, TreeSnapshotWarning,
};
use crate::{
    backup::BackupClient,
    columns::{
        balance_tree_per_network::BalanceTreePerNetworkColumn,
        certificate_header::CertificateHeaderColumn,
        certificate_per_network::{self, CertificatePerNetworkColumn},
        debug_certificates::DebugCertificatesProtoColumn,
        epochs::certificates::CertificatePerIndexProtoColumn,
        latest_settled_certificate_per_network::{
            LatestSettledCertificatePerNetworkColumn, SettledCertificate,
        },
    },
    storage::DB,
    stores::{
        debug::DebugStore, per_epoch::cf_definitions::EPOCHS_DB, state::StateStore,
        DebugWriter as _, StateWriter as _,
    },
    tests::TempDBDir,
    types::{SmtKey, SmtKeyType, SmtValue},
};

struct Fixture {
    _root: TempDBDir,
    network_id: NetworkId,
    certificate: Certificate,
    second_certificate: Certificate,
    unsettled_certificate: Certificate,
    settlement_tx_hash: SettlementTxHash,
    epoch_number: EpochNumber,
    certificate_index: CertificateIndex,
    second_certificate_index: CertificateIndex,
    token: TokenInfo,
    balance: U256,
}

fn fixture_settlement_tx_hash() -> SettlementTxHash {
    SettlementTxHash::new(Digest::from([0xaa; 32]))
}

fn proof(root: Digest, seed: u8) -> MerkleProof {
    MerkleProof::new(root, [Digest::from([seed; 32]); 32])
}

fn l1_info_leaf(seed: u8) -> L1InfoTreeLeaf {
    let mut leaf = L1InfoTreeLeaf {
        l1_info_tree_index: u32::from(seed),
        rer: Digest::from([seed.wrapping_add(1); 32]),
        mer: Digest::from([seed.wrapping_add(2); 32]),
        inner: L1InfoTreeLeafInner {
            global_exit_root: Digest::default(),
            block_hash: Digest::from([seed.wrapping_add(3); 32]),
            timestamp: 1_700_000_000 + u64::from(seed),
        },
    };
    leaf.inner.global_exit_root = leaf.ger();
    leaf
}

fn imported_bridge_exit(
    source_network: NetworkId,
    destination_network: NetworkId,
    seed: u8,
) -> ImportedBridgeExit {
    let bridge_exit = BridgeExit {
        leaf_type: LeafType::Transfer,
        token_info: TokenInfo {
            origin_network: NetworkId::new(2),
            origin_token_address: Address::from([seed; 20]),
        },
        dest_network: destination_network,
        dest_address: Address::from([seed.wrapping_add(1); 20]),
        amount: U256::from(u64::from(seed) + 1_000),
        metadata: Some(Digest::from([seed.wrapping_add(2); 32])),
    };
    let l1_leaf = l1_info_leaf(seed);
    let l1_info_root = Digest::from([seed.wrapping_add(4); 32]);
    let claim_data = if source_network == NetworkId::ETH_L1 {
        Claim::Mainnet(Box::new(ClaimFromMainnet {
            proof_leaf_mer: proof(l1_leaf.mer, seed.wrapping_add(5)),
            proof_ger_l1root: proof(l1_info_root, seed.wrapping_add(6)),
            l1_leaf,
        }))
    } else {
        Claim::Rollup(Box::new(ClaimFromRollup {
            proof_leaf_ler: proof(
                Digest::from([seed.wrapping_add(7); 32]),
                seed.wrapping_add(8),
            ),
            proof_ler_rer: proof(l1_leaf.rer, seed.wrapping_add(9)),
            proof_ger_l1root: proof(l1_info_root, seed.wrapping_add(10)),
            l1_leaf,
        }))
    };

    // `GlobalIndex::new` takes the zero-based rollup index in its `NetworkId`
    // argument in this protocol version, while `network_id()` returns the
    // externally visible one-based network ID.
    let global_index_network = if source_network == NetworkId::ETH_L1 {
        source_network
    } else {
        NetworkId::new(source_network.to_u32() - 1)
    };

    ImportedBridgeExit {
        bridge_exit,
        claim_data,
        global_index: GlobalIndex::new(global_index_network, u32::from(seed) + 100),
    }
}

fn seed_snapshot(settlement_tx_hash: Option<SettlementTxHash>) -> Fixture {
    let root = TempDBDir::new();
    let state_path = root.path.join("state");
    let epochs_path = root.path.join("epochs");
    std::fs::create_dir_all(&epochs_path).unwrap();

    let network_id = NetworkId::new(7);
    let token = TokenInfo {
        origin_network: NetworkId::new(2),
        origin_token_address: Address::from([0x22; 20]),
    };
    let bridge_exit = BridgeExit {
        leaf_type: LeafType::Transfer,
        token_info: token,
        dest_network: NetworkId::new(8),
        dest_address: Address::from([0x88; 20]),
        amount: U256::from(123u64),
        metadata: Some(Digest::from([0x44; 32])),
    };

    let mut certificate = Certificate {
        network_id,
        height: Height::ZERO,
        bridge_exits: vec![bridge_exit],
        imported_bridge_exits: vec![imported_bridge_exit(NetworkId::ETH_L1, network_id, 0x31)],
        prev_local_exit_root: LocalNetworkStateData::default().exit_tree.get_root().into(),
        ..Default::default()
    };

    let mut state = LocalNetworkStateData::default();
    let leaf_hash = certificate.bridge_exits[0].hash();
    state.exit_tree.add_leaf(leaf_hash).unwrap();
    certificate.new_local_exit_root = state.exit_tree.get_root().into();

    let second_certificate = Certificate {
        network_id,
        height: Height::new(1),
        prev_local_exit_root: certificate.new_local_exit_root,
        new_local_exit_root: certificate.new_local_exit_root,
        imported_bridge_exits: vec![imported_bridge_exit(NetworkId::new(4), network_id, 0x32)],
        ..Default::default()
    };

    let unsettled_certificate = Certificate {
        network_id,
        height: Height::new(2),
        prev_local_exit_root: certificate.new_local_exit_root,
        new_local_exit_root: certificate.new_local_exit_root,
        imported_bridge_exits: vec![imported_bridge_exit(NetworkId::new(5), network_id, 0x33)],
        ..Default::default()
    };

    let balance = U256::from(987_654u64);
    state
        .balance_tree
        .insert(token, Digest::from(balance.to_be_bytes::<32>()))
        .unwrap();

    let certificate_id = certificate.hash();
    let second_certificate_id = second_certificate.hash();
    let unsettled_certificate_id = unsettled_certificate.hash();
    let epoch_number = EpochNumber::new(3);
    let certificate_index = CertificateIndex::new(5);
    let second_certificate_index = CertificateIndex::new(6);
    let unsettled_certificate_index = CertificateIndex::new(7);
    let expected_tx_hash = fixture_settlement_tx_hash();

    {
        let db = Arc::new(StateStore::init_db(&state_path).unwrap());
        let store = StateStore::new(db.clone(), BackupClient::noop());
        store
            .write_local_network_state(&network_id, &state, &[leaf_hash])
            .unwrap();
        db.put::<CertificateHeaderColumn>(
            &certificate_id,
            &CertificateHeader {
                network_id,
                height: Height::ZERO,
                epoch_number: Some(epoch_number),
                certificate_index: Some(certificate_index),
                certificate_id,
                prev_local_exit_root: certificate.prev_local_exit_root,
                new_local_exit_root: certificate.new_local_exit_root,
                metadata: certificate.metadata,
                status: CertificateStatus::Settled,
                settlement_tx_hash,
            },
        )
        .unwrap();
        db.put::<CertificateHeaderColumn>(
            &second_certificate_id,
            &CertificateHeader {
                network_id,
                height: second_certificate.height,
                epoch_number: Some(epoch_number),
                certificate_index: Some(second_certificate_index),
                certificate_id: second_certificate_id,
                prev_local_exit_root: second_certificate.prev_local_exit_root,
                new_local_exit_root: second_certificate.new_local_exit_root,
                metadata: second_certificate.metadata,
                status: CertificateStatus::Settled,
                settlement_tx_hash,
            },
        )
        .unwrap();
        db.put::<CertificateHeaderColumn>(
            &unsettled_certificate_id,
            &CertificateHeader {
                network_id,
                height: unsettled_certificate.height,
                epoch_number: Some(epoch_number),
                certificate_index: Some(unsettled_certificate_index),
                certificate_id: unsettled_certificate_id,
                prev_local_exit_root: unsettled_certificate.prev_local_exit_root,
                new_local_exit_root: unsettled_certificate.new_local_exit_root,
                metadata: unsettled_certificate.metadata,
                status: CertificateStatus::Pending,
                settlement_tx_hash: None,
            },
        )
        .unwrap();
        db.put::<CertificatePerNetworkColumn>(
            &certificate_per_network::Key {
                network_id: network_id.to_u32(),
                height: Height::ZERO,
            },
            &certificate_id,
        )
        .unwrap();
        db.put::<CertificatePerNetworkColumn>(
            &certificate_per_network::Key {
                network_id: network_id.to_u32(),
                height: second_certificate.height,
            },
            &second_certificate_id,
        )
        .unwrap();
        db.put::<LatestSettledCertificatePerNetworkColumn>(
            &network_id,
            &SettledCertificate(
                second_certificate_id,
                second_certificate.height,
                epoch_number,
                second_certificate_index,
            ),
        )
        .unwrap();
    }

    {
        let epoch_path = epochs_path.join(epoch_number.to_string());
        let db = DB::open_cf(&epoch_path, EPOCHS_DB).unwrap();
        db.put::<CertificatePerIndexProtoColumn>(&certificate_index, &certificate)
            .unwrap();
        db.put::<CertificatePerIndexProtoColumn>(&second_certificate_index, &second_certificate)
            .unwrap();
        db.put::<CertificatePerIndexProtoColumn>(
            &unsettled_certificate_index,
            &unsettled_certificate,
        )
        .unwrap();
    }

    Fixture {
        _root: root,
        network_id,
        certificate,
        second_certificate,
        unsettled_certificate,
        settlement_tx_hash: expected_tx_hash,
        epoch_number,
        certificate_index,
        second_certificate_index,
        token,
        balance,
    }
}

fn replace_epoch_database_with_empty_directory(fixture: &Fixture) -> PathBuf {
    let epoch_path = fixture
        ._root
        .path
        .join("epochs")
        .join(fixture.epoch_number.to_string());
    std::fs::remove_dir_all(&epoch_path).expect("remove complete epoch database");
    std::fs::create_dir(&epoch_path).expect("replace epoch database with empty directory");
    epoch_path
}

fn seed_debug_certificates(fixture: &Fixture) {
    let debug = DebugStore::new_with_path(&fixture._root.path.join("debug"))
        .expect("create debug database");
    debug
        .add_certificate(&fixture.certificate)
        .expect("store first debug certificate");
    debug
        .add_certificate(&fixture.second_certificate)
        .expect("store second debug certificate");
}

fn move_second_certificate_to_later_epoch(fixture: &Fixture) -> EpochNumber {
    let later_epoch = EpochNumber::new(fixture.epoch_number.as_u64() + 1);
    let state_db = StateStore::init_db(&fixture._root.path.join("state"))
        .expect("open state database for fixture rewrite");
    let second_certificate_id = fixture.second_certificate.hash();
    state_db
        .put::<CertificateHeaderColumn>(
            &second_certificate_id,
            &CertificateHeader {
                network_id: fixture.network_id,
                height: fixture.second_certificate.height,
                epoch_number: Some(later_epoch),
                certificate_index: Some(fixture.second_certificate_index),
                certificate_id: second_certificate_id,
                prev_local_exit_root: fixture.second_certificate.prev_local_exit_root,
                new_local_exit_root: fixture.second_certificate.new_local_exit_root,
                metadata: fixture.second_certificate.metadata,
                status: CertificateStatus::Settled,
                settlement_tx_hash: Some(fixture.settlement_tx_hash),
            },
        )
        .expect("move second certificate header to later epoch");
    state_db
        .put::<LatestSettledCertificatePerNetworkColumn>(
            &fixture.network_id,
            &SettledCertificate(
                second_certificate_id,
                fixture.second_certificate.height,
                later_epoch,
                fixture.second_certificate_index,
            ),
        )
        .expect("move latest-settled pointer to later epoch");
    drop(state_db);

    let later_epoch_path = fixture
        ._root
        .path
        .join("epochs")
        .join(later_epoch.to_string());
    let later_epoch_db =
        DB::open_cf(&later_epoch_path, EPOCHS_DB).expect("create later epoch database for fixture");
    later_epoch_db
        .put::<CertificatePerIndexProtoColumn>(
            &fixture.second_certificate_index,
            &fixture.second_certificate,
        )
        .expect("store second certificate in later epoch");

    later_epoch
}

#[test]
fn reads_validated_exits_balances_and_settlement_hash() {
    let fixture = seed_snapshot(Some(SettlementTxHash::new(Digest::from([0xaa; 32]))));
    let reader = TreeSnapshotReader::open(&fixture._root.path).unwrap();
    let mut exits = Vec::new();
    reader
        .try_visit_network_exits(fixture.network_id, |exit| {
            exits.push(exit);
            Ok::<_, TreeSnapshotError>(())
        })
        .unwrap();
    let balances = reader
        .read_network_balances(fixture.network_id, |_| {})
        .unwrap();

    assert_eq!(exits.len(), 1);
    assert_eq!(exits[0].leaf_index, 0);
    assert_eq!(exits[0].bridge_exit, fixture.certificate.bridge_exits[0]);
    assert_eq!(exits[0].settlement_tx_hash, fixture.settlement_tx_hash);
    assert_eq!(balances.len(), 1);
    assert_eq!(balances[0].token, fixture.token);
    assert_eq!(balances[0].amount, fixture.balance);
}

#[test]
fn visits_settled_certificates_in_height_order_and_excludes_unsettled_data() {
    let fixture = seed_snapshot(Some(SettlementTxHash::new(Digest::from([0xaa; 32]))));
    let reader = TreeSnapshotReader::open(&fixture._root.path).unwrap();
    let mut snapshots = Vec::<SettledCertificateSnapshot>::new();

    reader
        .try_visit_network_certificates(fixture.network_id, |snapshot| {
            snapshots.push(snapshot);
            Ok::<_, TreeSnapshotError>(())
        })
        .unwrap();

    assert_eq!(snapshots.len(), 2);
    assert_eq!(snapshots[0].certificate_id, fixture.certificate.hash());
    assert_eq!(snapshots[0].certificate.height, Height::ZERO);
    assert_eq!(snapshots[0].first_local_exit_index, 0);
    assert_eq!(snapshots[0].certificate.imported_bridge_exits.len(), 1);
    assert!(matches!(
        snapshots[0].certificate.imported_bridge_exits[0].claim_data,
        Claim::Mainnet(_)
    ));
    assert_eq!(
        snapshots[0].certificate.imported_bridge_exits[0]
            .global_index
            .network_id(),
        NetworkId::ETH_L1
    );

    assert_eq!(
        snapshots[1].certificate_id,
        fixture.second_certificate.hash()
    );
    assert_eq!(snapshots[1].certificate.height, Height::new(1));
    assert_eq!(snapshots[1].first_local_exit_index, 1);
    assert_eq!(snapshots[1].certificate.imported_bridge_exits.len(), 1);
    assert!(matches!(
        snapshots[1].certificate.imported_bridge_exits[0].claim_data,
        Claim::Rollup(_)
    ));
    assert_eq!(
        snapshots[1].certificate.imported_bridge_exits[0]
            .global_index
            .network_id(),
        NetworkId::new(4)
    );

    assert!(snapshots
        .iter()
        .all(|snapshot| snapshot.certificate_id != fixture.unsettled_certificate.hash()));
}

#[test]
fn recovers_hash_matching_certificates_from_debug_when_epoch_database_is_unavailable() {
    let fixture = seed_snapshot(Some(fixture_settlement_tx_hash()));
    seed_debug_certificates(&fixture);
    let empty_epoch_path = replace_epoch_database_with_empty_directory(&fixture);
    let reader = TreeSnapshotReader::open(&fixture._root.path).expect("open snapshot reader");
    let mut warnings = Vec::new();
    let mut certificate_ids = Vec::new();

    reader
        .try_visit_network_certificates_with_warnings(
            fixture.network_id,
            |warning| warnings.push(warning),
            |snapshot| {
                certificate_ids.push(snapshot.certificate_id);
                Ok::<_, TreeSnapshotError>(())
            },
        )
        .expect("recover matching certificates from debug database");

    assert_eq!(
        certificate_ids,
        vec![
            fixture.certificate.hash(),
            fixture.second_certificate.hash()
        ]
    );
    assert_eq!(warnings.len(), 2);
    assert!(matches!(
        warnings[0],
        TreeSnapshotWarning::CertificateReadFromDebug {
            network_id,
            height,
            certificate_id,
            epoch_number,
            certificate_index,
            reason: DebugCertificateFallbackReason::EpochDatabaseUnavailable,
        } if network_id == fixture.network_id
            && height == fixture.certificate.height
            && certificate_id == fixture.certificate.hash()
            && epoch_number == fixture.epoch_number
            && certificate_index == fixture.certificate_index
    ));
    assert!(matches!(
        warnings[1],
        TreeSnapshotWarning::CertificateReadFromDebug {
            network_id,
            height,
            certificate_id,
            epoch_number,
            certificate_index,
            reason: DebugCertificateFallbackReason::EpochDatabaseUnavailable,
        } if network_id == fixture.network_id
            && height == fixture.second_certificate.height
            && certificate_id == fixture.second_certificate.hash()
            && epoch_number == fixture.epoch_number
            && certificate_index == fixture.second_certificate_index
    ));
    assert_eq!(
        std::fs::read_dir(empty_epoch_path)
            .expect("inspect unavailable epoch path")
            .count(),
        0,
        "read-only fallback must not create RocksDB files in an empty epoch directory"
    );
}

#[test]
fn missing_debug_certificate_does_not_hide_an_unavailable_epoch_database() {
    let fixture = seed_snapshot(Some(fixture_settlement_tx_hash()));
    let debug = DebugStore::new_with_path(&fixture._root.path.join("debug"))
        .expect("create empty debug database");
    drop(debug);
    let empty_epoch_path = replace_epoch_database_with_empty_directory(&fixture);
    let reader = TreeSnapshotReader::open(&fixture._root.path).expect("open snapshot reader");
    let mut warnings = Vec::new();

    let error = reader
        .try_visit_network_certificates_with_warnings(
            fixture.network_id,
            |warning| warnings.push(warning),
            |_| Ok::<_, TreeSnapshotError>(()),
        )
        .expect_err("missing debug certificate must not be accepted");

    assert!(matches!(
        error,
        TreeSnapshotError::EpochUnavailable { epoch_number, .. }
            if epoch_number == fixture.epoch_number
    ));
    assert!(warnings.is_empty());
    assert_eq!(
        std::fs::read_dir(empty_epoch_path)
            .expect("inspect unavailable epoch path")
            .count(),
        0
    );
}

#[test]
fn mismatched_debug_certificate_is_rejected() {
    let fixture = seed_snapshot(Some(fixture_settlement_tx_hash()));
    let debug =
        DebugStore::init_db(&fixture._root.path.join("debug")).expect("create debug database");
    debug
        .put::<DebugCertificatesProtoColumn>(
            &fixture.certificate.hash(),
            &fixture.second_certificate,
        )
        .expect("store mismatched debug certificate under expected key");
    drop(debug);
    let empty_epoch_path = replace_epoch_database_with_empty_directory(&fixture);
    let reader = TreeSnapshotReader::open(&fixture._root.path).expect("open snapshot reader");
    let mut warnings = Vec::new();

    let error = reader
        .try_visit_network_certificates_with_warnings(
            fixture.network_id,
            |warning| warnings.push(warning),
            |_| Ok::<_, TreeSnapshotError>(()),
        )
        .expect_err("mismatched debug certificate must not be accepted");

    assert!(matches!(error, TreeSnapshotError::Inconsistent(message)
        if message.contains("does not match its state header")));
    assert!(warnings.is_empty(), "only validated recovery is warned");
    assert_eq!(
        std::fs::read_dir(empty_epoch_path)
            .expect("inspect unavailable epoch path")
            .count(),
        0
    );
}

#[test]
fn recovers_missing_certificate_row_from_debug_in_latest_epoch() {
    let fixture = seed_snapshot(Some(fixture_settlement_tx_hash()));
    seed_debug_certificates(&fixture);
    let epoch_path = fixture
        ._root
        .path
        .join("epochs")
        .join(fixture.epoch_number.to_string());
    let epoch_db = DB::open_cf(&epoch_path, EPOCHS_DB).expect("open epoch database");
    epoch_db
        .delete::<CertificatePerIndexProtoColumn>(&fixture.second_certificate_index)
        .expect("remove latest certificate row");
    drop(epoch_db);
    let reader = TreeSnapshotReader::open(&fixture._root.path).expect("open snapshot reader");
    let mut warnings = Vec::new();
    let mut visited = Vec::new();

    reader
        .try_visit_network_certificates_with_warnings(
            fixture.network_id,
            |warning| warnings.push(warning),
            |snapshot| {
                visited.push(snapshot.certificate_id);
                Ok::<_, TreeSnapshotError>(())
            },
        )
        .expect("recover missing latest-epoch row from debug database");

    assert_eq!(
        visited,
        vec![
            fixture.certificate.hash(),
            fixture.second_certificate.hash()
        ]
    );
    assert!(matches!(
        warnings.as_slice(),
        [TreeSnapshotWarning::CertificateReadFromDebug {
            network_id,
            height,
            certificate_id,
            epoch_number,
            certificate_index,
            reason: DebugCertificateFallbackReason::EpochCertificateMissing,
        }] if *network_id == fixture.network_id
            && *height == fixture.second_certificate.height
            && *certificate_id == fixture.second_certificate.hash()
            && *epoch_number == fixture.epoch_number
            && *certificate_index == fixture.second_certificate_index
    ));
}

#[test]
fn missing_earlier_epoch_remains_fatal_even_with_matching_debug_certificate() {
    let fixture = seed_snapshot(Some(fixture_settlement_tx_hash()));
    let later_epoch = move_second_certificate_to_later_epoch(&fixture);
    seed_debug_certificates(&fixture);
    let empty_epoch_path = replace_epoch_database_with_empty_directory(&fixture);
    let reader = TreeSnapshotReader::open(&fixture._root.path).expect("open snapshot reader");
    let mut warnings = Vec::new();

    let error = reader
        .try_visit_network_certificates_with_warnings(
            fixture.network_id,
            |warning| warnings.push(warning),
            |_| Ok::<_, TreeSnapshotError>(()),
        )
        .expect_err("debug fallback must be limited to the latest referenced epoch");

    assert!(matches!(
        error,
        TreeSnapshotError::EpochUnavailable { epoch_number, .. }
            if epoch_number == fixture.epoch_number
    ));
    assert_eq!(later_epoch, EpochNumber::new(4));
    assert!(warnings.is_empty());
    assert_eq!(
        std::fs::read_dir(empty_epoch_path)
            .expect("inspect unavailable earlier epoch path")
            .count(),
        0
    );
}

#[test]
fn missing_latest_epoch_path_stays_absent_when_debug_recovery_succeeds() {
    let fixture = seed_snapshot(Some(fixture_settlement_tx_hash()));
    seed_debug_certificates(&fixture);
    let epoch_path = fixture
        ._root
        .path
        .join("epochs")
        .join(fixture.epoch_number.to_string());
    std::fs::remove_dir_all(&epoch_path).expect("remove latest epoch database");
    let reader = TreeSnapshotReader::open(&fixture._root.path).expect("open snapshot reader");
    let mut warnings = Vec::new();

    reader
        .try_visit_network_certificates_with_warnings(
            fixture.network_id,
            |warning| warnings.push(warning),
            |_| Ok::<_, TreeSnapshotError>(()),
        )
        .expect("recover latest epoch from debug database");

    assert_eq!(warnings.len(), 2);
    assert!(warnings.iter().all(|warning| matches!(
        warning,
        TreeSnapshotWarning::CertificateReadFromDebug {
            reason: DebugCertificateFallbackReason::EpochDatabaseUnavailable,
            ..
        }
    )));
    assert!(
        !epoch_path.exists(),
        "read-only recovery must not recreate a missing epoch path"
    );
}

#[test]
fn invalid_present_epoch_database_does_not_fall_back_to_debug() {
    let fixture = seed_snapshot(Some(fixture_settlement_tx_hash()));
    seed_debug_certificates(&fixture);
    let epoch_path = fixture
        ._root
        .path
        .join("epochs")
        .join(fixture.epoch_number.to_string());
    std::fs::remove_dir_all(&epoch_path).expect("remove valid epoch database");
    std::fs::create_dir(&epoch_path).expect("create invalid epoch directory");
    std::fs::write(epoch_path.join("CURRENT"), b"MANIFEST-000001\n")
        .expect("create regular RocksDB marker without a database");
    let reader = TreeSnapshotReader::open(&fixture._root.path).expect("open snapshot reader");
    let mut warnings = Vec::new();

    let error = reader
        .try_visit_network_certificates_with_warnings(
            fixture.network_id,
            |warning| warnings.push(warning),
            |_| Ok::<_, TreeSnapshotError>(()),
        )
        .expect_err("present but invalid epoch database must remain fatal");

    assert!(matches!(
        error,
        TreeSnapshotError::OpenEpoch { epoch_number, .. }
            if epoch_number == fixture.epoch_number
    ));
    assert!(warnings.is_empty());
    assert_eq!(
        std::fs::read(epoch_path.join("CURRENT")).expect("read unchanged marker"),
        b"MANIFEST-000001\n"
    );
    assert_eq!(
        std::fs::read_dir(epoch_path)
            .expect("inspect invalid epoch directory")
            .count(),
        1,
        "read-only open must not create files beside the invalid marker"
    );
}

#[test]
fn non_file_epoch_marker_is_fatal_without_debug_fallback() {
    let fixture = seed_snapshot(Some(fixture_settlement_tx_hash()));
    seed_debug_certificates(&fixture);
    let epoch_path = fixture
        ._root
        .path
        .join("epochs")
        .join(fixture.epoch_number.to_string());
    std::fs::remove_dir_all(&epoch_path).expect("remove valid epoch database");
    std::fs::create_dir_all(epoch_path.join("CURRENT")).expect("create non-file RocksDB marker");
    let reader = TreeSnapshotReader::open(&fixture._root.path).expect("open snapshot reader");
    let mut warnings = Vec::new();

    let error = reader
        .try_visit_network_certificates_with_warnings(
            fixture.network_id,
            |warning| warnings.push(warning),
            |_| Ok::<_, TreeSnapshotError>(()),
        )
        .expect_err("non-file CURRENT marker must remain fatal");

    assert!(matches!(
        error,
        TreeSnapshotError::InvalidEpochMarker { epoch_number, .. }
            if epoch_number == fixture.epoch_number
    ));
    assert!(warnings.is_empty());
    assert!(epoch_path.join("CURRENT").is_dir());
}

#[test]
fn visits_exits_and_lists_settlement_hashes_without_materializing_history() {
    let fixture = seed_snapshot(Some(SettlementTxHash::new(Digest::from([0xaa; 32]))));
    let reader = TreeSnapshotReader::open(&fixture._root.path).unwrap();

    assert_eq!(reader.network_ids(), vec![fixture.network_id]);
    assert_eq!(
        reader
            .network_settlement_tx_hashes(fixture.network_id)
            .unwrap(),
        vec![fixture.settlement_tx_hash]
    );

    let mut exits = Vec::new();
    reader
        .try_visit_network_exits(fixture.network_id, |exit| {
            exits.push(exit);
            Ok::<_, TreeSnapshotError>(())
        })
        .unwrap();
    assert_eq!(exits.len(), 1);
    assert_eq!(exits[0].leaf_index, 0);
    assert_eq!(exits[0].bridge_exit, fixture.certificate.bridge_exits[0]);
    assert_eq!(exits[0].certificate_id, fixture.certificate.hash());
}

#[test]
fn warns_on_balance_node_hash_mismatch_and_keeps_processing() {
    let fixture = seed_snapshot(Some(SettlementTxHash::new(Digest::from([0xaa; 32]))));

    {
        let store =
            StateStore::new_with_path(&fixture._root.path.join("state"), BackupClient::noop())
                .unwrap();
        let root = store
            .database()
            .get::<BalanceTreePerNetworkColumn>(&SmtKey {
                network_id: fixture.network_id.to_u32(),
                key_type: SmtKeyType::Root,
            })
            .unwrap()
            .unwrap();
        let SmtValue::Node(left, right) = root else {
            panic!("balance-tree root must be a node");
        };

        let (node_hash, node) = [left, right]
            .into_iter()
            .find_map(|hash| {
                let value = store
                    .database()
                    .get::<BalanceTreePerNetworkColumn>(&SmtKey {
                        network_id: fixture.network_id.to_u32(),
                        key_type: SmtKeyType::Node(hash),
                    })
                    .unwrap()?;
                match value {
                    SmtValue::Node(left, right) => Some((hash, (left, right))),
                    SmtValue::Leaf(_) => None,
                }
            })
            .expect("non-empty balance tree has a non-root node");
        assert_ne!(node.0, node.1, "test corruption must change the node hash");
        store
            .database()
            .put::<BalanceTreePerNetworkColumn>(
                &SmtKey {
                    network_id: fixture.network_id.to_u32(),
                    key_type: SmtKeyType::Node(node_hash),
                },
                &SmtValue::Node(node.1, node.0),
            )
            .unwrap();
    }

    let reader = TreeSnapshotReader::open(&fixture._root.path).unwrap();
    let mut warnings = Vec::new();
    let balances = reader
        .read_network_balances(fixture.network_id, |warning| warnings.push(warning))
        .unwrap();

    assert_eq!(balances.len(), 1);
    assert_eq!(balances[0].amount, fixture.balance);
    assert!(matches!(
        warnings.as_slice(),
        [TreeSnapshotWarning::BalanceNodeHashMismatch { network_id, .. }]
            if *network_id == fixture.network_id
    ));
}

#[test]
fn warns_on_balance_leaf_hash_mismatch_and_keeps_processing() {
    let fixture = seed_snapshot(Some(SettlementTxHash::new(Digest::from([0xaa; 32]))));

    {
        let store =
            StateStore::new_with_path(&fixture._root.path.join("state"), BackupClient::noop())
                .unwrap();
        let root = store
            .database()
            .get::<BalanceTreePerNetworkColumn>(&SmtKey {
                network_id: fixture.network_id.to_u32(),
                key_type: SmtKeyType::Root,
            })
            .unwrap()
            .unwrap();
        let SmtValue::Node(left, right) = root else {
            panic!("balance-tree root must be a node");
        };
        let leaf_hash = [left, right]
            .into_iter()
            .find(|hash| {
                matches!(
                    store
                        .database()
                        .get::<BalanceTreePerNetworkColumn>(&SmtKey {
                            network_id: fixture.network_id.to_u32(),
                            key_type: SmtKeyType::Node(*hash),
                        })
                        .unwrap(),
                    Some(SmtValue::Leaf(_))
                )
            })
            .expect("non-empty balance-tree root has an empty leaf child");
        let corrupt_payload = Digest::from([0x99; 32]);
        assert_ne!(corrupt_payload, leaf_hash);
        store
            .database()
            .put::<BalanceTreePerNetworkColumn>(
                &SmtKey {
                    network_id: fixture.network_id.to_u32(),
                    key_type: SmtKeyType::Node(leaf_hash),
                },
                &SmtValue::Leaf(corrupt_payload),
            )
            .unwrap();
    }

    let reader = TreeSnapshotReader::open(&fixture._root.path).unwrap();
    let mut warnings = Vec::new();
    let balances = reader
        .read_network_balances(fixture.network_id, |warning| warnings.push(warning))
        .unwrap();

    assert_eq!(balances.len(), 1);
    assert_eq!(balances[0].token, fixture.token);
    assert_eq!(balances[0].amount, fixture.balance);
    assert!(matches!(
        warnings.as_slice(),
        [TreeSnapshotWarning::BalanceLeafHashMismatch { network_id, .. }]
            if *network_id == fixture.network_id
    ));
}

#[test]
fn rejects_settled_certificate_without_transaction_hash() {
    let fixture = seed_snapshot(None);
    let reader = TreeSnapshotReader::open(&fixture._root.path).unwrap();
    let error = reader
        .network_settlement_tx_hashes(fixture.network_id)
        .unwrap_err();

    assert!(matches!(error, TreeSnapshotError::Inconsistent(message) if
        message.contains("has no settlement transaction hash")));
}

#[test]
fn readonly_state_store_rejects_writes() {
    let fixture = seed_snapshot(Some(SettlementTxHash::new(Digest::from([0xaa; 32]))));
    let store = StateStore::new_readonly_with_path(&fixture._root.path.join("state"))
        .expect("open read-only state store");

    let error = store
        .disable_network(
            &fixture.network_id,
            agglayer_types::network_info::DisabledBy::Admin,
        )
        .expect_err("read-only store must reject writes");

    assert!(matches!(
        error,
        crate::error::Error::DBError(crate::storage::DBError::ReadOnlyMode)
    ));
}

#[test]
fn opening_missing_snapshot_does_not_create_the_database_path() {
    let parent = TempDBDir::new();
    let storage_root = parent.path.join("missing-storage");

    assert!(!storage_root.exists());
    assert!(TreeSnapshotReader::open(&storage_root).is_err());
    assert!(!storage_root.exists());
}

#[test]
fn opening_empty_state_database_leaves_it_empty() {
    let storage_root = TempDBDir::new();
    let state_path = storage_root.path.join("state");
    std::fs::create_dir(&state_path).unwrap();

    assert!(TreeSnapshotReader::open(&storage_root.path).is_err());
    assert_eq!(std::fs::read_dir(state_path).unwrap().count(), 0);
}

#[test]
fn includes_disabled_networks() {
    let fixture = seed_snapshot(Some(SettlementTxHash::new(Digest::from([0xaa; 32]))));
    {
        let store =
            StateStore::new_with_path(&fixture._root.path.join("state"), BackupClient::noop())
                .expect("open writable state store");
        store
            .disable_network(
                &fixture.network_id,
                agglayer_types::network_info::DisabledBy::Admin,
            )
            .expect("disable network");
    }

    let reader = TreeSnapshotReader::open(&fixture._root.path).expect("open snapshot reader");

    assert_eq!(reader.network_ids(), vec![fixture.network_id]);
}

#[test]
fn exports_an_empty_network() {
    let root = TempDBDir::new();
    std::fs::create_dir_all(root.path.join("epochs")).unwrap();
    let network_id = NetworkId::new(12);
    {
        let store =
            StateStore::new_with_path(&root.path.join("state"), BackupClient::noop()).unwrap();
        store
            .write_local_network_state(&network_id, &LocalNetworkStateData::default(), &[])
            .unwrap();
    }

    let reader = TreeSnapshotReader::open(&root.path).unwrap();
    assert_eq!(reader.network_ids(), vec![network_id]);
    let mut exit_count = 0;
    reader
        .try_visit_network_exits(network_id, |_| {
            exit_count += 1;
            Ok::<_, TreeSnapshotError>(())
        })
        .unwrap();
    assert_eq!(exit_count, 0);
    assert!(reader
        .read_network_balances(network_id, |_| {})
        .unwrap()
        .is_empty());
}
