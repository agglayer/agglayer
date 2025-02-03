use std::{future::IntoFuture, path::PathBuf, sync::Arc};

use agglayer_config::Config;
use agglayer_rate_limiting as rate_limiting;
use anyhow::{bail, Result};
use node::Node;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info};

mod kernel;
mod logging;
mod rpc;
pub mod service;
mod signed_tx;
mod zkevm_node_client;

mod epoch_synchronizer;
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
pub fn main(cfg: PathBuf, version: &str) -> Result<()> {
    let cfg = cfg.canonicalize().map_err(|_| {
        anyhow::Error::msg(format!(
            "Configuration file path must be absolute, given: {}",
            cfg.display()
        ))
    })?;

    let config: Arc<Config> = if cfg.is_file() {
        let config = Config::try_load(cfg.as_path())?;
        Arc::new(config)
    } else {
        bail!(
            "Provided configuration file path is not a file: {}",
            cfg.display()
        )
    };

    let global_cancellation_token = CancellationToken::new();

    // Initialize the logger
    logging::tracing(&config.log);

    info!("Starting agglayer node version info: {}", version);

    let node_runtime = tokio::runtime::Builder::new_multi_thread()
        .thread_name("agglayer-node-runtime")
        .enable_all()
        .build()?;

    let metrics_runtime = tokio::runtime::Builder::new_multi_thread()
        .thread_name("metrics-runtime")
        .worker_threads(2)
        .enable_all()
        .build()?;

    // Create the metrics server.
    let metric_server = metrics_runtime.block_on(
        MetricsBuilder::builder()
            .addr(config.telemetry.addr)
            .cancellation_token(global_cancellation_token.clone())
            .build(),
    )?;

    // Spawn the metrics server into the metrics runtime.
    let metrics_handle = {
        // This guard is used to ensure that the metrics runtime is entered
        // before the server is spawned. This is necessary because the `into_future`
        // of `WithGracefulShutdown` is spawning various tasks before returning the
        // actual server instance to spawn.
        let _guard = metrics_runtime.enter();
        // Spawn the metrics server
        metrics_runtime.spawn(metric_server.into_future())
    };

    // Spawn the node.
    let node = node_runtime.block_on(
        Node::builder()
            .config(config.clone())
            .cancellation_token(global_cancellation_token.clone())
            .start(),
    )?;
    let terminate_signal = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Fail to setup SIGTERM signal")
            .recv()
            .await;
    };

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(async {
            tokio::select! {
                _ = node.await_shutdown() => {
                    info!("Node has shutdown, shutting down...");
                    // Cancel the global cancellation token to start the shutdown process.
                    global_cancellation_token.cancel();
                    // Wait for the metrics server to shutdown.
                    _ = metrics_handle.await;
                }

                _ = terminate_signal => {
                    info!("Received SIGTERM, shutting down...");
                    // Cancel the global cancellation token to start the shutdown process.
                    global_cancellation_token.cancel();
                    // Wait for the metrics server to shutdown.
                    _ = metrics_handle.await;
                }
                _ = tokio::signal::ctrl_c() => {
                    info!("Received SIGINT (ctrl-c), shutting down...");
                    // Cancel the global cancellation token to start the shutdown process.
                    global_cancellation_token.cancel();
                    // Wait for the metrics server to shutdown.
                    _ = metrics_handle.await;
                }
            }
        });

    node_runtime.shutdown_timeout(config.shutdown.runtime_timeout);
    metrics_runtime.shutdown_timeout(config.shutdown.runtime_timeout);

    debug!("Node shutdown completed.");

    Ok(())
}
