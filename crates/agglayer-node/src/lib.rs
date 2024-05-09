use std::{future::IntoFuture, path::PathBuf, sync::Arc, time::Duration};

use agglayer_config::Config;
use anyhow::Result;
use node::Node;
use tokio_util::sync::CancellationToken;
use tracing::info;

mod contracts;
mod kernel;
mod logging;
mod rpc;
mod signed_tx;
mod zkevm_node_client;

mod node;

use agglayer_telemetry::ServerBuilder as MetricsBuilder;

/// This is the main node entrypoint.
///
/// This function starts everything needed to run an Agglayer node.
/// Starting by a Tokio runtime which can be used by the different components.
/// The configuration file is parsed and used to configure the node.
///
/// This function returns on fatal error or after graceful shutdown has
/// completed.
pub fn main(cfg: PathBuf) -> Result<()> {
    // Load the configuration file
    let config: Arc<Config> = Arc::new(toml::from_str(&std::fs::read_to_string(cfg)?)?);

    let global_cancellation_token = CancellationToken::new();

    // Initialize the logger
    logging::tracing(&config.log);

    let node_runtime = tokio::runtime::Builder::new_multi_thread()
        .thread_name("agglayer-node-runtime")
        .enable_all()
        .build()?;

    let metrics_runtime = tokio::runtime::Builder::new_multi_thread()
        .thread_name("metrics-runtime")
        .worker_threads(2)
        .enable_all()
        .build()?;

    let metric_server = metrics_runtime.block_on(
        MetricsBuilder::builder()
            .addr(config.telemetry.addr)
            .cancellation_token(global_cancellation_token.clone())
            .build(),
    )?;

    let metrics_handle = {
        let _guard = metrics_runtime.enter();
        // Spawn the metrics server
        metrics_runtime.spawn(metric_server.into_future())
    };

    // Spawn the node
    let node = node_runtime.block_on(
        Node::builder()
            .config(config)
            .cancellation_token(global_cancellation_token.clone())
            .start(),
    )?;

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(async {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    info!("Received SIGINT (ctrl-c), shutting down...");
                    global_cancellation_token.cancel();
                    node.await_shutdown().await;
                    _ = metrics_handle.await;
                }
            }
        });

    node_runtime.shutdown_timeout(Duration::from_secs(5));
    metrics_runtime.shutdown_timeout(Duration::from_secs(5));

    Ok(())
}
