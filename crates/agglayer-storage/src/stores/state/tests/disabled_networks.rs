use std::{sync::Arc, time::SystemTime};

use agglayer_types::EpochNumber;

use crate::{
    columns::{
        disabled_networks::DisabledNetworksColumn,
        latest_settled_certificate_per_network::{
            LatestSettledCertificatePerNetworkColumn, SettledCertificate,
        },
    },
    storage::backup::BackupClient,
    stores::{state::StateStore, StateReader},
    tests::TempDBDir,
    types::network_info::v0::{DisabledBy, DisabledNetwork},
};

#[test]
fn can_retrieve_the_list_of_networks() {
    let tmp = TempDBDir::new();
    let db = Arc::new(StateStore::init_db(tmp.path.as_path()).unwrap());

    let store = StateStore::new(db.clone(), BackupClient::noop());
    db.put::<LatestSettledCertificatePerNetworkColumn>(
        &1.into(),
        &SettledCertificate(
            agglayer_types::CertificateId::new([0; 32].into()),
            agglayer_types::Height::ZERO,
            EpochNumber::ZERO,
            agglayer_types::CertificateIndex::ZERO,
        ),
    )
    .expect("Unable to put latest settled certificate into storage");

    assert_eq!(1, store.get_active_networks().unwrap().len());

    db.put::<DisabledNetworksColumn>(
        &1.into(),
        &DisabledNetwork {
            disabled_at: Some(SystemTime::now().into()),
            disabled_by: DisabledBy::Admin as i32,
        },
    )
    .expect("Unable to put latest settled certificate into storage");

    assert_eq!(0, store.get_active_networks().unwrap().len());
}
