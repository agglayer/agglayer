use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

pub(crate) const STORAGE_DIR: &str = "storage";
const METADATA_DB_NAME: &str = "metadata";
const PENDING_DB_NAME: &str = "pending";
const STATE_DB_NAME: &str = "state";
const EPOCHS_DB_PATH: &str = "epochs";
const DEBUG_DB_PATH: &str = "debug";

/// Configuration for the storage.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(from = "StorageConfigHelper", into = "StorageConfigHelper")]
#[serde(default)]
pub struct StorageConfig {
    /// Custom metadata storage path or inferred from the db path.
    pub metadata_db_path: PathBuf,
    /// Custom pending storage path or inferred from the db path.
    pub pending_db_path: PathBuf,
    /// Custom state storage path or inferred from the db path.
    pub state_db_path: PathBuf,
    /// Custom epochs storage path or inferred from the db path.
    pub epochs_db_path: PathBuf,
    /// Custom debug storage path or inferred from the db path.
    pub debug_db_path: PathBuf,
}

impl Default for StorageConfig {
    fn default() -> Self {
        StorageConfig {
            metadata_db_path: Path::new("./").join(STORAGE_DIR).join(METADATA_DB_NAME),
            pending_db_path: Path::new("./").join(STORAGE_DIR).join(PENDING_DB_NAME),
            state_db_path: Path::new("./").join(STORAGE_DIR).join(STATE_DB_NAME),
            epochs_db_path: Path::new("./").join(STORAGE_DIR).join(EPOCHS_DB_PATH),
            debug_db_path: Path::new("./").join(STORAGE_DIR).join(DEBUG_DB_PATH),
        }
    }
}

impl StorageConfig {
    pub fn path_contextualized(mut self, base_path: &Path) -> Self {
        self.metadata_db_path = normalize_path(&base_path.join(&self.metadata_db_path));
        self.pending_db_path = normalize_path(&base_path.join(&self.pending_db_path));
        self.state_db_path = normalize_path(&base_path.join(&self.state_db_path));
        self.epochs_db_path = normalize_path(&base_path.join(&self.epochs_db_path));
        self.debug_db_path = normalize_path(&base_path.join(&self.debug_db_path));

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
            debug_db_path: db_path.join(DEBUG_DB_PATH),
        }
    }
}

/// Helper struct to deserialize the storage configuration.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
struct StorageConfigHelper {
    #[serde(default = "default_db_path")]
    db_path: PathBuf,
    /// Custom metadata storage path or inferred from the db path.
    pub metadata_db_path: Option<PathBuf>,
    /// Custom pending storage path or inferred from the db path.
    pub pending_db_path: Option<PathBuf>,
    /// Custom state storage path or inferred from the db path.
    pub state_db_path: Option<PathBuf>,
    /// Custom epochs storage path or inferred from the db path.
    pub epochs_db_path: Option<PathBuf>,
    /// Custom debug storage path or inferred from the db path.
    pub debug_db_path: Option<PathBuf>,
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
            debug_db_path: value
                .debug_db_path
                .unwrap_or_else(|| value.db_path.join(DEBUG_DB_PATH)),
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
            debug_db_path: None,
        }
    }
}

fn default_db_path() -> PathBuf {
    PathBuf::new().join(STORAGE_DIR)
}

/// This function is extracted from `cargo`'s internal lib.
///
/// Link: https://github.com/rust-lang/cargo/blob/40ff7be1ad10d1947e22dfeb0f9fa8d2c26025a1/crates/cargo-util/src/paths.rs#L84
///
/// ## Explanation
///
/// Normalize a path, removing things like `.` and `..`.
///
/// CAUTION: This does not resolve symlinks (unlike
/// [`std::fs::canonicalize`]). This may cause incorrect or surprising
/// behavior at times. This should be used carefully. Unfortunately,
/// [`std::fs::canonicalize`] can be hard to use correctly, since it can often
/// fail, or on Windows returns annoying device paths. This is a problem Cargo
/// needs to improve on.
pub(crate) fn normalize_path(path: &Path) -> PathBuf {
    let mut components = path.components().peekable();
    let mut ret = if let Some(c @ Component::Prefix(..)) = components.peek().cloned() {
        components.next();
        PathBuf::from(c.as_os_str())
    } else {
        PathBuf::new()
    };

    for component in components {
        match component {
            Component::Prefix(..) => unreachable!(),
            Component::RootDir => {
                ret.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                ret.pop();
            }
            Component::Normal(c) => {
                ret.push(c);
            }
        }
    }
    ret
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
        assert_eq!(config.debug_db_path, PathBuf::from("/tmp/base/debug"));
    }
}
