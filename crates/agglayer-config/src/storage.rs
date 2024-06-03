use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;
use serde_with::serde_as;

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct StorageConfig {
    /// Path to the metadata storage directory.
    pub metadata_path: PathBuf,
    /// Path to the pending storage directory.
    pub pending_path: PathBuf,
    /// Path to the state storage directory.
    pub state_path: PathBuf,
    /// Path to the epochs storage directory.
    pub epochs_path: PathBuf,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            metadata_path: default_metadata_path(),
            pending_path: default_pending_path(),
            state_path: default_state_path(),
            epochs_path: default_epochs_path(),
        }
    }
}

/// Returns the default path to the metadata storage directory.
fn default_metadata_path() -> PathBuf {
    match dirs::config_dir() {
        Some(path) => path.join("agglayer/storage/metadata"),
        None => panic!("Impossible to get a home dir when generating the `storage.metadata_path`!"),
    }
}

/// Returns the default path to the pending storage directory.
fn default_pending_path() -> PathBuf {
    match dirs::config_dir() {
        Some(path) => path.join("agglayer/storage/pending"),
        None => panic!("Impossible to get a home dir when generating the `storage.pending_path`!"),
    }
}

/// Returns the default path to the state storage directory.
fn default_state_path() -> PathBuf {
    match dirs::config_dir() {
        Some(path) => path.join("agglayer/storage/state"),
        None => panic!("Impossible to get a home dir when generating the `storage.state_path`!"),
    }
}

/// Returns the default path to the epochs storage directory.
fn default_epochs_path() -> PathBuf {
    match dirs::config_dir() {
        Some(path) => path.join("agglayer/storage/epochs"),
        None => panic!("Impossible to get a home dir when generating the `storage.epochs_path`!"),
    }
}
