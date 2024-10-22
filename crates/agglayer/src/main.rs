use clap::Parser;
use cli::Cli;

mod cli;

fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    match cli.cmd {
        cli::Commands::Run { cfg } => agglayer_node::main(cfg)?,
        cli::Commands::Prover { cfg } => agglayer_prover::main(cfg)?,
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
    }

    Ok(())
}
