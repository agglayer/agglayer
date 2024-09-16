use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

const STORAGE_DIR: &str = "storage";
const METADATA_DB_NAME: &str = "metadata";
const PENDING_DB_NAME: &str = "pending";
const STATE_DB_NAME: &str = "state";
const EPOCHS_DB_PATH: &str = "epochs";

/// Configuration for the storage.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(from = "StorageConfigHelper", into = "StorageConfigHelper")]
pub struct StorageConfig {
    /// Custom metadata storage path or inferred from the db path.
    pub metadata_db_path: PathBuf,
    /// Custom pending storage path or inferred from the db path.
    pub pending_db_path: PathBuf,
    /// Custom state storage path or inferred from the db path.
    pub state_db_path: PathBuf,
    /// Custom epochs storage path or inferred from the db path.
    pub epochs_db_path: PathBuf,
}

impl StorageConfig {
    pub fn path_contextualized(mut self, base_path: &Path) -> Self {
        self.metadata_db_path = base_path.join(&self.metadata_db_path);
        self.pending_db_path = base_path.join(&self.pending_db_path);
        self.state_db_path = base_path.join(&self.state_db_path);
        self.epochs_db_path = base_path.join(&self.epochs_db_path);

        self
    }

    /// Creates a new storage configuration with the default path.
    pub fn new_with_default_path() -> Self {
        Self::new_from_path(&Path::new("./").join(STORAGE_DIR))
    }

    /// Creates a new storage configuration with the given path.
    pub fn new_from_path(value: &Path) -> Self {
        let db_path = value.join(STORAGE_DIR);

        Self {
            metadata_db_path: db_path.join(METADATA_DB_NAME),
            pending_db_path: db_path.join(PENDING_DB_NAME),
            state_db_path: db_path.join(STATE_DB_NAME),
            epochs_db_path: db_path.join(EPOCHS_DB_PATH),
        }
    }
}

/// Helper struct to deserialize the storage configuration.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
struct StorageConfigHelper {
    db_path: PathBuf,
    /// Custom metadata storage path or inferred from the db path.
    pub metadata_db_path: Option<PathBuf>,
    /// Custom pending storage path or inferred from the db path.
    pub pending_db_path: Option<PathBuf>,
    /// Custom state storage path or inferred from the db path.
    pub state_db_path: Option<PathBuf>,
    /// Custom epochs storage path or inferred from the db path.
    pub epochs_db_path: Option<PathBuf>,
}

impl From<StorageConfigHelper> for StorageConfig {
    fn from(value: StorageConfigHelper) -> Self {
        StorageConfig {
            metadata_db_path: value
                .metadata_db_path
                .unwrap_or_else(|| value.db_path.join(METADATA_DB_NAME)),
            pending_db_path: value
                .pending_db_path
                .unwrap_or_else(|| value.db_path.join(PENDING_DB_NAME)),
            state_db_path: value
                .state_db_path
                .unwrap_or_else(|| value.db_path.join(STATE_DB_NAME)),
            epochs_db_path: value
                .epochs_db_path
                .unwrap_or_else(|| value.db_path.join(EPOCHS_DB_PATH)),
        }
    }
}

impl From<StorageConfig> for StorageConfigHelper {
    fn from(value: StorageConfig) -> Self {
        let db_path = value
            .state_db_path
            .parent()
            .expect("Unable to define a base_path for the storage")
            .to_path_buf();

        Self {
            db_path,
            metadata_db_path: None,
            pending_db_path: None,
            state_db_path: None,
            epochs_db_path: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn base_path() {
        let value = toml::toml! {
            db-path = "/tmp/base"
        };

        let cfg = toml::to_string(&value).unwrap();
        let config: StorageConfig = toml::from_str(&cfg).unwrap();

        assert_eq!(config.metadata_db_path, PathBuf::from("/tmp/base/metadata"));
        assert_eq!(config.pending_db_path, PathBuf::from("/tmp/base/pending"));
        assert_eq!(config.state_db_path, PathBuf::from("/tmp/base/state"));
        assert_eq!(config.epochs_db_path, PathBuf::from("/tmp/base/epochs"));
    }
}
