use std::{net::SocketAddr, path::PathBuf};

use alloy::{providers::ProviderBuilder, rpc::client::ClientBuilder};
use clap::Parser;
use cli::Cli;
use config::Config;
use jsonrpsee::server::Server;
use kernel::Kernel;
use rpc::{AgglayerImpl, AgglayerServer};
use tracing::{debug, info};

mod cli;
mod config;
mod contracts;
mod kernel;
mod logging;
mod rpc;
mod signed_tx;
mod zkevm_node_client;

async fn run(cfg: PathBuf) -> anyhow::Result<()> {
    let config: Config = toml::from_str(&std::fs::read_to_string(cfg)?)?;
    config.set_log_env();
    logging::tracing(&config.log);

    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(config.grpc.port);
    let addr = SocketAddr::from((config.grpc.host, port));

    // Create the signer based on the configuration.
    let signer = config.get_configured_signer().await?;
    debug!("Signer successfully created");

    // Create the client for the L1 that the RPC provider will use.
    let client = ClientBuilder::default().http(config.l1.node_url.clone());

    // Create the provider with the signer and client.
    // The provider is using recommanded fillers for the moment.
    // Which include a set of layers to handle gas estimation, nonce
    // management, and chain-id fetching.
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .signer(signer)
        .on_client(client.boxed());

    debug!("Provider successfully created");
    // Construct the core.
    let core = Kernel::new(provider, config);
    // Bind the core to the RPC server.
    let service = AgglayerImpl::new(core).into_rpc();

    let server = Server::builder().build(addr).await?;
    let handle = server.start(service);

    info!("Agglayer started and listening on {addr}");
    handle.stopped().await;

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    match cli.cmd {
        cli::Commands::Run { cfg } => run(cfg).await?,
    }

    Ok(())
}
