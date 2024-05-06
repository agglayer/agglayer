use clap::Parser;
use cli::Cli;

mod cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    match cli.cmd {
        cli::Commands::Run { cfg } => agglayer_node::run(cfg).await?,
    }

    Ok(())
}
