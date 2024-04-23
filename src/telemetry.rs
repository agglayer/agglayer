use std::{collections::HashMap, net::SocketAddr};

use axum::{
    http::{Response, StatusCode},
    routing::get,
    serve::Serve,
    Router,
};
use lazy_static::lazy_static;
use prometheus::{register_int_counter_vec, Encoder as _, IntCounterVec, Registry, TextEncoder};
use tracing::info;

lazy_static! {
    pub static ref AGGLAYER_METRIC_REGISTRY: Registry =
        Registry::new_custom(Some("agglayer".to_string()), Some(HashMap::from([]))).unwrap();

    // Backward compatibility with the old metrics from agglayer go implementation
    // Those metrics are not linked to any registry
    pub static ref SEND_TX: IntCounterVec = register_int_counter_vec!(
        "send_tx",
        "Number of transactions received on the RPC",
        &["rollup_id"]
    )
    .unwrap();

    pub static ref VERIFY_ZKP: IntCounterVec = register_int_counter_vec!(
        "verify_zkp",
        "Number of ZKP verifications",
        &["rollup_id"]
    )
    .unwrap();

    pub static ref VERIFY_SIGNATURE: IntCounterVec = register_int_counter_vec!(
        "verify_signature",
        "Number of signature verifications",
        &["rollup_id"]
    )
    .unwrap();

    pub static ref CHECK_TX: IntCounterVec = register_int_counter_vec!(
        "check_tx",
        "Number of transactions checked",
        &["rollup_id"]
    )
    .unwrap();

    pub static ref EXECUTE: IntCounterVec = register_int_counter_vec!(
        "execute",
        "Number of transactions executed",
        &["rollup_id"]
    )
    .unwrap();

    pub static ref SETTLE: IntCounterVec = register_int_counter_vec!(
        "settle",
        "Number of transactions settled",
        &["rollup_id"]
    )
    .unwrap();
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("Unable to bind metrics server: {0}")]
    UnableToBindMetricsServer(#[from] std::io::Error),

    #[error("Metrics server address is not set")]
    MetricsServerAddressNotSet,
}

#[derive(Default)]
pub struct ServerBuilder {
    serve_addr: Option<SocketAddr>,
}

impl ServerBuilder {
    pub fn serve_addr(mut self, addr: Option<SocketAddr>) -> Self {
        self.serve_addr = addr;

        self
    }

    pub async fn build(
        mut self,
    ) -> Result<Serve<axum::routing::IntoMakeService<Router>, axum::Router>, Error> {
        init_metrics();

        let serve_addr = self
            .serve_addr
            .take()
            .ok_or(Error::MetricsServerAddressNotSet)?;

        let app = Router::new().route(
            "/metrics",
            get(|| async {
                match gather_metrics() {
                    Ok(metrics) => Response::new(metrics),
                    Err(error) => Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(error.to_string())
                        .unwrap(),
                }
            }),
        );

        info!("Starting metrics server on {}", serve_addr);

        let listener = tokio::net::TcpListener::bind(serve_addr).await?;
        Ok(axum::serve(listener, app.into_make_service()))
    }
}

#[derive(Debug, thiserror::Error)]
enum MetricsError {
    #[error("Error gathering metrics: {0}")]
    GatheringMetrics(#[from] prometheus::Error),

    #[error("Error formatting metrics: {0}")]
    FormattingMetrics(#[from] std::string::FromUtf8Error),
}

fn gather_metrics() -> Result<String, MetricsError> {
    let mut buffer = Vec::new();
    let encoder = TextEncoder::new();

    // Gather the metrics.
    let metric_families = prometheus::gather();
    // Encode them to send.
    encoder.encode(&metric_families, &mut buffer)?;

    Ok(String::from_utf8(buffer)?)
}

fn init_metrics() {
    SETTLE.reset();
    VERIFY_ZKP.reset();
    VERIFY_SIGNATURE.reset();
    SEND_TX.reset();
    CHECK_TX.reset();
    EXECUTE.reset();
}
