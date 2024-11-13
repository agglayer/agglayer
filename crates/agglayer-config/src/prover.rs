use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::Duration,
};

use serde::{Deserialize, Serialize};
use serde_with::serde_as;

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

    /// The maximum number of concurrent queries the prover can handle.
    #[serde(default = "default_max_concurrency_limit")]
    pub max_concurrency_limit: usize,

    /// The maximum duration of a request.
    #[serde(default = "default_max_request_duration")]
    #[serde(with = "crate::with::HumanDuration")]
    pub max_request_duration: Duration,

    /// The maximum number of buffered queries.
    #[serde(default = "default_max_buffered_queries")]
    pub max_buffered_queries: usize,

    /// The CPU prover configuration.
    #[serde(default)]
    pub cpu_prover: CpuProverConfig,

    /// The network prover configuration.
    #[serde(default)]
    pub network_prover: NetworkProverConfig,

    /// The GPU prover configuration.
    #[serde(default)]
    pub gpu_prover: GpuProverConfig,
}

impl Default for ProverConfig {
    fn default() -> Self {
        Self {
            grpc_endpoint: default_socket_addr(),
            log: Log::default(),
            telemetry: TelemetryConfig::default(),
            shutdown: ShutdownConfig::default(),
            max_concurrency_limit: default_max_concurrency_limit(),
            max_request_duration: default_max_request_duration(),
            max_buffered_queries: default_max_buffered_queries(),
            cpu_prover: CpuProverConfig::default(),
            network_prover: NetworkProverConfig::default(),
            gpu_prover: GpuProverConfig::default(),
        }
    }
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct CpuProverConfig {
    #[serde(default = "default_activation_cpu_prover")]
    pub enabled: bool,

    #[serde(default = "default_max_concurrency_limit")]
    pub max_concurrency_limit: usize,

    #[serde_as(as = "Option<crate::with::HumanDuration>")]
    pub proving_request_timeout: Option<Duration>,

    #[serde(default = "default_cpu_proving_timeout")]
    #[serde(with = "crate::with::HumanDuration")]
    pub proving_timeout: Duration,
}

impl CpuProverConfig {
    // This constant represents the number of second added to the proving_timeout
    pub const DEFAULT_PROVING_TIMEOUT_PADDING: Duration = Duration::from_secs(1);

    pub fn get_proving_request_timeout(&self) -> Duration {
        self.proving_request_timeout
            .unwrap_or_else(|| self.proving_timeout + Self::DEFAULT_PROVING_TIMEOUT_PADDING)
    }
}

impl Default for CpuProverConfig {
    fn default() -> Self {
        Self {
            enabled: default_activation_cpu_prover(),
            max_concurrency_limit: default_max_concurrency_limit(),
            proving_request_timeout: None,
            proving_timeout: default_cpu_proving_timeout(),
        }
    }
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct NetworkProverConfig {
    #[serde(default = "default_activation_network_prover")]
    pub enabled: bool,

    #[serde_as(as = "Option<crate::with::HumanDuration>")]
    pub proving_request_timeout: Option<Duration>,

    #[serde(default = "default_network_proving_timeout")]
    #[serde(with = "crate::with::HumanDuration")]
    pub proving_timeout: Duration,
}

impl NetworkProverConfig {
    // This constant represents the number of second added to the proving_timeout
    pub const DEFAULT_PROVING_TIMEOUT_PADDING: Duration = Duration::from_secs(1);

    pub fn get_proving_request_timeout(&self) -> Duration {
        self.proving_request_timeout
            .unwrap_or_else(|| self.proving_timeout + Self::DEFAULT_PROVING_TIMEOUT_PADDING)
    }
}

impl Default for NetworkProverConfig {
    fn default() -> Self {
        Self {
            enabled: default_activation_network_prover(),
            proving_request_timeout: None,
            proving_timeout: default_network_proving_timeout(),
        }
    }
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct GpuProverConfig {
    #[serde(default = "default_activation_gpu_prover")]
    pub enabled: bool,

    #[serde(default = "default_max_concurrency_limit")]
    pub max_concurrency_limit: usize,

    #[serde_as(as = "Option<crate::with::HumanDuration>")]
    pub proving_request_timeout: Option<Duration>,

    #[serde(default = "default_cpu_proving_timeout")]
    #[serde(with = "crate::with::HumanDuration")]
    pub proving_timeout: Duration,
}

impl GpuProverConfig {
    // This constant represents the number of second added to the proving_timeout
    pub const DEFAULT_PROVING_TIMEOUT_PADDING: Duration = Duration::from_secs(1);

    pub fn get_proving_request_timeout(&self) -> Duration {
        self.proving_request_timeout
            .unwrap_or_else(|| self.proving_timeout + Self::DEFAULT_PROVING_TIMEOUT_PADDING)
    }
}

impl Default for GpuProverConfig {
    fn default() -> Self {
        Self {
            enabled: default_activation_gpu_prover(),
            max_concurrency_limit: default_max_concurrency_limit(),
            proving_request_timeout: None,
            proving_timeout: default_cpu_proving_timeout(),
        }
    }
}

const fn default_socket_addr() -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080)
}

const fn default_activation_cpu_prover() -> bool {
    true
}

const fn default_activation_network_prover() -> bool {
    false
}

const fn default_activation_gpu_prover() -> bool {
    false
}

pub(crate) fn default_prover_entrypoint() -> String {
    format!("http://{}", default_socket_addr())
}

const fn default_max_concurrency_limit() -> usize {
    100
}

const fn default_max_buffered_queries() -> usize {
    100
}

const fn default_max_request_duration() -> Duration {
    Duration::from_secs(60 * 5)
}

const fn default_cpu_proving_timeout() -> Duration {
    Duration::from_secs(60 * 5)
}

const fn default_network_proving_timeout() -> Duration {
    Duration::from_secs(60 * 5)
}
