use std::net::SocketAddr;

use serde::Deserialize;

pub(crate) const DEFAULT_IP: std::net::Ipv4Addr = std::net::Ipv4Addr::new(0, 0, 0, 0);
pub(crate) const AGGLAYER_RPC_OTEL_SCOPE_NAME: &str = "rpc";
pub(crate) const AGGLAYER_KERNEL_OTEL_SCOPE_NAME: &str = "kernel";

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "PascalCase")]
pub struct TelemetryConfig {
    #[serde(rename = "PrometheusAddr", default = "default_metrics_api_addr")]
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
