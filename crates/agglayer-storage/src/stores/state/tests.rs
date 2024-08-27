use std::sync::Arc;

use crate::{
    columns::latest_certificate_per_network::{
        LatestSettledCertificatePerNetworkColumn, ProvenCertificate,
    },
    storage::{state_db_cf_definitions, DB},
    stores::{state::StateStore, StateReader as _},
    tests::TempDBDir,
    types::NetworkId,
};

#[test]
fn can_retrieve_list_of_network() {
    let tmp = TempDBDir::new();
    let db = Arc::new(DB::open_cf(tmp.path.as_path(), state_db_cf_definitions()).unwrap());
    let store = StateStore::new(db.clone());
    assert!(store.get_active_networks().unwrap().is_empty());

    db.put::<LatestSettledCertificatePerNetworkColumn>(
        &NetworkId(1),
        &ProvenCertificate([0; 32], 0, 0),
    )
    .expect("Unable to put certificate into storage");
    assert!(store.get_active_networks().unwrap().len() == 1);
}
