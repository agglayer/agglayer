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

    // Attempt to decrypt the first local wallet in the configuration.
    let wallet = config.local_wallet()?;
    // Create a new L1 RPC provider.
    let rpc = Provider::<Http>::try_from(config.l1.node_url.as_str())?;
    // Link the wallet to the provider for automatic transaction signing.
    let signer_middleware = SignerMiddleware::new(rpc, wallet);
    // Construct the core.
    let core = Kernel::new(KernelArgs {
        rpc: signer_middleware,
        config,
    });
    // Bind the core to the RPC server.
    let service = AgglayerImpl::new(core).into_rpc();

    info!("Listening on {addr}");
    let server = Server::builder().build(addr).await?;
    let handle = server.start(service);
    handle.stopped().await;

    Ok(())
}
