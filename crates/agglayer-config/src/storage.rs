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
#[derive(Debug, Clone, PartialEq, Eq)]
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
        if !self.metadata_db_path.is_absolute() {
            self.metadata_db_path = base_path.join(&self.metadata_db_path);
        }

        if !self.pending_db_path.is_absolute() {
            self.pending_db_path = base_path.join(&self.pending_db_path);
        }

        if !self.state_db_path.is_absolute() {
            self.state_db_path = base_path.join(&self.state_db_path);
        }

        if !self.epochs_db_path.is_absolute() {
            self.epochs_db_path = base_path.join(&self.epochs_db_path);
        }

        self
    }

    /// Creates a new storage configuration with the default path.
    pub fn new_with_default_path() -> Self {
        let base_path: PathBuf = Path::new("./").join(STORAGE_DIR);

        Self {
            metadata_db_path: default_metadata_path(&base_path),
            pending_db_path: default_pending_path(&base_path),
            state_db_path: default_state_path(&base_path),
            epochs_db_path: default_epochs_path(&base_path),
        }
    }
}

impl From<&Path> for StorageConfig {
    fn from(value: &Path) -> Self {
        let base = value.join(STORAGE_DIR);

        Self {
            metadata_db_path: base.join(METADATA_DB_NAME),
            pending_db_path: base.join(PENDING_DB_NAME),
            state_db_path: base.join(STATE_DB_NAME),
            epochs_db_path: base.join(EPOCHS_DB_PATH),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct StorageConfigHelper {
    db_path: PathBuf,
}

impl<'de> Deserialize<'de> for StorageConfig {
    fn deserialize<D>(deserializer: D) -> Result<StorageConfig, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let helper = StorageConfigHelper::deserialize(deserializer)?;

        Ok(StorageConfig {
            metadata_db_path: default_metadata_path(&helper.db_path),
            pending_db_path: default_pending_path(&helper.db_path),
            state_db_path: default_state_path(&helper.db_path),
            epochs_db_path: default_epochs_path(&helper.db_path),
        })
    }
}

impl Serialize for StorageConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::Error;
        match self.state_db_path.parent() {
            Some(path) => StorageConfigHelper {
                db_path: path.to_path_buf(),
            }
            .serialize(serializer),
            None => Err(Error::custom(
                "Unable to define a base_path for the storage",
            )),
        }
    }
}

/// Returns the default path to the metadata storage directory.
fn default_metadata_path(path: &Path) -> PathBuf {
    path.join("metadata")
}

/// Returns the default path to the pending storage directory.
fn default_pending_path(path: &Path) -> PathBuf {
    path.join("pending")
}

/// Returns the default path to the state storage directory.
fn default_state_path(path: &Path) -> PathBuf {
    path.join("state")
}

/// Returns the default path to the epochs storage directory.
fn default_epochs_path(path: &Path) -> PathBuf {
    path.join("epochs")
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
