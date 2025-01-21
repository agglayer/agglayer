use std::path::Path;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::Duration,
};

use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::{shutdown::ShutdownConfig, telemetry::TelemetryConfig, ConfigurationError, Log};

/// The Agglayer Prover configuration.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct ProverConfig {
    /// The gRPC endpoint used by the prover.
    #[serde(default = "default_socket_addr")]
    pub grpc_endpoint: SocketAddr,

    #[serde(default, skip_serializing_if = "crate::default")]
    pub grpc: GrpcConfig,

    /// The log configuration.
    #[serde(default, alias = "Log")]
    pub log: Log,

    /// Telemetry configuration.
    #[serde(default, alias = "Telemetry")]
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

    /// The primary prover to be used for generation of the pessimistic proof
    #[serde(default)]
    pub primary_prover: AgglayerProverType,

    /// The fallback prover to be used for generation of the pessimistic proof
    #[serde(default)]
    pub fallback_prover: Option<AgglayerProverType>,
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
            primary_prover: AgglayerProverType::NetworkProver(NetworkProverConfig::default()),
            fallback_prover: None,
            grpc: Default::default(),
        }
    }
}

impl ProverConfig {
    pub fn try_load(path: &Path) -> Result<Self, ConfigurationError> {
        let reader = std::fs::read_to_string(path).map_err(|source| {
            ConfigurationError::UnableToReadConfigFile {
                path: path.to_path_buf(),
                source,
            }
        })?;

        let deserializer = toml::de::Deserializer::new(&reader);
        serde::Deserialize::deserialize(deserializer)
            .map_err(ConfigurationError::DeserializationError)
    }
}

/// Type of the prover to be used for generation of the pessimistic proof
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AgglayerProverType {
    NetworkProver(NetworkProverConfig),
    CpuProver(CpuProverConfig),
    GpuProver(GpuProverConfig),
    MockProver(MockProverConfig),
}

impl Default for AgglayerProverType {
    fn default() -> Self {
        AgglayerProverType::NetworkProver(NetworkProverConfig::default())
    }
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct CpuProverConfig {
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
            proving_request_timeout: None,
            proving_timeout: default_network_proving_timeout(),
        }
    }
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct GpuProverConfig {
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
            max_concurrency_limit: default_max_concurrency_limit(),
            proving_request_timeout: None,
            proving_timeout: default_cpu_proving_timeout(),
        }
    }
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct MockProverConfig {
    #[serde(default = "default_max_concurrency_limit")]
    pub max_concurrency_limit: usize,

    #[serde_as(as = "Option<crate::with::HumanDuration>")]
    pub proving_request_timeout: Option<Duration>,

    #[serde(default = "default_cpu_proving_timeout")]
    #[serde(with = "crate::with::HumanDuration")]
    pub proving_timeout: Duration,
}

impl MockProverConfig {
    // This constant represents the number of second added to the proving_timeout
    pub const DEFAULT_PROVING_TIMEOUT_PADDING: Duration = Duration::from_secs(1);

    pub fn get_proving_request_timeout(&self) -> Duration {
        self.proving_request_timeout
            .unwrap_or_else(|| self.proving_timeout + Self::DEFAULT_PROVING_TIMEOUT_PADDING)
    }
}

impl Default for MockProverConfig {
    fn default() -> Self {
        Self {
            max_concurrency_limit: default_max_concurrency_limit(),
            proving_request_timeout: None,
            proving_timeout: default_cpu_proving_timeout(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct GrpcConfig {
    #[serde(
        skip_serializing_if = "same_as_default_max_decoding_message_size",
        default = "default_max_decoding_message_size"
    )]
    pub max_decoding_message_size: usize,
    #[serde(
        skip_serializing_if = "same_as_default_max_encoding_message_size",
        default = "default_max_encoding_message_size"
    )]
    pub max_encoding_message_size: usize,
}

impl Default for GrpcConfig {
    fn default() -> Self {
        Self {
            max_decoding_message_size: default_max_decoding_message_size(),
            max_encoding_message_size: default_max_encoding_message_size(),
        }
    }
}

#[derive(Default, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct ClientProverConfig {
    #[serde(default)]
    pub grpc: GrpcConfig,
}

const fn default_max_decoding_message_size() -> usize {
    4 * 1024 * 1024
}
fn same_as_default_max_decoding_message_size(value: &usize) -> bool {
    *value == default_max_decoding_message_size()
}
const fn default_max_encoding_message_size() -> usize {
    4 * 1024 * 1024
}
fn same_as_default_max_encoding_message_size(value: &usize) -> bool {
    *value == default_max_encoding_message_size()
}

const fn default_socket_addr() -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080)
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
