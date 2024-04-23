use std::{future::IntoFuture, path::PathBuf};

use clap::Parser;
use cli::Cli;
use config::Config;
use ethers::prelude::*;
use jsonrpsee::server::Server;
use kernel::Kernel;
use rpc::{AgglayerImpl, AgglayerServer};
use tokio::spawn;
use tracing::{info, Instrument as _};

mod cli;
mod config;
mod contracts;
mod kernel;
mod logging;
mod rpc;
mod signed_tx;
mod telemetry;
mod zkevm_node_client;

use telemetry::ServerBuilder as MetricsBuilder;

async fn run(cfg: PathBuf) -> anyhow::Result<()> {
    let config: Config = toml::from_str(&std::fs::read_to_string(cfg)?)?;
    logging::tracing(&config.log);

    let addr = config.rpc_addr();
    let telemetry_addr = config.telemetry.addr;

    // Create a new L1 RPC provider with the configured signer.
    let rpc = Provider::<Http>::try_from(config.l1.node_url.as_str())?
        .with_signer(config.get_configured_signer().await?);
    // Construct the core.
    let core = Kernel::new(rpc, config);
    // Bind the core to the RPC server.
    let service = AgglayerImpl::new(core).into_rpc();

    info!("Serving metrics on {}", telemetry_addr);
    let metrics_server = MetricsBuilder::default()
        .serve_addr(Some(telemetry_addr))
        .build()
        .in_current_span()
        .await?;

    let _metrics_handler = spawn(metrics_server.into_future());

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
