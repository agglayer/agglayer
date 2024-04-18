use std::{net::SocketAddr, path::PathBuf};

use clap::Parser;
use cli::Cli;
use config::Config;
use ethers::prelude::*;
use jsonrpsee::server::Server;
use kernel::{Kernel, KernelArgs};
use rpc::{AgglayerImpl, AgglayerServer};
use tracing::info;

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

    // Create a new L1 RPC provider with the configured signer.
    let rpc = Provider::<Http>::try_from(config.l1.node_url.as_str())?
        .with_signer(config.get_configured_signer().await?);
    // Construct the core.
    let core = Kernel::new(KernelArgs { rpc, config });
    // Bind the core to the RPC server.
    let service = AgglayerImpl::new(core).into_rpc();

    info!("Listening on {addr}");
    let server = Server::builder().build(addr).await?;
    let handle = server.start(service);
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
