use std::{future::IntoFuture, path::PathBuf, sync::Arc};

use agglayer_config::prover::ProverConfig;
use agglayer_telemetry::ServerBuilder as MetricsBuilder;
use anyhow::Result;
use prover::Prover;
use sp1_sdk::HashableKey;
use tokio_util::sync::CancellationToken;
use tracing::info;

// TODO: Mutualize with agglayer-node
mod logging;

mod executor;
#[cfg(feature = "testutils")]
pub mod fake;
pub mod prover;
mod rpc;

/// This is the main prover entrypoint.
///
/// This function starts everything needed to run an Agglayer Prover.
/// Starting by a Tokio runtime which can be used by the different components.
/// The configuration file is parsed and used to configure the prover.
///
/// This function returns on fatal error or after graceful shutdown has
/// completed.
pub fn main(cfg: PathBuf) -> Result<()> {
    // Load the configuration file
    let config: Arc<ProverConfig> = Arc::new(toml::from_str(&std::fs::read_to_string(cfg)?)?);

    let global_cancellation_token = CancellationToken::new();

    // Initialize the logger
    logging::tracing(&config.log);

    let node_runtime = tokio::runtime::Builder::new_multi_thread()
        .thread_name("agglayer-prover-runtime")
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
        Prover::builder()
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
                _ = terminate_signal => {
                    info!("Received SIGTERM, shutting down...");
                    // Cancel the global cancellation token to start the shutdown process.
                    global_cancellation_token.cancel();
                    // Wait for the node to shutdown.
                    node.await_shutdown().await;
                    // Wait for the metrics server to shutdown.
                    _ = metrics_handle.await;
                }
                _ = tokio::signal::ctrl_c() => {
                    info!("Received SIGINT (ctrl-c), shutting down...");
                    // Cancel the global cancellation token to start the shutdown process.
                    global_cancellation_token.cancel();
                    // Wait for the node to shutdown.
                    node.await_shutdown().await;
                    // Wait for the metrics server to shutdown.
                    _ = metrics_handle.await;
                }
            }
        });

    node_runtime.shutdown_timeout(config.shutdown.runtime_timeout);
    metrics_runtime.shutdown_timeout(config.shutdown.runtime_timeout);

    Ok(())
}

pub fn get_vkey() -> String {
    let vkey = executor::Executor::get_vkey();
    vkey.bytes32().to_string()
}

#[cfg(feature = "testutils")]
#[tokio::main]
pub async fn start_prover(config: Arc<ProverConfig>, global_cancellation_token: CancellationToken) {
    let prover = Prover::builder()
        .config(config)
        .cancellation_token(global_cancellation_token)
        .start()
        .await
        .unwrap();
    prover.await_shutdown().await;
}
