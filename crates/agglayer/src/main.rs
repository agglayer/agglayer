use clap::Parser;
use cli::Cli;

mod cli;

fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    match cli.cmd {
        cli::Commands::Run { cfg } => agglayer_node::main(cfg)?,
        cli::Commands::Config {} => println!(
            "{}",
            toml::to_string(&agglayer_config::Config::default()).unwrap()
        ),
    }

    Ok(())
}
