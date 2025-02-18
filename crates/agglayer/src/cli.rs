//! Agglayer command line interface.
use std::path::{Path, PathBuf};

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
        /// The path to the agglayer dir.
        #[arg(
            long,
            short,
            value_hint = ValueHint::DirPath,
        )]
        path: PathBuf,
    },

    ProverConfig,

    Prover {
        /// The path to the configuration file.
        #[arg(long, short, value_hint = ValueHint::FilePath, default_value = "agglayer-prover.toml", env = "PROVER_CONFIG_PATH")]
        cfg: PathBuf,
    },

    Vkey,

    #[clap(subcommand)]
    Backup(Backup),
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
            Self::State => (cfg.storage.state_db_path.join("state"), path.join("state")),
            Self::Pending => (
                cfg.storage.pending_db_path.join("pending"),
                path.join("pending"),
            ),
            Self::Epoch(epoch_number) => (
                cfg.storage.epochs_db_path.join(format!("{}", epoch_number)),
                path.join(format!("epochs/{}", epoch_number)),
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
                    return Err(format!("Unexpected DbKind: {}", s));
                };

                let epoch = epoch
                    .parse::<u64>()
                    .map_err(|e| format!("Invalid epoch: {}", e))?;

                Ok(DbKind::Epoch(epoch))
            }
        }
    }
}

fn parse_db_kind_version(s: &str) -> Result<(DbKind, u32), String> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 {
        return Err(format!(
            "Invalid format for argument '{}'. Expected 'name:version'",
            s
        ));
    }

    let db_kind = parts[0].parse::<DbKind>()?;
    let version = parts[1]
        .parse::<u32>()
        .map_err(|e| format!("Invalid version '{}': {}", parts[1], e))?;

    Ok((db_kind, version))
}
