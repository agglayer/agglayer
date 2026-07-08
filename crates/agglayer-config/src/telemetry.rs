use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use super::DEFAULT_IP;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct TelemetryConfig {
    #[serde(
        rename = "prometheus-addr",
        alias = "PrometheusAddr",
        default = "default_metrics_api_addr"
    )]
    pub addr: SocketAddr,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            addr: default_metrics_api_addr(),
        }
    }
}

const fn default_metrics_api_addr() -> SocketAddr {
    SocketAddr::V4(std::net::SocketAddrV4::new(DEFAULT_IP, 3000))
}
