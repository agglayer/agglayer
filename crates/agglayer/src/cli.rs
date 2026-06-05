//! Agglayer command line interface.
use std::{
    num::NonZeroU64,
    path::{Path, PathBuf},
};

use clap::{Parser, Subcommand, ValueHint};

use crate::version;

/// Agglayer command line interface.
#[derive(Parser)]
#[command(version = version())]
#[command(propagate_version = true)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) cmd: Commands,
}

#[derive(Subcommand)]
pub(crate) enum Commands {
    Run {
        /// The path to the configuration file.
        #[arg(long, short, value_hint = ValueHint::FilePath, default_value = "agglayer.toml", env = "CONFIG_PATH")]
        cfg: PathBuf,
    },

    Config {
        /// The path to the agglayer dir.
        #[arg(
            long,
            short,
            value_hint = ValueHint::DirPath,
            env = "CONFIG_PATH"
        )]
        base_dir: PathBuf,
    },

    ValidateConfig {
        /// The path to the configuration file.
        #[arg(
            long,
            short,
            value_hint = ValueHint::FilePath,
        )]
        path: PathBuf,
    },

    Vkey,
    VkeySelector,

    #[clap(subcommand)]
    Backup(Backup),

    /// Run the storage migration in place against the configured or selected
    /// data directory. Brings every store (state, pending, debug, every epoch)
    /// up to the current schema. Running this command may modify selected
    /// stores.
    MigrateStorage {
        /// Path to the agglayer configuration file. Migration paths are
        /// derived from `[storage]`.
        #[arg(long, short, value_hint = ValueHint::FilePath, default_value = "agglayer.toml", env = "AGGLAYER_CONFIG_PATH")]
        cfg: PathBuf,

        /// Override the configured storage directory. The command derives
        /// `state`, `pending`, `debug`, and `epochs` paths directly under
        /// this root. Running `migrate-storage` may modify those stores.
        #[arg(long, value_hint = ValueHint::DirPath, env = "AGGLAYER_MIGRATION_STORAGE_PATH")]
        storage_path: Option<PathBuf>,

        /// Operator-supplied environment label (`mainnet`, `testnet`, …)
        /// used in the markdown report's heading and filename. Defaults
        /// to the configured data directory's basename, or `local` when
        /// the basename cannot be derived.
        #[arg(long, env = "AGGLAYER_MIGRATION_ENV_LABEL")]
        env_label: Option<String>,

        /// Skip the epoch sweep entirely; only state, pending, and debug
        /// run. Useful when iterating on the upgrade procedure or when
        /// epoch migration was already done in a previous run.
        #[arg(long)]
        skip_epochs: bool,

        /// Cap the epoch sweep to the N most-recent epochs (highest
        /// numeric names first). Useful for spot checks: the active
        /// data lives at the latest epochs while the lowest-numbered
        /// ones are typically empty.
        #[arg(long)]
        latest_epochs: Option<NonZeroU64>,

        /// Write the markdown report to this file path. By default the
        /// markdown is printed to stdout; pass this flag to redirect it
        /// to a file instead. The flag is independent of `--html-file`,
        /// so you can request both, neither, or either.
        #[arg(long, value_hint = ValueHint::FilePath)]
        markdown_file: Option<PathBuf>,

        /// Write the HTML report to this file path. When unset, no HTML
        /// is produced. The HTML is self-contained (no external
        /// resources) and openable in any browser.
        #[arg(long, value_hint = ValueHint::FilePath)]
        html_file: Option<PathBuf>,

        /// Suppress the non-zero exit on fatal store outcomes. By
        /// default the command exits non-zero when any store fails so
        /// CI/orchestration sees the failure; pass this flag to keep
        /// the run "advisory" (markdown report still flags failures).
        #[arg(long)]
        no_fail_on_error: bool,
    },
}

#[derive(Subcommand)]
pub(crate) enum Backup {
    /// List all backups.
    List {
        #[arg(long, short, value_hint = ValueHint::FilePath, default_value = "agglayer.toml", env = "CONFIG_PATH")]
        config_path: PathBuf,
    },

    /// Restore from a backup.
    Restore {
        #[arg(long, short, value_hint = ValueHint::FilePath, default_value = "agglayer.toml", env = "CONFIG_PATH")]
        config_path: PathBuf,
        #[arg(value_parser = parse_db_kind_version)]
        db_versions: Vec<(DbKind, u32)>,
    },
}

#[derive(Debug, Clone)]
pub(crate) enum DbKind {
    State,
    Pending,
    Epoch(u64),
}

impl DbKind {
    pub(crate) fn create_paths(
        &self,
        cfg: &agglayer_config::Config,
        path: &Path,
    ) -> (PathBuf, PathBuf) {
        match self {
            Self::State => (cfg.storage.state_db_path.clone(), path.join("state")),
            Self::Pending => (cfg.storage.pending_db_path.clone(), path.join("pending")),
            Self::Epoch(epoch_number) => (
                cfg.storage.epochs_db_path.join(format!("{epoch_number}")),
                path.join(format!("epochs/{epoch_number}")),
            ),
        }
    }
}

impl std::str::FromStr for DbKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str().trim() {
            "state" => Ok(DbKind::State),
            "pending" => Ok(DbKind::Pending),
            s => {
                let Some(epoch) = s.strip_prefix("epoch_") else {
                    return Err(format!("Unexpected DbKind: {s}"));
                };

                let epoch = epoch
                    .parse::<u64>()
                    .map_err(|e| format!("Invalid epoch: {e}"))?;

                Ok(DbKind::Epoch(epoch))
            }
        }
    }
}

fn parse_db_kind_version(s: &str) -> Result<(DbKind, u32), String> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 {
        return Err(format!(
            "Invalid format for argument '{s}'. Expected 'name:version'"
        ));
    }

    let db_kind = parts[0].parse::<DbKind>()?;
    let version = parts[1]
        .parse::<u32>()
        .map_err(|e| format!("Invalid version '{}': {}", parts[1], e))?;

    Ok((db_kind, version))
}

#[cfg(test)]
mod tests {
    use agglayer_config::Config;
    use clap::Parser;

    use super::*;

    #[test]
    fn migrate_storage_rejects_zero_latest_epochs() {
        let err = match Cli::try_parse_from(["agglayer", "migrate-storage", "--latest-epochs", "0"])
        {
            Ok(_) => panic!("zero latest-epochs should be rejected at the CLI boundary"),
            Err(err) => err,
        };

        assert!(
            err.to_string().contains("latest-epochs"),
            "error should mention the rejected flag, got {err}"
        );
    }

    #[test]
    fn migrate_storage_accepts_explicit_storage_path() {
        let cli = Cli::try_parse_from([
            "agglayer",
            "migrate-storage",
            "--storage-path",
            "/var/lib/agglayer/storage",
        ])
        .unwrap();

        match cli.cmd {
            Commands::MigrateStorage { storage_path, .. } => {
                assert_eq!(
                    storage_path.unwrap(),
                    PathBuf::from("/var/lib/agglayer/storage")
                );
            }
            _ => panic!("expected migrate-storage command"),
        }
    }

    #[test]
    fn migrate_storage_rejects_dry_run_flag() {
        let err = match Cli::try_parse_from(["agglayer", "migrate-storage", "--dry-run"]) {
            Ok(_) => panic!("dry-run should be rejected at the CLI boundary"),
            Err(err) => err,
        };

        assert_eq!(err.kind(), clap::error::ErrorKind::UnknownArgument);
    }

    #[test]
    fn rejects_storage_doctor_subcommand() {
        let err = match Cli::try_parse_from(["agglayer", "storage-doctor", "list"]) {
            Ok(_) => panic!("storage-doctor should not be exposed in this CLI"),
            Err(err) => err,
        };

        assert_eq!(err.kind(), clap::error::ErrorKind::InvalidSubcommand);
    }

    #[test]
    fn testing_path_state() {
        let path_normal = PathBuf::from("/tmp/normal");
        let config = Config::new(&path_normal);

        let path_normal = path_normal.join("storage");
        let kind = DbKind::State;
        let path_backup = PathBuf::from("/tmp/storage/backup");
        let (destination, backup) = kind.create_paths(&config, &path_backup);

        assert_eq!(destination, path_normal.join("state"));
        assert_eq!(backup, path_backup.join("state"));
    }

    #[test]
    fn testing_path_pending() {
        let path_normal = PathBuf::from("/tmp/normal");
        let config = Config::new(&path_normal);

        let path_normal = path_normal.join("storage");
        let kind = DbKind::Pending;
        let path_backup = PathBuf::from("/tmp/storage/backup");
        let (destination, backup) = kind.create_paths(&config, &path_backup);

        assert_eq!(destination, path_normal.join("pending"));
        assert_eq!(backup, path_backup.join("pending"));
    }

    #[test]
    fn testing_path_epochs() {
        let path_normal = PathBuf::from("/tmp/normal");
        let config = Config::new(&path_normal);

        let path_normal = path_normal.join("storage");
        let kind = DbKind::Epoch(10);
        let path_backup = PathBuf::from("/tmp/storage/backup");
        let (destination, backup) = kind.create_paths(&config, &path_backup);

        assert_eq!(destination, path_normal.join("epochs/10"));
        assert_eq!(backup, path_backup.join("epochs/10"));
    }
}
