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
                    if let Some(outbound) = &config.outbound {
                        eprintln!("warning: {}", outbound.ignored_config_warning());
                    }
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
            } else {
                println!("Backups are not enabled in the configuration file.");
                exit(1);
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
    }

    Ok(())
}

fn install_default_crypto_provider() {
    // rustls cannot infer a provider when transitive dependencies enable both
    // built-in crypto backends. Install one before any TLS client is built.
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
}

/// Common version information about the executed agglayer binary.
///
/// The git describe and commit timestamp are resolved at compile time. They
/// default to the values derived from the local `.git` repository by `vergen`,
/// but can be overridden via the `AGGLAYER_BUILD_DESCRIBE` and
/// `AGGLAYER_BUILD_TIMESTAMP` environment variables. The override path exists
/// for builds without a `.git` directory (e.g. the Docker image build), where
/// `vergen` would otherwise emit a `VERGEN_IDEMPOTENT_OUTPUT` placeholder.
pub fn version() -> String {
    let pkg_name = env!("CARGO_PKG_NAME");
    let git_describe = resolve_build_value(
        option_env!("AGGLAYER_BUILD_DESCRIBE"),
        env!("VERGEN_GIT_DESCRIBE"),
    );
    let timestamp = resolve_build_value(
        option_env!("AGGLAYER_BUILD_TIMESTAMP"),
        env!("VERGEN_GIT_COMMIT_TIMESTAMP"),
    );
    format!("{pkg_name} ({git_describe}) [git commit timestamp: {timestamp}]")
}

/// Prefer an explicit build-time override, falling back to the git-derived
/// value when the override is absent or empty.
fn resolve_build_value<'a>(override_value: Option<&'a str>, fallback: &'a str) -> &'a str {
    match override_value {
        Some(value) if !value.is_empty() => value,
        _ => fallback,
    }
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

    #[test]
    fn build_value_prefers_override_when_present() {
        assert_eq!(
            super::resolve_build_value(Some("v1.2.3-7-gabc1234"), "fallback"),
            "v1.2.3-7-gabc1234"
        );
    }

    #[test]
    fn build_value_falls_back_when_override_absent() {
        assert_eq!(super::resolve_build_value(None, "fallback"), "fallback");
    }

    #[test]
    fn build_value_falls_back_when_override_is_empty() {
        assert_eq!(super::resolve_build_value(Some(""), "fallback"), "fallback");
    }
}
