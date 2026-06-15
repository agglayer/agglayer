use std::{
    env::temp_dir,
    fs::create_dir_all,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use rand::Rng as _;

pub mod mocks;

pub struct TempDBDir {
    pub path: PathBuf,
}

impl Default for TempDBDir {
    fn default() -> Self {
        Self::new()
    }
}

impl TempDBDir {
    pub fn new() -> Self {
        let mut path = temp_dir().join("agglayer");

        let folder_name = std::thread::current().name().unwrap().replace("::", "_");
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Failed to get time since epoch");

        let mut rng = rand::rng();

        path.push(format!(
            "{}/{}_{}",
            folder_name,
            time.as_nanos(),
            rng.random::<u64>()
        ));

        create_dir_all(path.clone()).expect("Failed to create temp dir");

        Self { path }
    }
}

impl Drop for TempDBDir {
    fn drop(&mut self) {
        _ = std::fs::remove_dir_all(&self.path);
    }
}

/// Asserts that a store's migration gate round-trips: storage created through
/// `open_gate` must re-open through the same gate as `SchemaStatus::Current`.
///
/// This guards the coupling between a store's `DECLARED_MIGRATION_STEPS`
/// constant and the number of migration steps its `init_db` actually records.
/// If they drift, the second open returns `DBOpenError::StorageNeedsMigration`
/// and this assertion fails in CI, instead of the node rejecting storage it
/// just created on its next restart.
///
/// Pass each store's `open_migrated_or_create_db` (e.g.
/// `StateStore::open_migrated_or_create_db`).
pub fn assert_storage_gate_roundtrips<F>(open_gate: F)
where
    F: Fn(&std::path::Path) -> Result<crate::storage::DB, crate::storage::DBOpenError>,
{
    let tmp = TempDBDir::new();
    let path = tmp.path.join("gate-roundtrip");

    // First open creates and migrates fresh storage.
    open_gate(&path).expect("migration gate should create fresh storage on first open");

    // Storage created through the gate must re-open through the same gate.
    // A failure here means DECLARED_MIGRATION_STEPS is out of sync with the
    // number of steps init_db records.
    open_gate(&path).expect(
        "storage created through the migration gate must re-open as Current; \
         DECLARED_MIGRATION_STEPS is out of sync with init_db's recorded step count",
    );
}
