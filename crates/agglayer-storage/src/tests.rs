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

/// Open a state database the way a build without the network info backfill
/// did, so that the next open by the current binary performs the backfill.
pub fn init_state_db_before_network_info_backfill(
    path: &std::path::Path,
) -> Result<crate::storage::DB, crate::storage::DBOpenError> {
    use crate::stores::state::cf_definitions;

    crate::storage::DB::builder(path, cf_definitions::STATE_DB_V0)?
        .ensure_cfs(cf_definitions::STATE_DB_V1_ADDED_CFS)?
        .ensure_cfs(cf_definitions::STATE_DB_V2_ADDED_CFS)?
        .finalize(cf_definitions::STATE_DB)
}
