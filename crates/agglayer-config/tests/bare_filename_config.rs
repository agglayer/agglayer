use std::path::{Path, PathBuf};

use agglayer_config::Config;
use pretty_assertions::assert_eq;

struct CurrentDirGuard(PathBuf);

impl Drop for CurrentDirGuard {
    fn drop(&mut self) {
        std::env::set_current_dir(&self.0).unwrap();
    }
}

#[test]
fn bare_filename_config_path_uses_current_directory() {
    let original_dir = std::env::current_dir().unwrap();
    let tests_dir = original_dir.join("tests");
    std::env::set_current_dir(&tests_dir).unwrap();
    let _current_dir_guard = CurrentDirGuard(original_dir);

    let config = Config::try_load(Path::new("bare_filename_config.toml")).unwrap();

    assert_eq!(
        config.storage.state_db_path,
        tests_dir.canonicalize().unwrap().join("storage/state")
    );
}
