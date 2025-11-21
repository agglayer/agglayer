use std::{path::PathBuf, sync::Arc};

use eyre::Context as _;
use prover_engine::ProverEngine;

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
pub fn main(cfg: PathBuf, version: &str, program: &'static [u8]) -> eyre::Result<()> {
    let config = Arc::new(agglayer_prover_config::ProverConfig::try_load(&cfg)?);

    // Initialize the logger
    prover_logger::tracing(&config.log);

    let global_cancellation_token = CancellationToken::new();

    info!("Starting agglayer prover version info: {}", version);

    let prover_runtime = tokio::runtime::Builder::new_multi_thread()
        .thread_name("agglayer-prover-runtime")
        .enable_all()
        .build()?;

    let metrics_runtime = tokio::runtime::Builder::new_multi_thread()
        .thread_name("metrics-runtime")
        .worker_threads(2)
        .enable_all()
        .build()?;

    let pp_service = prover_runtime
        .block_on(crate::prover::Prover::create_service(&config, program))
        .context("Failed to create PP service")?;

    ProverEngine::new(
        config.grpc_endpoint,
        config.telemetry.addr,
        config.shutdown.runtime_timeout,
    )
    .add_rpc_service(pp_service)
    .set_rpc_runtime(prover_runtime)
    .set_metrics_runtime(metrics_runtime)
    .set_cancellation_token(global_cancellation_token)
    .start()
}

pub async fn compute_program_vkey(program: &'static [u8]) -> eyre::Result<String> {
    let vkey = prover_executor::Executor::compute_program_vkey(program)
        .await
        .context("Failed to compute program vkey")?;
    Ok(vkey.bytes32())
}

#[cfg(feature = "testutils")]
mod testutils {
    use std::sync::Arc;

    use agglayer_prover_config::ProverConfig;
    use tokio_util::sync::CancellationToken;

    use super::prover::Prover;

    #[tokio::main]
    pub async fn start_prover(
        config: Arc<ProverConfig>,
        global_cancellation_token: CancellationToken,
        program: &'static [u8],
    ) {
        let prover = Prover::builder()
            .config(config)
            .cancellation_token(global_cancellation_token)
            .program(program)
            .start()
            .await
            .unwrap();
        prover.await_shutdown().await;
    }
}

use sp1_sdk::HashableKey as _;
#[cfg(feature = "testutils")]
pub use testutils::start_prover;
use tokio_util::sync::CancellationToken;
use tracing::info;
