//! Agglayer command line interface.
use std::path::PathBuf;

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
}
