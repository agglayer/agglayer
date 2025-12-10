use std::process::exit;

use agglayer_config::storage::backup::BackupConfig;
use clap::Parser;
use cli::Cli;
use eyre::Context as _;
use pessimistic_proof::ELF;
use sp1_sdk::HashableKey as _;

mod cli;

fn main() -> eyre::Result<()> {
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    match cli.cmd {
        cli::Commands::Run { cfg } => agglayer_node::main(cfg, &version(), None)?,
        cli::Commands::Config { base_dir } => println!(
            "{}",
            toml::to_string_pretty(&agglayer_config::Config::new(&base_dir))
                .context("Failed to serialize Config to TOML")?
        ),
        cli::Commands::ValidateConfig { path } => {
            match agglayer_config::Config::try_load(path.as_path()) {
                Ok(config) => {
                    println!(
                        "{}",
                        toml::to_string_pretty(&config)
                            .context("Failed to serialize ValidateConfig to TOML")?
                    );
                }
                Err(error) => eprintln!("{error}"),
            }
        }
        cli::Commands::Vkey => {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()?
                .block_on(async move {
                    let vkey_hex = compute_program_vkey(ELF)
                        .await
                        .context("Failed to compute program vkey");
                    match vkey_hex {
                        Ok(vkey_hex) => println!("{vkey_hex}"),
                        Err(error) => eprintln!("{error:?}"),
                    }
                });
        }

        cli::Commands::VkeySelector => {
            let vkey_selector_hex =
                hex::encode(pessimistic_proof::core::PESSIMISTIC_PROOF_PROGRAM_SELECTOR);
            println!("0x{vkey_selector_hex}");
        }

        cli::Commands::Backup(cli::Backup::List { config_path: cfg }) => {
            let cfg = agglayer_config::Config::try_load(&cfg)?;

            if let BackupConfig::Enabled { path, .. } = cfg.storage.backup {
                match agglayer_storage::storage::backup::BackupEngine::list_backups(&path) {
                    Ok(result) => println!("{}", serde_json::to_string(&result).unwrap()),
                    Err(error) => eprintln!("{error}"),
                }
            }
        }

        cli::Commands::Backup(cli::Backup::Restore {
            config_path: cfg,
            db_versions,
        }) => {
            let cfg = agglayer_config::Config::try_load(&cfg)?;

            if let BackupConfig::Enabled { ref path, .. } = cfg.storage.backup {
                for (db_kind, version) in db_versions {
                    let (db_path, backup_path) = db_kind.create_paths(&cfg, path);

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

pub async fn compute_program_vkey(program: &'static [u8]) -> eyre::Result<String> {
    let vkey = prover_executor::Executor::compute_program_vkey(program)
        .await
        .context("Failed to compute program vkey")?;
    Ok(vkey.bytes32())
}
