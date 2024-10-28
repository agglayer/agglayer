use std::sync::Arc;

use crate::{
    columns::metadata::MetadataColumn,
    storage::{state_db_cf_definitions, DB},
    stores::{state::StateStore, MetadataReader as _, MetadataWriter as _},
    tests::TempDBDir,
    types::{MetadataKey, MetadataValue},
};

#[test]
fn can_retrieve_the_last_settled_epoch() {
    let tmp = TempDBDir::new();
    let db = Arc::new(DB::open_cf(tmp.path.as_path(), state_db_cf_definitions()).unwrap());

    let store = StateStore::new(db.clone());
    assert!(store.get_latest_settled_epoch().unwrap().is_none());

    db.put::<MetadataColumn>(
        &MetadataKey::LatestSettledEpoch,
        &MetadataValue::LatestSettledEpoch(1),
    )
    .expect("Unable to put latest settled epoch into storage");

    assert!(matches!(store.get_latest_settled_epoch().unwrap(), Some(1)));
}

#[test]
fn can_set_the_latest_epoch_settled() {
    let tmp = TempDBDir::new();
    let db = Arc::new(DB::open_cf(tmp.path.as_path(), state_db_cf_definitions()).unwrap());

    let store = StateStore::new(db.clone());
    assert!(store.get_latest_settled_epoch().unwrap().is_none());

    store
        .set_latest_settled_epoch(2)
        .expect("Unable to set latest settled epoch");

    assert!(matches!(
        db.get::<MetadataColumn>(&MetadataKey::LatestSettledEpoch),
        Ok(Some(MetadataValue::LatestSettledEpoch(2)))
    ));
}
