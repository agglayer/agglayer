use agglayer_types::{
    aggchain_proof::{AggchainData, MultisigPayload},
    Certificate, CertificateIndex, CertificateStatus, Digest, EpochNumber, Height,
    LocalNetworkStateData,
};
use rstest::rstest;

use super::*;
use crate::{
    stores::{
        state::tests::{network_id, store},
        NetworkInfoReader, StateWriter,
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

/// A store that keeps its temporary directory, for tests that write enough to
/// make RocksDB flush: the shared `store` fixture drops the directory on
/// return, and the flush is what notices.
fn owned_store() -> (crate::tests::TempDBDir, StateStore) {
    let tmp = crate::tests::TempDBDir::new();
    let db = std::sync::Arc::new(StateStore::init_db(tmp.path.as_path()).unwrap());

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

#[test_log::test(rstest)]
#[case::preserve_claim(false)]
#[case::replace_claim(true)]
fn settled_info_advances_together(network_id: NetworkId, #[case] replace_claim: bool) {
    let (_tmp, store) = owned_store();
    let mut local_state = LocalNetworkStateData::default();
    let mut expected_claim = None;
    let mut published_settled_id = None;
    let mut published_leaf_count = None;

    for height in [Height::ZERO, Height::new(1)] {
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

        let claim =
            (height == Height::ZERO || replace_claim).then_some(agglayer_types::SettledClaim {
                global_index: leaf,
                bridge_exit_hash: leaf,
            });
        expected_claim = claim.clone().or(expected_claim);
        store
            .set_latest_settled_certificate_for_network(
                &network_id,
                &height,
                &certificate_id,
                &EpochNumber::ZERO,
                &CertificateIndex::ZERO,
                claim,
            )
            .unwrap();

        let info = store.get_network_info(network_id).unwrap();
        assert_eq!(info.settled_certificate_id, Some(certificate_id));
        assert_eq!(info.settled_height, Some(height));
        assert_eq!(info.settled_ler, Some(certificate.new_local_exit_root));
        assert_eq!(info.settled_let_leaf_count, Some(height.as_u64() + 1));
        assert_eq!(info.settled_claim, expected_claim);
        assert_eq!(
            StateStore::network_info_from_snapshot(&snapshot, network_id).unwrap(),
            before,
        );
        published_settled_id = info.settled_certificate_id;
        published_leaf_count = info.settled_let_leaf_count;
    }
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
    let claim = agglayer_types::SettledClaim {
        global_index: Digest([7; 32]),
        bridge_exit_hash: Digest([9; 32]),
    };
    store
        .set_latest_settled_certificate_for_network(
            &network_id,
            &next.height,
            &next_id,
            &epoch,
            &CertificateIndex::new(1),
            Some(claim.clone()),
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
            &Height::ZERO,
            &certificate_id,
            &EpochNumber::ZERO,
            &CertificateIndex::ZERO,
            None,
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
