use std::{sync::Arc, time::SystemTime};

use agglayer_types::Digest;

use crate::{
    backup::BackupClient,
    columns::{disabled_networks::DisabledNetworksColumn, network_info::NetworkInfoColumn},
    stores::{state::StateStore, StateReader},
    tests::TempDBDir,
    types::network_info::v0::{DisabledBy, DisabledNetwork},
};

#[test]
fn can_retrieve_the_list_of_networks() {
    let tmp = TempDBDir::new();
    let db = Arc::new(StateStore::init_db(tmp.path.as_path(), None).unwrap());

    let store = StateStore::new(db.clone(), BackupClient::noop());
    let (key, value) = StateStore::network_info_row(
        1.into(),
        StateStore::settled_certificate_value(
            agglayer_types::CertificateId::new([0; 32].into()),
            Digest::ZERO.into(),
            None,
            None,
        ),
    );
    db.put::<NetworkInfoColumn>(&key, &value)
        .expect("Unable to put the settled row into storage");

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
