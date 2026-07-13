use std::net::SocketAddr;

use axum::{
    extract::State,
    http::{Response, StatusCode},
    routing::get,
    serve::WithGracefulShutdown,
    Router,
};
use eyre::Context as _;
use lazy_static::lazy_static;
use opentelemetry::global;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use prometheus::{Encoder as _, Registry, TextEncoder};
use tokio_util::sync::CancellationToken;
use tracing::{debug, info};

use crate::constant::{AGGLAYER_KERNEL_OTEL_SCOPE_NAME, AGGLAYER_RPC_OTEL_SCOPE_NAME};

mod constant;

pub mod certificate;
pub mod clock;
pub mod network;
pub mod settlement;

// Testing.
#[cfg(feature = "testutils")]
pub mod testutils;

pub use opentelemetry::KeyValue;

lazy_static! {
    // Backward compatibility with the old metrics from agglayer go implementation
    // Those metrics are not linked to any registry
    pub static ref SEND_TX: opentelemetry::metrics::Counter<u64> = global::meter(AGGLAYER_RPC_OTEL_SCOPE_NAME)
        .u64_counter("send_tx")
        .with_description("Number of transactions received on the RPC")
        .build();

    pub static ref VERIFY_ZKP: opentelemetry::metrics::Counter<u64> = global::meter(AGGLAYER_KERNEL_OTEL_SCOPE_NAME)
        .u64_counter("verify_zkp")
        .with_description("Number of ZKP verifications")
        .build();

    pub static ref VERIFY_SIGNATURE: opentelemetry::metrics::Counter<u64> = global::meter(AGGLAYER_KERNEL_OTEL_SCOPE_NAME)
        .u64_counter("verify_signature")
        .with_description("Number of signature verifications")
        .build();

    pub static ref CHECK_TX: opentelemetry::metrics::Counter<u64> = global::meter(AGGLAYER_KERNEL_OTEL_SCOPE_NAME)
        .u64_counter("check_tx")
        .with_description("Number of transactions checked")
        .build();

    pub static ref EXECUTE: opentelemetry::metrics::Counter<u64> = global::meter(AGGLAYER_KERNEL_OTEL_SCOPE_NAME)
        .u64_counter("execute")
        .with_description("Number of transactions executed")
        .build();

    pub static ref SETTLE: opentelemetry::metrics::Counter<u64> = global::meter(AGGLAYER_KERNEL_OTEL_SCOPE_NAME)
        .u64_counter("settle")
        .with_description("Number of transactions settled")
        .build();
}

pub mod prover {
    use lazy_static::lazy_static;
    use opentelemetry::global;

    use crate::constant::AGGLAYER_PROVER_RPC_OTEL_SCOPE_NAME;

    lazy_static! {
        pub static ref PROVING_REQUEST_RECV: opentelemetry::metrics::Counter<u64> =
            global::meter(AGGLAYER_PROVER_RPC_OTEL_SCOPE_NAME)
                .u64_counter("proving_request_recv")
                .with_description("Number of proving request received")
                .build();
        pub static ref PROVING_REQUEST_SUCCEEDED: opentelemetry::metrics::Counter<u64> =
            global::meter(AGGLAYER_PROVER_RPC_OTEL_SCOPE_NAME)
                .u64_counter("proving_request_succeeded")
                .with_description("Number of proving request that succeeded")
                .build();
        pub static ref PROVING_REQUEST_FAILED: opentelemetry::metrics::Counter<u64> =
            global::meter(AGGLAYER_PROVER_RPC_OTEL_SCOPE_NAME)
                .u64_counter("proving_request_failed")
                .with_description("Number of proving request that failed")
                .build();
        pub static ref PROVING_FALLBACK_TRIGGERED: opentelemetry::metrics::Counter<u64> =
            global::meter(AGGLAYER_PROVER_RPC_OTEL_SCOPE_NAME)
                .u64_counter("proving_fallback_triggered")
                .with_description("Number of proving fallback triggered")
                .build();
    }
}

pub struct ServerBuilder {}

#[buildstructor::buildstructor]
impl ServerBuilder {
    /// Function that builds a new Metrics server and returns a
    /// [`WithGracefulShutdown`] instance ready to be spawn.
    ///
    /// The available methods are:
    ///
    /// - `builder`: Creates a new builder instance.
    /// - `addr`: Sets the [`SocketAddr`] to bind the metrics server to.
    /// - `registry`: Sets the [`Registry`] to use for metrics. (optional)
    /// - `build`: Builds the metrics server and returns a
    ///   [`WithGracefulShutdown`] instance.
    ///
    /// # Examples
    /// ```
    /// # use std::sync::Arc;
    /// # use agglayer_telemetry::ServerBuilder;
    /// # use tokio_util::sync::CancellationToken;
    /// # use std::net::SocketAddr;
    /// #
    ///
    /// async fn build_metrics() -> eyre::Result<()> {
    ///     ServerBuilder::builder()
    ///         .addr("127.0.0.1".parse::<SocketAddr>().unwrap())
    ///         .cancellation_token(CancellationToken::new())
    ///         .build()
    ///         .await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    ///
    /// # Panics
    ///
    /// Panics on failure of the internal meter provider setup (unlikely)
    ///
    /// # Errors
    ///
    /// This function will return an error if the provided addr is invalid
    #[builder(entry = "builder", exit = "build", visibility = "pub")]
    pub async fn serve(
        addr: SocketAddr,
        registry: Option<Registry>,
        cancellation_token: CancellationToken,
    ) -> eyre::Result<
        WithGracefulShutdown<
            tokio::net::TcpListener,
            axum::routing::IntoMakeService<Router>,
            axum::Router,
            impl futures::Future<Output = ()>,
        >,
    > {
        let registry = registry.unwrap_or_default();
        init_meter_provider(&registry);

        let app = Router::new()
            .route(
                "/metrics",
                get(|State(registry): State<prometheus::Registry>| async move {
                    match encode_registry(&registry) {
                        Ok(metrics) => Response::new(metrics),
                        Err(error) => Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .body(error.to_string())
                            .unwrap(),
                    }
                }),
            )
            .with_state(registry);

        info!("Starting metrics server on {}", addr);

        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .wrap_err_with(|| format!("Unable to bind metrics server on {addr}"))?;

        Ok(axum::serve(listener, app.into_make_service())
            .with_graceful_shutdown(shutdown_signal(cancellation_token)))
    }
}

/// Installs a fresh [`SdkMeterProvider`] backed by `registry` as the
/// process-global meter provider.
fn init_meter_provider(registry: &Registry) {
    // configure OpenTelemetry to use the registry
    let exporter = opentelemetry_prometheus::exporter()
        .with_registry(registry.clone())
        .build()
        .unwrap();

    // set up a meter to create instruments
    let provider = SdkMeterProvider::builder().with_reader(exporter).build();

    global::set_meter_provider(provider);
}

/// Encodes the current content of `registry` as prometheus text.
fn encode_registry(registry: &Registry) -> eyre::Result<String> {
    let encoder = TextEncoder::new();
    let metric_families = registry.gather();
    let mut result = Vec::new();
    encoder
        .encode(&metric_families, &mut result)
        .wrap_err("Error gathering metrics")?;

    String::from_utf8(result).wrap_err("Error formatting metrics")
}

async fn shutdown_signal(cancellation: CancellationToken) {
    tokio::select! {
        _ = cancellation.cancelled() => {
            debug!("Shutting down metrics server...");
        },
    }
}
