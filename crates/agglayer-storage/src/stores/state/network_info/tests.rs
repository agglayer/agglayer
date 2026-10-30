use agglayer_types::{
    aggchain_proof::{AggchainData, MultisigPayload},
    Certificate, CertificateIndex, CertificateStatus, Digest, EpochNumber, Height,
    LocalNetworkStateData, SettledClaim as Claim,
};
use rstest::rstest;

use super::*;
use crate::{
    stores::{
        state::tests::{network_id, store},
        NetworkInfoReader, NetworkSettlement, StateReader, StateWriter,
    },
    types::network_info::{
        v0::{
            LatestPendingCertificateHeight, LatestPendingCertificateId,
            LatestPendingCertificateInfo, LatestProvenCertificateInfo, NetworkInfoValue,
            NetworkType, SettledCertificate, SettledCertificateId,
        },
        Key,
    },
};

/// The settlement of `certificate` in epoch 0, as the network task publishes
/// it.
fn settlement(
    certificate: &Certificate,
    new_pp_root: Digest,
    settled_let_leaf_count: u64,
    settled_claim: Option<Claim>,
) -> NetworkSettlement {
    NetworkSettlement {
        certificate_id: certificate.hash(),
        height: certificate.height,
        epoch_number: EpochNumber::ZERO,
        certificate_index: CertificateIndex::ZERO,
        new_pp_root,
        new_local_exit_root: certificate.new_local_exit_root,
        settled_let_leaf_count,
        network_type: agglayer_types::NetworkType::from(&certificate.aggchain_data),
        settled_claim,
    }
}

/// The stored settled row, with every field set.
fn stored_settled_row(store: &StateStore, network_id: NetworkId) -> SettledCertificate {
    let stored = store
        .db
        .get::<NetworkInfoColumn>(&Key {
            network_id: network_id.to_u32(),
            kind: network_info_value::ValueDiscriminants::SettledCertificate,
        })
        .unwrap();
    let Some(NetworkInfoValue {
        value: Some(network_info_value::Value::SettledCertificate(row)),
    }) = stored
    else {
        panic!("no settled row: {stored:?}");
    };
    assert!(row.certificate_id.is_some(), "{row:?}");
    assert!(row.ler.is_some(), "{row:?}");
    assert!(row.pp_root.is_some(), "{row:?}");
    assert!(row.let_leaf_count.is_some(), "{row:?}");
    row
}

/// A store that keeps its temporary directory, for tests that write enough to
/// make RocksDB flush: the shared `store` fixture drops the directory on
/// return, and the flush is what notices.
fn owned_store() -> (crate::tests::TempDBDir, StateStore) {
    let tmp = crate::tests::TempDBDir::new();
    let db = std::sync::Arc::new(StateStore::init_db(tmp.path.as_path(), None).unwrap());

    (
        tmp,
        StateStore::new(db, crate::backup::BackupClient::noop()),
    )
}

#[test_log::test(rstest)]
fn fetching_an_unexisting_network(network_id: NetworkId, store: StateStore) {
    let network_info = store.get_network_info(network_id).unwrap();

    assert_eq!(network_info.latest_pending_certificate_id, None);
    assert_eq!(network_info.latest_pending_height, None);
    assert_eq!(
        store.get_latest_pending_certificate_id(network_id).unwrap(),
        None
    );
    assert_eq!(
        store.get_latest_proven_certificate_id(network_id).unwrap(),
        None
    );
}

#[test_log::test(rstest)]
fn fetching_an_existing_network(network_id: NetworkId, store: StateStore) {
    store
        .db
        .put::<NetworkInfoColumn>(
            &Key {
                network_id: network_id.to_u32(),
                kind: network_info_value::ValueDiscriminants::NetworkType,
            },
            &NetworkInfoValue {
                value: Some(network_info_value::Value::NetworkType(
                    NetworkType::MultisigOnly as i32,
                )),
            },
        )
        .unwrap();
    let network_info = store.get_network_info(network_id).unwrap();

    assert_eq!(
        network_info.network_type,
        agglayer_types::NetworkType::MultisigOnly
    );
}

#[test_log::test(rstest)]
fn pending_and_proven_ids_round_trip(network_id: NetworkId, store: StateStore) {
    let pending_id = CertificateId::new(Digest::from([1; 32]));
    let proven_id = CertificateId::new(Digest::from([2; 32]));
    let pending_height = Height::new(42);

    store
        .db
        .put::<NetworkInfoColumn>(
            &Key {
                network_id: network_id.to_u32(),
                kind: network_info_value::ValueDiscriminants::LatestPendingCertificateInfo,
            },
            &NetworkInfoValue {
                value: Some(network_info_value::Value::LatestPendingCertificateInfo(
                    LatestPendingCertificateInfo {
                        height: Some(LatestPendingCertificateHeight {
                            height: pending_height.as_u64(),
                        }),
                        id: Some(LatestPendingCertificateId {
                            id: pending_id.as_digest().as_slice().to_vec().into(),
                        }),
                    },
                )),
            },
        )
        .unwrap();
    store
        .db
        .put::<NetworkInfoColumn>(
            &Key {
                network_id: network_id.to_u32(),
                kind: network_info_value::ValueDiscriminants::LatestProvenCertificateInfo,
            },
            &NetworkInfoValue {
                value: Some(network_info_value::Value::LatestProvenCertificateInfo(
                    LatestProvenCertificateInfo {
                        id: proven_id.as_digest().as_slice().to_vec().into(),
                    },
                )),
            },
        )
        .unwrap();

    let network_info = store.get_network_info(network_id).unwrap();
    assert_eq!(network_info.latest_pending_certificate_id, Some(pending_id));
    assert_eq!(network_info.latest_pending_height, Some(pending_height));
    assert_eq!(
        store.get_latest_pending_certificate_id(network_id).unwrap(),
        Some(pending_id)
    );
    assert_eq!(
        store.get_latest_proven_certificate_id(network_id).unwrap(),
        Some(proven_id)
    );
}

#[test_log::test(rstest)]
fn settled_pointer_allows_absent_optional_aggregates(network_id: NetworkId, store: StateStore) {
    let certificate = Certificate::new_for_test(network_id, Height::new(7));
    let certificate_id = certificate.hash();
    let epoch = EpochNumber::new(3);
    let index = CertificateIndex::new(1);
    store
        .insert_certificate_header(&certificate, CertificateStatus::Settled)
        .unwrap();
    store
        .assign_certificate_to_epoch(&certificate_id, &epoch, &index)
        .unwrap();
    store
        .db
        .put::<NetworkInfoColumn>(
            &Key {
                network_id: network_id.to_u32(),
                kind: network_info_value::ValueDiscriminants::SettledCertificate,
            },
            &NetworkInfoValue {
                value: Some(network_info_value::Value::SettledCertificate(
                    SettledCertificate {
                        certificate_id: Some(SettledCertificateId {
                            id: certificate_id.as_digest().as_slice().to_vec().into(),
                        }),
                        pp_root: None,
                        let_leaf_count: None,
                        ler: None,
                    },
                )),
            },
        )
        .unwrap();

    let network_info = store.get_network_info(network_id).unwrap();
    assert_eq!(network_info.settled_certificate_id, Some(certificate_id));
    assert_eq!(network_info.settled_height, Some(certificate.height));
    assert_eq!(
        network_info.settled_ler,
        Some(certificate.new_local_exit_root)
    );
    assert_eq!(network_info.settled_pp_root, None);
    assert_eq!(network_info.settled_let_leaf_count, None);
    assert_eq!(
        network_info.latest_epoch_with_settlement,
        Some(epoch.as_u64())
    );
}

/// Two settlements of one network, the second of another type, covering the
/// four claim rules: a first settlement writes its claim or no claim row, a
/// later one replaces the cached claim or preserves it.
#[test_log::test(rstest)]
#[case::first_claim_preserved(true, false)]
#[case::first_claim_replaced(true, true)]
#[case::first_claim_written_later(false, true)]
#[case::never_claimed(false, false)]
fn settled_info_advances_together(
    network_id: NetworkId,
    #[case] first_claim: bool,
    #[case] second_claim: bool,
) {
    let (_tmp, store) = owned_store();
    let mut local_state = LocalNetworkStateData::default();
    let mut expected_claim = None;
    let mut published_settled_id = None;
    let mut published_leaf_count = None;

    for (height, has_claim) in [(Height::ZERO, first_claim), (Height::new(1), second_claim)] {
        let snapshot = store.db.snapshot();
        let before = StateStore::network_info_from_snapshot(&snapshot, network_id).unwrap();
        let mut certificate = Certificate::new_for_test(network_id, height);
        if height != Height::ZERO {
            certificate.aggchain_data = AggchainData::MultisigOnly {
                multisig: MultisigPayload(Vec::new()),
            };
        }
        let certificate_id = certificate.hash();
        let leaf = Digest([height.as_u64() as u8; 32]);
        let pp_root = Digest([height.as_u64() as u8 + 0x10; 32]);
        local_state.exit_tree.add_leaf(leaf).unwrap();
        store
            .write_local_network_state(&network_id, &local_state, &[leaf])
            .unwrap();
        store
            .insert_certificate_header(&certificate, CertificateStatus::Proven)
            .unwrap();
        store
            .assign_certificate_to_epoch(
                &certificate_id,
                &EpochNumber::ZERO,
                &CertificateIndex::ZERO,
            )
            .unwrap();

        // None of that moves the published settlement: the leaf count and the
        // settled pointer advance only when the settlement publishes them.
        let unpublished = store.get_network_info(network_id).unwrap();
        assert_eq!(unpublished.settled_certificate_id, published_settled_id);
        assert_eq!(unpublished.settled_let_leaf_count, published_leaf_count);

        let claim = has_claim.then_some(Claim {
            global_index: leaf,
            bridge_exit_hash: leaf,
        });
        expected_claim = claim.clone().or(expected_claim);
        let settlement = settlement(
            &certificate,
            pp_root,
            u64::from(local_state.exit_tree.leaf_count()),
            claim,
        );
        store
            .set_latest_settled_certificate_for_network(&network_id, &settlement)
            .unwrap();

        let info = store.get_network_info(network_id).unwrap();
        assert_eq!(info.settled_certificate_id, Some(certificate_id));
        assert_eq!(info.settled_height, Some(height));
        assert_eq!(info.settled_ler, Some(certificate.new_local_exit_root));
        assert_eq!(info.settled_pp_root, Some(pp_root));
        assert_eq!(info.settled_let_leaf_count, Some(height.as_u64() + 1));
        assert_eq!(info.settled_claim, expected_claim);
        // The type is the settled certificate's, even when it changes.
        assert_eq!(info.network_type, settlement.network_type);
        assert_eq!(
            store
                .get_latest_settled_certificate_per_network(&network_id)
                .unwrap(),
            Some((
                network_id,
                SettledCursor(
                    certificate_id,
                    height,
                    EpochNumber::ZERO,
                    CertificateIndex::ZERO
                )
            ))
        );
        // The row stores the LER too, so it never depends on the header.
        let row = stored_settled_row(&store, network_id);
        assert_eq!(
            row.ler.unwrap().root.as_ref(),
            Digest::from(certificate.new_local_exit_root).as_slice()
        );
        assert_eq!(
            StateStore::network_info_from_snapshot(&snapshot, network_id).unwrap(),
            before,
        );
        published_settled_id = info.settled_certificate_id;
        published_leaf_count = info.settled_let_leaf_count;
    }
    assert_eq!(
        store.get_network_info(network_id).unwrap().network_type,
        agglayer_types::NetworkType::MultisigOnly
    );
}

/// A settlement after the backfill replaces the backfilled root and leaf
/// count with its own, from the settlement input rather than the live tree,
/// and leaves the pending and proven rows alone.
#[test_log::test(rstest)]
fn settlement_replaces_backfilled_aggregates_and_keeps_other_rows(
    network_id: NetworkId,
    store: StateStore,
) {
    let backfilled = Certificate::new_for_test(network_id, Height::ZERO);
    let backfilled_claim = Claim {
        global_index: Digest([4; 32]),
        bridge_exit_hash: Digest([5; 32]),
    };
    let pending_row = StateStore::network_info_row(
        network_id,
        network_info_value::Value::LatestPendingCertificateInfo(LatestPendingCertificateInfo {
            height: Some(LatestPendingCertificateHeight { height: 2 }),
            id: Some(LatestPendingCertificateId {
                id: vec![6; 32].into(),
            }),
        }),
    );
    let proven_row = StateStore::network_info_row(
        network_id,
        network_info_value::Value::LatestProvenCertificateInfo(LatestProvenCertificateInfo {
            id: vec![7; 32].into(),
        }),
    );
    let backfilled_rows = [
        StateStore::network_info_row(
            network_id,
            network_info_value::Value::NetworkType(NetworkType::Generic as i32),
        ),
        StateStore::network_info_row(
            network_id,
            StateStore::settled_certificate_value(
                backfilled.hash(),
                backfilled.new_local_exit_root,
                Some(Digest([0xbb; 32])),
                Some(7),
            ),
        ),
        StateStore::network_info_row(
            network_id,
            StateStore::settled_claim_value(&backfilled_claim),
        ),
    ];
    for (key, value) in backfilled_rows.iter().chain([&pending_row, &proven_row]) {
        store.db.put::<NetworkInfoColumn>(key, value).unwrap();
    }
    store
        .insert_certificate_header(&backfilled, CertificateStatus::Settled)
        .unwrap();

    // The live tree holds one leaf; the settlement reports another count.
    let mut local_state = LocalNetworkStateData::default();
    let leaf = Digest([1; 32]);
    local_state.exit_tree.add_leaf(leaf).unwrap();
    store
        .write_local_network_state(&network_id, &local_state, &[leaf])
        .unwrap();
    let next = Certificate::new_for_test(network_id, Height::new(1));
    store
        .insert_certificate_header(&next, CertificateStatus::Settled)
        .unwrap();
    // Every test certificate shares the empty-tree root, so a root the
    // header does not hold shows the row stores the input's.
    let input_ler = Digest([0xdd; 32]);
    store
        .set_latest_settled_certificate_for_network(
            &network_id,
            &NetworkSettlement {
                new_local_exit_root: input_ler.into(),
                ..settlement(&next, Digest([0xcc; 32]), 5, None)
            },
        )
        .unwrap();

    let info = store.get_network_info(network_id).unwrap();
    assert_eq!(info.settled_certificate_id, Some(next.hash()));
    assert_eq!(info.settled_pp_root, Some(Digest([0xcc; 32])));
    assert_eq!(info.settled_let_leaf_count, Some(5));
    assert_eq!(info.network_type, agglayer_types::NetworkType::Ecdsa);
    assert_eq!(info.settled_claim, Some(backfilled_claim));
    let row = stored_settled_row(&store, network_id);
    assert_eq!(row.ler.unwrap().root.as_ref(), input_ler.as_slice());
    for (key, value) in [pending_row, proven_row] {
        assert_eq!(
            store.db.get::<NetworkInfoColumn>(&key).unwrap(),
            Some(value)
        );
    }
}

/// A settlement batch that fails to write advances neither the legacy cursor
/// nor any cached row. Every write fails on a read-only database, so this
/// covers the failure path; all-or-nothing on a live database rests on the
/// writer staging everything into one `write_batch`.
#[test]
fn failed_settlement_batch_leaves_both_representations_unchanged() {
    use std::sync::Arc;

    use crate::{
        backup::BackupClient,
        storage::{DBError, DB},
    };
    let (tmp, store) = owned_store();
    let network_id = NetworkId::new(1);
    let first = Certificate::new_for_test(network_id, Height::ZERO);
    let first_claim = Claim {
        global_index: Digest([4; 32]),
        bridge_exit_hash: Digest([5; 32]),
    };
    store
        .insert_certificate_header(&first, CertificateStatus::Settled)
        .unwrap();
    store
        .set_latest_settled_certificate_for_network(
            &network_id,
            &settlement(&first, Digest([0xaa; 32]), 1, Some(first_claim)),
        )
        .unwrap();
    let next = Certificate::new_for_test(network_id, Height::new(1));
    store
        .insert_certificate_header(&next, CertificateStatus::Settled)
        .unwrap();
    let cursor = store
        .get_latest_settled_certificate_per_network(&network_id)
        .unwrap();
    let info = store.get_network_info(network_id).unwrap();
    drop(store);

    let store = StateStore::new(
        Arc::new(
            DB::open_cf_readonly(&tmp.path, crate::stores::state::cf_definitions::STATE_DB)
                .unwrap(),
        ),
        BackupClient::noop(),
    );
    let mut next_settlement = settlement(&next, Digest([0xcc; 32]), 2, None);
    next_settlement.network_type = agglayer_types::NetworkType::Generic;
    assert!(matches!(
        store.set_latest_settled_certificate_for_network(&network_id, &next_settlement),
        Err(Error::DBError(DBError::ReadOnlyMode))
    ));
    assert_eq!(
        store
            .get_latest_settled_certificate_per_network(&network_id)
            .unwrap(),
        cursor
    );
    assert_eq!(store.get_network_info(network_id).unwrap(), info);
}

#[test_log::test(rstest)]
#[case::legacy_cursor(false)]
#[case::cached_pointer_without_leaf_count(true)]
fn legacy_settlement_does_not_borrow_the_next_settlements_leaf_count(
    network_id: NetworkId,
    #[case] cached_pointer: bool,
) {
    let (_tmp, store) = owned_store();
    let mut local_state = LocalNetworkStateData::default();
    let first_leaf = Digest([1; 32]);
    local_state.exit_tree.add_leaf(first_leaf).unwrap();
    store
        .write_local_network_state(&network_id, &local_state, &[first_leaf])
        .unwrap();
    let certificate = Certificate::new_for_test(network_id, Height::ZERO);
    let certificate_id = certificate.hash();
    let epoch = EpochNumber::new(3);
    store
        .insert_certificate_header(&certificate, CertificateStatus::Proven)
        .unwrap();
    store
        .assign_certificate_to_epoch(&certificate_id, &epoch, &CertificateIndex::ZERO)
        .unwrap();
    store
        .db
        .put::<LatestSettledCertificatePerNetworkColumn>(
            &network_id,
            &SettledCursor(certificate_id, Height::ZERO, epoch, CertificateIndex::ZERO),
        )
        .unwrap();
    if cached_pointer {
        let (key, value) = StateStore::network_info_row(
            network_id,
            network_info_value::Value::SettledCertificate(SettledCertificate {
                certificate_id: Some(SettledCertificateId {
                    id: certificate_id.as_digest().as_slice().to_vec().into(),
                }),
                ..Default::default()
            }),
        );
        store.db.put::<NetworkInfoColumn>(&key, &value).unwrap();
    }

    // The next settlement writes the LET before publishing its cursor. A
    // database snapshot in this interval must not associate that count with
    // the old settled certificate, even when both reads use the snapshot.
    let second_leaf = Digest([2; 32]);
    local_state.exit_tree.add_leaf(second_leaf).unwrap();
    store
        .write_local_network_state(&network_id, &local_state, &[second_leaf])
        .unwrap();
    let snapshot = store.db.snapshot();
    let before = store.get_network_info(network_id).unwrap();
    assert_eq!(before.settled_certificate_id, Some(certificate_id));
    assert_eq!(before.settled_height, Some(Height::ZERO));
    assert_eq!(before.settled_ler, Some(certificate.new_local_exit_root));
    assert_eq!(before.latest_epoch_with_settlement, Some(epoch.as_u64()));
    assert_eq!(before.settled_let_leaf_count, None);

    let next = Certificate::new_for_test(network_id, Height::new(1));
    let next_id = next.hash();
    store
        .insert_certificate_header(&next, CertificateStatus::Proven)
        .unwrap();
    store
        .assign_certificate_to_epoch(&next_id, &epoch, &CertificateIndex::new(1))
        .unwrap();
    let claim = Claim {
        global_index: Digest([7; 32]),
        bridge_exit_hash: Digest([9; 32]),
    };
    store
        .set_latest_settled_certificate_for_network(
            &network_id,
            &NetworkSettlement {
                epoch_number: epoch,
                certificate_index: CertificateIndex::new(1),
                ..settlement(&next, Digest([0xcc; 32]), 2, Some(claim.clone()))
            },
        )
        .unwrap();

    assert_eq!(
        StateStore::network_info_from_snapshot(&snapshot, network_id).unwrap(),
        before,
    );
    let after = store.get_network_info(network_id).unwrap();
    assert_eq!(after.settled_certificate_id, Some(next_id));
    assert_eq!(after.settled_let_leaf_count, Some(2));
    assert_eq!(after.settled_claim, Some(claim));
}

#[test_log::test(rstest)]
fn settled_header_is_read_from_the_same_snapshot(network_id: NetworkId, store: StateStore) {
    let certificate = Certificate::new_for_test(network_id, Height::ZERO);
    let certificate_id = certificate.hash();
    store
        .insert_certificate_header(&certificate, CertificateStatus::Proven)
        .unwrap();
    store
        .assign_certificate_to_epoch(&certificate_id, &EpochNumber::ZERO, &CertificateIndex::ZERO)
        .unwrap();
    store
        .set_latest_settled_certificate_for_network(
            &network_id,
            &settlement(&certificate, Digest::ZERO, 0, None),
        )
        .unwrap();
    let snapshot = store.db.snapshot();

    // Rewriting a header after the snapshot must not change the joined fields.
    let mut header = store
        .db
        .get::<CertificateHeaderColumn>(&certificate_id)
        .unwrap()
        .unwrap();
    header.epoch_number = Some(EpochNumber::new(1));
    store
        .db
        .put::<CertificateHeaderColumn>(&certificate_id, &header)
        .unwrap();

    assert_eq!(
        StateStore::network_info_from_snapshot(&snapshot, network_id)
            .unwrap()
            .latest_epoch_with_settlement,
        Some(0),
    );
    assert_eq!(
        store
            .get_network_info(network_id)
            .unwrap()
            .latest_epoch_with_settlement,
        Some(1)
    );
}

#[test]
fn pending_and_proven_writers_preserve_other_rows_and_header_metadata() {
    use crate::stores::StateReader;
    let (_tmp, store) = owned_store();
    let certificate = Certificate::new_for_test(NetworkId::new(1), Height::new(7));
    let id = certificate.hash();
    store
        .insert_certificate_header(&certificate, CertificateStatus::Pending)
        .unwrap();
    let type_row = StateStore::network_info_row(
        certificate.network_id,
        network_info_value::Value::NetworkType(NetworkType::Ecdsa as i32),
    );
    store
        .db
        .put::<NetworkInfoColumn>(&type_row.0, &type_row.1)
        .unwrap();
    let settled_row = StateStore::network_info_row(
        certificate.network_id,
        StateStore::settled_certificate_value(id, certificate.new_local_exit_root, None, Some(3)),
    );
    let claim_row = StateStore::network_info_row(
        certificate.network_id,
        StateStore::settled_claim_value(&agglayer_types::SettledClaim {
            global_index: Digest([4; 32]),
            bridge_exit_hash: Digest([5; 32]),
        }),
    );
    for (key, value) in [&settled_row, &claim_row] {
        store.db.put::<NetworkInfoColumn>(key, value).unwrap();
    }
    let error = CertificateStatus::error(agglayer_types::CertificateStatusError::InternalError(
        "retry".into(),
    ));
    store
        .publish_pending_certificate(&certificate, &error)
        .unwrap();
    let header = store.get_certificate_header(&id).unwrap().unwrap();
    assert_eq!(header.status, error);
    assert_eq!(header.new_local_exit_root, certificate.new_local_exit_root);
    assert_eq!(
        store
            .get_latest_pending_certificate_id(certificate.network_id)
            .unwrap(),
        Some(id)
    );
    assert_eq!(
        store
            .get_latest_pending_height(certificate.network_id)
            .unwrap(),
        Some(certificate.height)
    );
    assert_eq!(
        store
            .get_latest_proven_certificate_id(certificate.network_id)
            .unwrap(),
        None
    );
    store
        .update_certificate_header_status(&id, &CertificateStatus::Proven)
        .unwrap();
    assert_eq!(
        store
            .get_latest_proven_certificate_id(certificate.network_id)
            .unwrap(),
        None
    );
    store
        .set_latest_proven_certificate_pointer(certificate.network_id, id)
        .unwrap();
    assert_eq!(
        store
            .get_latest_proven_certificate_id(certificate.network_id)
            .unwrap(),
        Some(id)
    );
    assert_eq!(
        store.db.get::<NetworkInfoColumn>(&type_row.0).unwrap(),
        Some(type_row.1)
    );
    for (key, value) in [settled_row, claim_row] {
        assert_eq!(
            store.db.get::<NetworkInfoColumn>(&key).unwrap(),
            Some(value)
        );
    }
}

#[test]
fn failed_pending_batch_preserves_initial_header_and_previous_pointer() {
    use std::sync::Arc;

    use crate::{
        backup::BackupClient,
        storage::{DBError, DB},
        stores::StateReader,
    };
    let (tmp, store) = owned_store();
    let first = Certificate::new_for_test(NetworkId::new(1), Height::ZERO);
    let next = Certificate::new_for_test(NetworkId::new(1), Height::new(1));
    store
        .insert_certificate_header(&first, CertificateStatus::Pending)
        .unwrap();
    store
        .set_latest_pending_certificate_pointer(first.network_id, first.height, first.hash())
        .unwrap();
    store
        .insert_certificate_header(&next, CertificateStatus::Pending)
        .unwrap();
    let header = store.get_certificate_header(&next.hash()).unwrap().unwrap();
    drop(store);
    let store = StateStore::new(
        Arc::new(
            DB::open_cf_readonly(&tmp.path, crate::stores::state::cf_definitions::STATE_DB)
                .unwrap(),
        ),
        BackupClient::noop(),
    );
    assert!(matches!(
        store.publish_pending_certificate(&next, &CertificateStatus::Proven),
        Err(Error::DBError(DBError::ReadOnlyMode))
    ));
    assert_eq!(
        store.get_certificate_header(&next.hash()).unwrap().unwrap(),
        header
    );
    assert_eq!(
        store
            .get_latest_pending_certificate_id(first.network_id)
            .unwrap(),
        Some(first.hash())
    );
}
