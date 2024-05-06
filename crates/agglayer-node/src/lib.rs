use std::{future::IntoFuture, path::PathBuf, sync::Arc};

use agglayer_config::Config;
use ethers::prelude::*;
use jsonrpsee::server::{middleware::http::ProxyGetRequestLayer, PingConfig, ServerBuilder};
use kernel::Kernel;
use rpc::{AgglayerImpl, AgglayerServer};
use tokio::spawn;
use tower_http::cors::CorsLayer;
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

    let addr = config.rpc_addr();
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
    let mut service = AgglayerImpl::new(core).into_rpc();
    service.register_method("system_health", |_, _| {
        println!("system_health");
        serde_json::json!({ "health": true })
    })?;

    let mut server_builder = ServerBuilder::new()
        .max_request_body_size(config.rpc.max_request_body_size)
        .max_response_body_size(config.rpc.max_response_body_size)
        .max_connections(config.rpc.max_connections)
        .set_batch_request_config(match config.rpc.batch_request_limit {
            None => jsonrpsee::server::BatchRequestConfig::Unlimited,
            Some(0) => jsonrpsee::server::BatchRequestConfig::Disabled,
            Some(n) => jsonrpsee::server::BatchRequestConfig::Limit(n),
        });

    if let Some(duration) = config.rpc.ping_interval {
        server_builder =
            server_builder.enable_ws_ping(PingConfig::default().ping_interval(duration));
    }

    let cors = CorsLayer::new()
        .allow_methods([
            hyper::Method::POST,
            hyper::Method::GET,
            hyper::Method::OPTIONS,
        ])
        .allow_origin(tower_http::cors::Any)
        .allow_headers([hyper::header::CONTENT_TYPE]);

    let middleware = tower::ServiceBuilder::new()
        .layer(ProxyGetRequestLayer::new("/health", "system_health")?)
        .layer(cors);

    let server = server_builder
        .set_http_middleware(middleware)
        .build(addr)
        .await?;

    info!("Listening on {addr}");
    let server_handle = server.start(service);
    server_handle.stopped().await;

    Ok(())
}
