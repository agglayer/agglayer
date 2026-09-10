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

    #[clap(subcommand)]
    Storage(Storage),
}

#[derive(Subcommand)]
pub(crate) enum Storage {
    /// Export validated local exits, imported exits, and balances from a copied
    /// database.
    ///
    /// The storage root must contain `state/` and `epochs/`. Databases are
    /// opened read-only. The command creates fresh `let/`, `ibe/`, and `lbt/`
    /// directories below the output path and refuses to overwrite any of them.
    ///
    /// LET files contain a JSON array in historical leaf order. Amounts are
    /// decimal strings; `token` is the token information committed in the
    /// leaf, while `amountToken` identifies the token in which `amount` is
    /// denominated. Each row always includes its settlement transaction hash.
    /// LBT files contain a JSON object from `<network>:<address>` to a decimal
    /// amount string. IBE files contain compact imported-exit rows in settled
    /// certificate order, including their global indexes and L1 info-tree
    /// context but omitting Merkle proofs.
    ///
    /// When `--l1-rpc-url` or `AGGLAYER_L1_RPC_URL` is set, the command
    /// verifies each successful settlement receipt against its canonical L1
    /// block and fills `settlementBlockNumber`, `settlementBlockHash`, and
    /// `settledAt` (UTC). Validated lookups are appended to
    /// `.agglayer-settlement-rpc-cache-v1.jsonl` in the output directory and
    /// reused without another receipt or block request after a failed
    /// invocation. The persistent cache is retained after successful RPC runs
    /// and assumes previously validated blocks will not reorg. Without either
    /// RPC setting, those three fields are `null`; an existing cache is ignored
    /// and no cache is created.
    ExportTrees {
        /// Copied Agglayer storage root containing `state/` and `epochs/`.
        #[arg(long, value_hint = ValueHint::DirPath)]
        storage_path: PathBuf,

        /// Destination directory for `let/<network>.json`,
        /// `ibe/<network>.json`, and `lbt/<network>.json`.
        #[arg(long, value_hint = ValueHint::DirPath)]
        output_path: PathBuf,

        /// L1 JSON-RPC endpoint used to enrich local and imported exits with
        /// certificate settlement block timestamps. Only HTTP(S) URLs are
        /// accepted. Prefer `AGGLAYER_L1_RPC_URL` for endpoints
        /// containing credentials; `--l1-rpc-url` takes precedence when
        /// both are provided.
        #[arg(
            long,
            env = "AGGLAYER_L1_RPC_URL",
            hide_env_values = true,
            value_hint = ValueHint::Url
        )]
        l1_rpc_url: Option<String>,
    },

    /// Enrich an `export-trees` result with settlement-day USD prices.
    ///
    /// LET and IBE arrays are streamed into fresh output files and each row
    /// receives a `settlementDayPricing` object. Pricing uses `amountToken` and
    /// the UTC day of `settledAt`: the certificate settlement for LET rows and
    /// the claiming-certificate settlement for IBE rows. Unit prices, token
    /// decimals, normalized amounts, and USD values are written as exact
    /// decimal strings. LBT files are copied unchanged because they have no
    /// valuation timestamp.
    ///
    /// Historical prices come from DefiLlama and remain subject to its terms.
    /// Provider observations are requested near noon UTC, their actual
    /// timestamps and confidence are retained, and missing coverage is written
    /// as a structured unavailable status rather than guessed. Successful and
    /// unavailable lookups are cached in the output directory so interrupted
    /// runs can resume without repeating completed requests. Use
    /// `--refresh-misses` to retry cached provider omissions while continuing
    /// to reuse successful quotes. To refresh a completed no-clobber export,
    /// use its cache as `--seed-price-cache` with a fresh output path.
    ///
    /// The input is never modified. The command creates fresh `let/`, `ibe/`,
    /// and `lbt/` directories plus `pricing-report.json` below the output path
    /// and refuses to overwrite an existing result.
    EnrichTreePrices {
        /// Existing `export-trees` output containing `let/`, `ibe/`, and
        /// `lbt/`.
        #[arg(long, value_hint = ValueHint::DirPath)]
        input_path: PathBuf,

        /// Destination for the enriched copy and persistent pricing cache.
        #[arg(long, value_hint = ValueHint::DirPath)]
        output_path: PathBuf,

        /// Retry cached unavailable price lookups; successful quotes remain
        /// cached.
        #[arg(long)]
        refresh_misses: bool,

        /// Existing cache to copy read-only into a fresh output before
        /// enrichment.
        #[arg(long, value_hint = ValueHint::FilePath)]
        seed_price_cache: Option<PathBuf>,
    },
}

pub(crate) fn parse_l1_rpc_url(value: &str) -> Result<url::Url, String> {
    let url = url::Url::parse(value).map_err(|error| format!("invalid L1 RPC URL: {error}"))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err("L1 RPC URL must use http or https".to_owned());
    }
    Ok(url)
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
    use clap::Parser as _;

    use super::*;

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

    #[test]
    fn parses_tree_export_with_optional_l1_enrichment() {
        let cli = Cli::try_parse_from([
            "agglayer",
            "storage",
            "export-trees",
            "--storage-path",
            "/tmp/storage-copy",
            "--output-path",
            "/tmp/export",
            "--l1-rpc-url",
            "http://127.0.0.1:8545",
        ])
        .expect("tree export command should parse");

        let Commands::Storage(Storage::ExportTrees {
            storage_path,
            output_path,
            l1_rpc_url,
        }) = cli.cmd
        else {
            panic!("expected storage export command");
        };

        assert_eq!(storage_path, PathBuf::from("/tmp/storage-copy"));
        assert_eq!(output_path, PathBuf::from("/tmp/export"));
        assert_eq!(
            l1_rpc_url.expect("RPC URL should be present"),
            "http://127.0.0.1:8545"
        );
    }

    #[test]
    fn rejects_non_http_l1_rpc_urls() {
        let error = parse_l1_rpc_url("file:///tmp/not-an-rpc")
            .expect_err("non-HTTP RPC URL should be rejected");

        assert!(error.contains("must use http or https"));
    }

    #[test]
    fn parses_tree_price_enrichment() {
        let cli = Cli::try_parse_from([
            "agglayer",
            "storage",
            "enrich-tree-prices",
            "--input-path",
            "/tmp/tree-export",
            "--output-path",
            "/tmp/priced-export",
        ])
        .expect("tree price enrichment command should parse");

        let Commands::Storage(Storage::EnrichTreePrices {
            input_path,
            output_path,
            refresh_misses,
            seed_price_cache,
        }) = cli.cmd
        else {
            panic!("expected storage price enrichment command");
        };

        assert_eq!(input_path, PathBuf::from("/tmp/tree-export"));
        assert_eq!(output_path, PathBuf::from("/tmp/priced-export"));
        assert!(!refresh_misses);
        assert!(seed_price_cache.is_none());
    }
}
