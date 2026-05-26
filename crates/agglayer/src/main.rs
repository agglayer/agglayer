use std::process::exit;

use agglayer_config::storage::backup::BackupConfig;
use clap::Parser;
use cli::Cli;
use eyre::Context as _;
use pessimistic_proof::ELF;
use sp1_sdk::HashableKey as _;

mod cli;
mod doctor_report;
mod migrate_report;

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

            let env_label = env_label.unwrap_or_else(default_env_label);

            let opts = agglayer_storage::migrate::MigrateOptions {
                state_db_path: Some(cfg.storage.state_db_path.clone()),
                pending_db_path: Some(cfg.storage.pending_db_path.clone()),
                debug_db_path: Some(cfg.storage.debug_db_path.clone()),
                epochs_db_path: Some(cfg.storage.epochs_db_path.clone()),
                env_label,
                skip_epochs,
                latest_epochs,
            };

            let outcome = agglayer_storage::migrate::run(opts);

            // Default behaviour: print the markdown report to stdout so
            // the operator immediately sees the outcome. If the operator
            // explicitly redirected markdown to a file, we surface the
            // file path on stderr instead (and skip stdout to avoid
            // duplicating the report).
            match markdown_file.as_deref() {
                None => println!("{}", migrate_report::render_markdown(&outcome)),
                Some(path) => {
                    migrate_report::write_to_file(path, &migrate_report::render_markdown(&outcome))
                        .with_context(|| {
                            format!("failed to write markdown report to {}", path.display())
                        })?;
                    eprintln!("Markdown report: {}", path.display());
                }
            }
            if let Some(path) = html_file.as_deref() {
                migrate_report::write_to_file(path, &migrate_report::render_html(&outcome))
                    .with_context(|| {
                        format!("failed to write HTML report to {}", path.display())
                    })?;
                eprintln!("HTML report:     {}", path.display());
            }

            if !no_fail_on_error && !outcome.is_success() {
                exit(1);
            }
        }

        cli::Commands::StorageDoctor(cli::StorageDoctor::List {
            cfg,
            env_label,
            markdown_file,
            html_file,
        }) => {
            let cfg = agglayer_config::Config::try_load(&cfg)?;
            let env_label = env_label.unwrap_or_else(default_env_label);

            let mut rows = Vec::new();
            rows.extend(scan_or_warn(
                agglayer_storage::diagnostics::scan_unparsable_pending_rows(
                    &cfg.storage.pending_db_path,
                ),
                "pending",
            ));
            rows.extend(scan_or_warn(
                agglayer_storage::diagnostics::scan_unparsable_debug_rows(
                    &cfg.storage.debug_db_path,
                ),
                "debug",
            ));
            rows.extend(scan_or_warn(
                agglayer_storage::diagnostics::scan_unparsable_epoch_rows(
                    &cfg.storage.epochs_db_path,
                ),
                "epoch",
            ));

            let generated_at = std::time::SystemTime::now();
            match markdown_file.as_deref() {
                None => println!(
                    "{}",
                    doctor_report::render_markdown(&env_label, generated_at, &rows)
                ),
                Some(path) => {
                    doctor_report::write_to_file(
                        path,
                        &doctor_report::render_markdown(&env_label, generated_at, &rows),
                    )
                    .with_context(|| {
                        format!("failed to write markdown report to {}", path.display())
                    })?;
                    eprintln!("Markdown report: {}", path.display());
                }
            }
            if let Some(path) = html_file.as_deref() {
                doctor_report::write_to_file(
                    path,
                    &doctor_report::render_html(&env_label, generated_at, &rows),
                )
                .with_context(|| format!("failed to write HTML report to {}", path.display()))?;
                eprintln!("HTML report:     {}", path.display());
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

/// Downgrade a diagnostics scan failure to a warn-log on stderr + an
/// empty result, matching the migrate runner's policy: a scan error
/// never blocks the report.
fn scan_or_warn(
    result: Result<
        Vec<agglayer_storage::diagnostics::UnparsableRow>,
        agglayer_storage::diagnostics::ScanError,
    >,
    store: &'static str,
) -> Vec<agglayer_storage::diagnostics::UnparsableRow> {
    match result {
        Ok(rows) => rows,
        Err(error) => {
            eprintln!("warning: storage-doctor scan for `{store}` failed: {error}");
            Vec::new()
        }
    }
}

fn default_env_label() -> String {
    "local".to_string()
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
    use super::*;

    #[test]
    fn installs_a_default_rustls_crypto_provider() {
        install_default_crypto_provider();

        assert!(rustls::crypto::CryptoProvider::get_default().is_some());
    }

    #[test]
    fn default_env_label_is_local() {
        assert_eq!(default_env_label(), "local");
    }
}
