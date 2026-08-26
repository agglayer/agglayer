use agglayer_types::{
    Certificate, CertificateIndex, CertificateStatus, Digest, EpochNumber, Height,
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
fn settled_claim_round_trips_through_the_settled_cursor(network_id: NetworkId, store: StateStore) {
    let claim = agglayer_types::SettledClaim {
        global_index: Digest([7u8; 32]),
        bridge_exit_hash: Digest([9u8; 32]),
    };

    store
        .set_latest_settled_certificate_for_network(
            &network_id,
            &Height::ZERO,
            &Digest([1u8; 32]).into(),
            &EpochNumber::ZERO,
            &CertificateIndex::ZERO,
            Some(claim.clone()),
        )
        .unwrap();

    assert_eq!(
        store.get_network_info(network_id).unwrap().settled_claim,
        Some(claim)
    );
}

#[test_log::test(rstest)]
fn a_certificate_without_a_claim_leaves_the_stored_one_in_place(
    network_id: NetworkId,
    store: StateStore,
) {
    let claim = agglayer_types::SettledClaim {
        global_index: Digest([7u8; 32]),
        bridge_exit_hash: Digest([9u8; 32]),
    };

    for (height, settled_claim) in [(Height::ZERO, Some(claim.clone())), (Height::new(1), None)] {
        store
            .set_latest_settled_certificate_for_network(
                &network_id,
                &height,
                &Digest([1u8; 32]).into(),
                &EpochNumber::ZERO,
                &CertificateIndex::ZERO,
                settled_claim,
            )
            .unwrap();
    }

    assert_eq!(
        store.get_network_info(network_id).unwrap().settled_claim,
        Some(claim)
    );
}

#[test_log::test(rstest)]
fn a_recovered_claim_is_stored_once_and_never_overwritten(
    network_id: NetworkId,
    store: StateStore,
) {
    let settled = agglayer_types::SettledClaim {
        global_index: Digest([1u8; 32]),
        bridge_exit_hash: Digest([2u8; 32]),
    };
    let scanned = agglayer_types::SettledClaim {
        global_index: Digest([3u8; 32]),
        bridge_exit_hash: Digest([4u8; 32]),
    };

    // Nothing stored yet: the scanned claim fills the gap.
    store
        .set_settled_claim_if_absent(&network_id, &scanned)
        .unwrap();
    assert_eq!(
        store.get_network_info(network_id).unwrap().settled_claim,
        Some(scanned.clone())
    );

    // Settlement always wins over a scan.
    store
        .set_latest_settled_certificate_for_network(
            &network_id,
            &Height::ZERO,
            &Digest([5u8; 32]).into(),
            &EpochNumber::ZERO,
            &CertificateIndex::ZERO,
            Some(settled.clone()),
        )
        .unwrap();

    // A scan racing that settlement must not put the older claim back.
    store
        .set_settled_claim_if_absent(&network_id, &scanned)
        .unwrap();
    assert_eq!(
        store.get_network_info(network_id).unwrap().settled_claim,
        Some(settled)
    );
}
