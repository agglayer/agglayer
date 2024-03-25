use std::net::SocketAddr;

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
mod init;
mod kernel;
mod rpc;
mod signed_proof;
mod zkevm_node_client;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    init::tracing();

    let cli = Cli::parse();
    let config: Config = toml::from_str(&std::fs::read_to_string(cli.config_path)?)?;

    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(config.grpc.port);
    let addr = SocketAddr::from((config.grpc.host, port));

    let rpc = Provider::<Http>::try_from(config.l1.node_url.as_str())?;
    let core = Kernel::new(KernelArgs { rpc, config });
    let service = AgglayerImpl::new(core).into_rpc();

    info!("Listening on {addr}");
    let server = Server::builder().build(addr).await?;
    let handle = server.start(service);
    handle.stopped().await;

    Ok(())
}
