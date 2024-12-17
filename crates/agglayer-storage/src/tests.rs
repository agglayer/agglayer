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

        let mut rng = rand::thread_rng();

        path.push(format!(
            "{}/{}_{}",
            folder_name,
            time.as_nanos(),
            rng.gen::<u64>()
        ));

        create_dir_all(path.clone()).expect("Failed to create temp dir");

        Self { path }
    }
}

impl Drop for TempDBDir {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.path).unwrap();
    }
}
