use clap::Parser;
use cli::Cli;
use pessimistic_proof::ELF;

mod cli;

fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    match cli.cmd {
        cli::Commands::Run { cfg } => agglayer_node::main(cfg, &version())?,
        cli::Commands::Prover { cfg } => agglayer_prover::main(cfg, &version(), ELF)?,
        cli::Commands::ProverConfig => println!(
            "{}",
            toml::to_string_pretty(&agglayer_prover_config::ProverConfig::default()).unwrap()
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
            let vkey = agglayer_prover::get_vkey(ELF);
            println!("{}", vkey);
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
