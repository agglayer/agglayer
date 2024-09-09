use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use serde::{Deserialize, Serialize};

use crate::{shutdown::ShutdownConfig, telemetry::TelemetryConfig, Log};

/// The Agglayer Prover configuration.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct ProverConfig {
    /// The gRPC endpoint used by the prover.
    #[serde(default = "default_socket_addr")]
    pub grpc_endpoint: SocketAddr,
    /// The log configuration.
    #[serde(alias = "Log")]
    pub log: Log,

    /// Telemetry configuration.
    #[serde(alias = "Telemetry")]
    pub telemetry: TelemetryConfig,

    /// The list of configuration options used during shutdown.
    #[serde(default)]
    pub shutdown: ShutdownConfig,
}

impl Default for ProverConfig {
    fn default() -> Self {
        Self {
            grpc_endpoint: default_socket_addr(),
            log: Log::default(),
            telemetry: TelemetryConfig::default(),
            shutdown: ShutdownConfig::default(),
        }
    }
}

const fn default_socket_addr() -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080)
}
