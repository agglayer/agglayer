use std::net::SocketAddr;

use serde::Deserialize;

use super::DEFAULT_IP;

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct TelemetryConfig {
    #[serde(rename = "PrometheusAddr", default = "default_metrics_api_addr")]
    pub(crate) addr: SocketAddr,
}

const fn default_metrics_api_addr() -> SocketAddr {
    SocketAddr::V4(std::net::SocketAddrV4::new(DEFAULT_IP, 3000))
}
