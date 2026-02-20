use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Configuration for taking backups on node startup.
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum StartupBackupMode {
    /// Do not take backups on startup.
    #[serde(alias = "yolo")]
    Never,

    /// Take backup on startup if database migration is required,
    /// just before the migration is executed.
    #[default]
    IfMigrating,

    /// Always take backup on node startup.
    Always,
}

/// Configuration for Storage backups.
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum BackupConfig {
    /// Backups are disabled.
    #[default]
    Disabled,

    /// Backups are enabled.
    #[serde(untagged, rename_all = "kebab-case")]
    Enabled {
        /// Path to the directory where backups are stored.
        path: PathBuf,

        /// Maximum number of backups to keep for the state storage.
        #[serde(default = "default_max_backup_number")]
        state_max_backup_count: usize,

        /// Maximum number of backups to keep for the pending storage.
        #[serde(default = "default_max_backup_number")]
        pending_max_backup_count: usize,

        /// Whether to take backup on node startup.
        #[serde(default)]
        on_startup: StartupBackupMode,
    },
}

impl BackupConfig {
    /// Default maximum number of backups to keep.
    const DEFAULT_MAX_BACKUP_NUMBER: usize = 100;

    pub fn with_path<P: Into<PathBuf>>(path: P) -> Self {
        BackupConfig::Enabled {
            path: path.into(),
            state_max_backup_count: default_max_backup_number(),
            pending_max_backup_count: default_max_backup_number(),
            on_startup: StartupBackupMode::default(),
        }
    }

    pub fn is_disabled(&self) -> bool {
        *self == BackupConfig::Disabled
    }
}

const fn default_max_backup_number() -> usize {
    BackupConfig::DEFAULT_MAX_BACKUP_NUMBER
}
