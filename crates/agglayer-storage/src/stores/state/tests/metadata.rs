use std::sync::Arc;

use agglayer_types::EpochNumber;

use crate::{
    columns::metadata::MetadataColumn,
    storage::backup::BackupClient,
    stores::{state::StateStore, MetadataReader as _, MetadataWriter as _},
    tests::TempDBDir,
    types::{MetadataKey, MetadataValue},
};

#[test]
fn can_retrieve_the_last_settled_epoch() {
    let tmp = TempDBDir::new();
    let db = Arc::new(StateStore::init_db(tmp.path.as_path()).unwrap());

    let store = StateStore::new(db.clone(), BackupClient::noop());
    assert!(store.get_latest_settled_epoch().unwrap().is_none());

    db.put::<MetadataColumn>(
        &MetadataKey::LatestSettledEpoch,
        &MetadataValue::LatestSettledEpoch(EpochNumber::new(1)),
    )
    .expect("Unable to put latest settled epoch into storage");

    assert!(
        matches!(store.get_latest_settled_epoch().unwrap(), Some(e1) if e1 == EpochNumber::new(1))
    );
}

#[test]
fn can_set_the_latest_epoch_settled() {
    let tmp = TempDBDir::new();
    let db = Arc::new(StateStore::init_db(tmp.path.as_path()).unwrap());

    let store = StateStore::new(db.clone(), BackupClient::noop());
    assert!(store.get_latest_settled_epoch().unwrap().is_none());

    store
        .set_latest_settled_epoch(EpochNumber::new(2))
        .expect("Unable to set latest settled epoch");

    assert!(matches!(
        db.get::<MetadataColumn>(&MetadataKey::LatestSettledEpoch),
        Ok(Some(MetadataValue::LatestSettledEpoch(e))) if e.as_u64() == 2
    ));
}
