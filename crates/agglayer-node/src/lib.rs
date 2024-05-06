use std::{future::IntoFuture, path::PathBuf, sync::Arc};

use agglayer_config::Config;
use ethers::prelude::*;
use kernel::Kernel;
use rpc::AgglayerImpl;
use tokio::spawn;
use tracing::{info, Instrument as _};

mod contracts;
mod kernel;
mod logging;
mod rpc;
mod signed_tx;
mod telemetry;
mod zkevm_node_client;

use telemetry::ServerBuilder as MetricsBuilder;

pub async fn run(cfg: PathBuf) -> anyhow::Result<()> {
    let config: Arc<Config> = Arc::new(toml::from_str(&std::fs::read_to_string(cfg)?)?);
    logging::tracing(&config.log);

    let telemetry_addr = config.telemetry.addr;

    // Create a new L1 RPC provider with the configured signer.
    let rpc = Provider::<Http>::try_from(config.l1.node_url.as_str())?
        .with_signer(config.get_configured_signer().await?);

    // Construct the core.
    let core = Kernel::new(rpc, config.clone());

    info!("Serving metrics on {}", telemetry_addr);
    let metrics_server = MetricsBuilder::default()
        .serve_addr(Some(telemetry_addr))
        .build()
        .in_current_span()
        .await?;

    let _metrics_handler = spawn(metrics_server.into_future());

    // Bind the core to the RPC server.
    let server_handle = AgglayerImpl::new(core).start(config).await?;

    server_handle.stopped().await;

    Ok(())
}
