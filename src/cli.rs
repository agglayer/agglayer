//! Agglayer command line interface.
use std::path::PathBuf;

use clap::{Parser, ValueHint};

/// Agglayer command line interface.
#[derive(Parser)]
pub(crate) struct Cli {
    /// The path to the configuration file.
    #[arg(long, short, value_hint = ValueHint::FilePath, default_value = "agglayer.toml", env = "CONFIG_PATH")]
    pub(crate) config_path: PathBuf,
}
