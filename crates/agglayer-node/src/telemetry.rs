use std::net::SocketAddr;

use axum::{
    extract::State,
    http::{Response, StatusCode},
    routing::get,
    serve::Serve,
    Router,
};
use lazy_static::lazy_static;
use opentelemetry::global;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use prometheus::{Encoder as _, TextEncoder};
use tracing::info;

pub(crate) const AGGLAYER_RPC_OTEL_SCOPE_NAME: &str = "rpc";
pub(crate) const AGGLAYER_KERNEL_OTEL_SCOPE_NAME: &str = "kernel";

lazy_static! {
    // Backward compatibility with the old metrics from agglayer go implementation
    // Those metrics are not linked to any registry
    pub static ref SEND_TX: opentelemetry::metrics::Counter<u64> = global::meter(AGGLAYER_RPC_OTEL_SCOPE_NAME)
        .u64_counter("send_tx")
        .with_description("Number of transactions received on the RPC")
        .init();

    pub static ref VERIFY_ZKP: opentelemetry::metrics::Counter<u64> = global::meter(AGGLAYER_KERNEL_OTEL_SCOPE_NAME)
        .u64_counter("verify_zkp")
        .with_description("Number of ZKP verifications")
        .init();

    pub static ref VERIFY_SIGNATURE: opentelemetry::metrics::Counter<u64> = global::meter(AGGLAYER_KERNEL_OTEL_SCOPE_NAME)
        .u64_counter("verify_signature")
        .with_description("Number of signature verifications")
        .init();

    pub static ref CHECK_TX: opentelemetry::metrics::Counter<u64> = global::meter(AGGLAYER_KERNEL_OTEL_SCOPE_NAME)
        .u64_counter("check_tx")
        .with_description("Number of transactions checked")
        .init();

    pub static ref EXECUTE: opentelemetry::metrics::Counter<u64> = global::meter(AGGLAYER_KERNEL_OTEL_SCOPE_NAME)
        .u64_counter("execute")
        .with_description("Number of transactions executed")
        .init();

    pub static ref SETTLE: opentelemetry::metrics::Counter<u64> = global::meter(AGGLAYER_KERNEL_OTEL_SCOPE_NAME)
        .u64_counter("settle")
        .with_description("Number of transactions settled")
        .init();
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("Unable to bind metrics server: {0}")]
    UnableToBindMetricsServer(#[from] std::io::Error),

    #[error("Metrics server address is not set")]
    MetricsServerAddressNotSet,
}

#[derive(Debug, thiserror::Error)]
enum MetricsError {
    #[error("Error gathering metrics: {0}")]
    GatheringMetrics(#[from] prometheus::Error),

    #[error("Error formatting metrics: {0}")]
    FormattingMetrics(#[from] std::string::FromUtf8Error),

    #[error("Error exporting metrics: {0}")]
    OpenTelemetry(#[from] opentelemetry::metrics::MetricsError),
}

#[derive(Default, Clone)]
pub struct ServerBuilder {
    serve_addr: Option<SocketAddr>,
    registry: prometheus::Registry,
}

impl ServerBuilder {
    pub fn serve_addr(mut self, addr: Option<SocketAddr>) -> Self {
        self.serve_addr = addr;

        self
    }

    pub async fn build(
        mut self,
    ) -> Result<Serve<axum::routing::IntoMakeService<Router>, axum::Router>, Error> {
        let _ = self.init_meter_provider();

        let serve_addr = self
            .serve_addr
            .take()
            .ok_or(Error::MetricsServerAddressNotSet)?;

        let app = Router::new()
            .route(
                "/metrics",
                get(|State(registry): State<prometheus::Registry>| async move {
                    match Self::gather_metrics(&registry) {
                        Ok(metrics) => Response::new(metrics),
                        Err(error) => Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .body(error.to_string())
                            .unwrap(),
                    }
                }),
            )
            .with_state(self.registry);

        info!("Starting metrics server on {}", serve_addr);

        let listener = tokio::net::TcpListener::bind(serve_addr).await?;
        Ok(axum::serve(listener, app.into_make_service()))
    }

    fn init_meter_provider(&mut self) -> Result<(), MetricsError> {
        // configure OpenTelemetry to use the registry
        let exporter = opentelemetry_prometheus::exporter()
            .with_registry(self.registry.clone())
            .build()?;

        // set up a meter meter to create instruments
        let provider = SdkMeterProvider::builder().with_reader(exporter).build();

        global::set_meter_provider(provider);
        Ok(())
    }

    fn gather_metrics(registry: &prometheus::Registry) -> Result<String, MetricsError> {
        // Encode data as text or protobuf
        let encoder = TextEncoder::new();
        let metric_families = registry.gather();
        let mut result = Vec::new();
        encoder.encode(&metric_families, &mut result)?;

        Ok(String::from_utf8(result)?)
    }
}
