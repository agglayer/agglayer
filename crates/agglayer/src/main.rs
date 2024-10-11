use clap::Parser;
use cli::Cli;

mod cli;

fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    match cli.cmd {
        cli::Commands::Run { cfg } => agglayer_node::main(cfg)?,
        cli::Commands::Prover { cfg } => agglayer_prover::main(cfg)?,
        cli::Commands::Config { prover: true, .. } => println!(
            "{}",
            toml::to_string(&agglayer_config::prover::ProverConfig::default()).unwrap()
        ),

        cli::Commands::Config {
            base_dir,
            prover: false,
        } => println!(
            "{}",
            toml::to_string(&agglayer_config::Config::new(&base_dir)).unwrap()
        ),
    }

    Ok(())
}
