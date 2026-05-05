use std::process::exit;

use agglayer_config::storage::backup::BackupConfig;
use clap::Parser;
use cli::Cli;
use eyre::Context as _;
use pessimistic_proof::ELF;
use sp1_sdk::HashableKey as _;

mod cli;

fn main() -> eyre::Result<()> {
    install_default_crypto_provider();

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
                    let vkey_hex = compute_program_vkey(ELF).await;
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
                match agglayer_storage::backup::BackupEngine::list_backups(&path) {
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

                    agglayer_storage::backup::BackupEngine::restore_at(
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

        cli::Commands::MigrateStorage {
            cfg,
            env_label,
            skip_epochs,
            latest_epochs,
            markdown_file,
            html_file,
            no_fail_on_error,
        } => {
            let cfg = agglayer_config::Config::try_load(&cfg)?;

            // Default the env_label to the storage parent directory's
            // basename so concatenated reports across environments stay
            // distinguishable without requiring the operator to set it
            // explicitly.
            let env_label = env_label.unwrap_or_else(|| {
                cfg.storage
                    .pending_db_path
                    .parent()
                    .and_then(|p| p.file_name())
                    .and_then(|s| s.to_str())
                    .unwrap_or("snapshot")
                    .to_string()
            });

            let opts = agglayer_storage::migrate::MigrateOptions {
                state_db_path: Some(cfg.storage.state_db_path.clone()),
                pending_db_path: Some(cfg.storage.pending_db_path.clone()),
                debug_db_path: Some(cfg.storage.debug_db_path.clone()),
                epochs_db_path: Some(cfg.storage.epochs_db_path.clone()),
                env_label,
                skip_epochs,
                latest_epochs,
                markdown_file: markdown_file.clone(),
                html_file: html_file.clone(),
            };

            let outcome = agglayer_storage::migrate::run(opts)
                .map_err(|e| eyre::eyre!("storage migration runner failed: {e}"))?;

            // Default behaviour: print the markdown report to stdout so
            // the operator immediately sees the outcome. If the operator
            // explicitly redirected markdown to a file, we surface the
            // file path on stderr instead (and skip stdout to avoid
            // duplicating the report).
            match markdown_file.as_deref() {
                None => println!("{}", agglayer_storage::migrate::render_markdown(&outcome)),
                Some(path) => eprintln!("Markdown report: {}", path.display()),
            }
            if let Some(path) = html_file.as_deref() {
                eprintln!("HTML report:     {}", path.display());
            }

            if !no_fail_on_error && !outcome.is_success() {
                exit(1);
            }
        }
    }

    Ok(())
}

fn install_default_crypto_provider() {
    // rustls cannot infer a provider when transitive dependencies enable both
    // built-in crypto backends. Install one before any TLS client is built.
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
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

#[cfg(test)]
mod tests {
    #[test]
    fn installs_a_default_rustls_crypto_provider() {
        super::install_default_crypto_provider();

        assert!(rustls::crypto::CryptoProvider::get_default().is_some());
    }
}
