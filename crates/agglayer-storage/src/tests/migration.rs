//! Database migration schema tests.
//!
//! These tests just check the database schema assumed by the code corresponds
//! to the schema running in production. The databases were obtained using the
//! `sample-storage-for-test` tool in the examples folder.

use std::path::Path;

use crate::{
    stores::{
        debug::DebugStore, pending::PendingStore, per_epoch::PerEpochStore, state::StateStore,
    },
    tests::{extract_tarball, TempDBDir},
};

fn extract_tmp_db(tarball_name: impl AsRef<Path>) -> TempDBDir {
    let dir = TempDBDir::new();
    let tarball_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/tests/db")
        .join(tarball_name);
    extract_tarball(tarball_path.as_path(), &dir.path).unwrap();
    dir
}

#[rstest::rstest]
fn open_state(#[values("bali", "cardona", "mainnet")] env: &str) {
    let tarball_name = format!("schema-{env}-state.tar.gz");
    let db_dir = extract_tmp_db(tarball_name);
    StateStore::init_db(&db_dir.path).unwrap();
}

#[rstest::rstest]
fn open_pending(#[values("bali", "cardona", "mainnet")] env: &str) {
    let tarball_name = format!("schema-{env}-pending.tar.gz");
    let db_dir = extract_tmp_db(tarball_name);
    PendingStore::init_db(&db_dir.path).unwrap();
}

#[rstest::rstest]
fn open_epoch(#[values("bali", "cardona", "mainnet")] env: &str) {
    let tarball_name = format!("schema-{env}-epoch.tar.gz");
    let db_dir = extract_tmp_db(tarball_name);
    PerEpochStore::<PendingStore, StateStore>::init_db(&db_dir.path).unwrap();
}

#[rstest::rstest]
fn open_debug(#[values("mainnet")] env: &str) {
    let tarball_name = format!("schema-{env}-debug.tar.gz");
    let db_dir = extract_tmp_db(tarball_name);
    DebugStore::init_db(&db_dir.path).unwrap();
}
