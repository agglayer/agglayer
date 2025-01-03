use std::process::exit;

use agglayer_config::storage::backup::BackupConfig;
use clap::Parser;
use cli::Cli;

mod cli;

fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    match cli.cmd {
        cli::Commands::Run { cfg } => agglayer_node::main(cfg, &version(), None)?,
        cli::Commands::Prover { cfg } => agglayer_prover::main(cfg, &version())?,
        cli::Commands::ProverConfig => println!(
            "{}",
            toml::to_string_pretty(&agglayer_config::prover::ProverConfig::default()).unwrap()
        ),
        cli::Commands::Config { base_dir } => println!(
            "{}",
            toml::to_string_pretty(&agglayer_config::Config::new(&base_dir)).unwrap()
        ),
        cli::Commands::ValidateConfig { path } => {
            match agglayer_config::Config::try_load(path.as_path()) {
                Ok(config) => {
                    println!("{}", toml::to_string_pretty(&config).unwrap());
                }
                Err(error) => eprintln!("{}", error),
            }
        }
        cli::Commands::Vkey => {
            let vkey = agglayer_prover::get_vkey();
            println!("{}", vkey);
        }

        cli::Commands::Backups(cli::Backups::List { cfg }) => {
            let cfg = agglayer_config::Config::try_load(&cfg)?;

            if let BackupConfig::Enabled { path, .. } = cfg.storage.backup {
                let result =
                    agglayer_storage::storage::backup::BackupEngine::list_backups(&path).unwrap();

                println!("{}", serde_json::to_string(&result).unwrap());
            }
        }

        cli::Commands::Backups(cli::Backups::Restore { cfg, db_versions }) => {
            let cfg = agglayer_config::Config::try_load(&cfg)?;

            if let BackupConfig::Enabled { path, .. } = cfg.storage.backup {
                for (db_kind, version) in db_versions {
                    let (db_path, backup_path) = match db_kind {
                        cli::DbKind::State => {
                            (cfg.storage.state_db_path.join("state"), path.join("state"))
                        }
                        cli::DbKind::Pending => (
                            cfg.storage.pending_db_path.join("pending"),
                            path.join("pending"),
                        ),
                        cli::DbKind::Epoch(epoch_number) => (
                            cfg.storage.epochs_db_path.join(format!("{}", epoch_number)),
                            path.join(format!("epochs/{}", epoch_number)),
                        ),
                    };

                    agglayer_storage::storage::backup::BackupEngine::restore_at(
                        &backup_path,
                        &db_path,
                        version,
                    )?;
                }
            } else {
                println!("Backups are not enabled in the configuration file.");
                exit(1);
            }
        }
    }

    Ok(())
}

/// Common version information about the executed agglayer binary.
pub fn version() -> String {
    let pkg_name = env!("CARGO_PKG_NAME");
    let git_describe = env!("VERGEN_GIT_DESCRIBE");
    let timestamp = env!("VERGEN_GIT_COMMIT_TIMESTAMP");
    format!("{pkg_name} ({git_describe}) [git commit timestamp: {timestamp}]")
}
