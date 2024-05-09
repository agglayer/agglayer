use std::{future::IntoFuture, path::PathBuf, sync::Arc};

use agglayer_config::Config;
use anyhow::Result;
use node::Node;

mod contracts;
mod kernel;
mod logging;
mod rpc;
mod signed_tx;
mod telemetry;
mod zkevm_node_client;

mod node;

use telemetry::ServerBuilder as MetricsBuilder;

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

    // Initialize the logger
    logging::tracing(&config.log);

    let node_runtime = tokio::runtime::Builder::new_multi_thread()
        .thread_name("agglayer-node-runtime")
        .enable_all()
        .build()?;

    let metrics_runtime = tokio::runtime::Builder::new_multi_thread()
        .thread_name("metrics-runtime")
        .enable_all()
        .build()?;

    // Spawn the metrics server
    metrics_runtime.spawn(
        metrics_runtime
            .block_on(
                MetricsBuilder::builder()
                    .addr(config.telemetry.addr)
                    .build(),
            )?
            .into_future(),
    );

    // Spawn the node
    node_runtime.block_on(Node::builder().config(config).start())?;

    Ok(())
}
