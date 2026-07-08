use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use super::DEFAULT_IP;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct TelemetryConfig {
    #[serde(
        rename = "prometheus-addr",
        alias = "PrometheusAddr",
        default = "default_metrics_api_addr"
    )]
    pub addr: SocketAddr,

    /// Deployment environment label (e.g. "bali", "cardona", "mainnet") applied
    /// to certificate metrics. Set by the deployment; defaults to "unknown".
    #[serde(default = "default_environment")]
    pub environment: String,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            addr: default_metrics_api_addr(),
            environment: default_environment(),
        }
    }
}

const fn default_metrics_api_addr() -> SocketAddr {
    SocketAddr::V4(std::net::SocketAddrV4::new(DEFAULT_IP, 3000))
}

fn default_environment() -> String {
    "unknown".to_string()
}
