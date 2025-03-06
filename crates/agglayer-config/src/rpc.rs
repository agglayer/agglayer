use std::{net::Ipv4Addr, str::FromStr, time::Duration};

use jsonrpsee::core::TEN_MB_SIZE_BYTES;
use serde::{Deserialize, Deserializer, Serialize};
use serde_with::serde_as;

/// The default port for the local gRPC server.
const DEFAULT_GRPC_PORT: u16 = 9089;

/// The default port for the local ReadRPC server.
const DEFAULT_READRPC_PORT: u16 = 9090;

/// The default port for the local admin RPC server.
const DEFAULT_ADMIN_PORT: u16 = 9091;

/// The local RPC server configuration.
#[serde_as]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct RpcConfig {
    /// The default port for the local gRPC server.
    /// Overridden by `AGGLAYER_GRPC_PORT` environment variable, defaults to
    /// 9089.
    #[serde(
        default = "default_grpc_port",
        deserialize_with = "deserialize_grpc_port"
    )]
    pub grpc_port: u16,

    /// The default port for the local ReadRPC server.
    /// Overridden by `AGGLAYER_READRPC_PORT` environment variable, defaults to
    /// 9090.
    #[serde(
        default = "default_readrpc_port",
        deserialize_with = "deserialize_readrpc_port"
    )]
    pub readrpc_port: u16,

    /// The default port for the local AdminRPC server.
    /// Overridden by `AGGLAYER_ADMIN_PORT` environment variable, defaults to
    /// 9091.
    #[serde(
        default = "default_admin_port",
        deserialize_with = "deserialize_admin_port"
    )]
    pub admin_port: u16,

    #[serde(default = "default_host")]
    pub host: Ipv4Addr,

    /// The maximum size of the request body in bytes.
    #[serde(
        skip_serializing_if = "same_as_default_body_size",
        default = "default_body_size"
    )]
    pub max_request_body_size: u32,
    /// The maximum size of the response body in bytes.
    #[serde(
        skip_serializing_if = "same_as_default_body_size",
        default = "default_body_size"
    )]
    pub max_response_body_size: u32,
    /// The maximum number of connections.
    #[serde(
        skip_serializing_if = "same_as_default_max_connections",
        default = "default_max_connections"
    )]
    pub max_connections: u32,
    /// The maximum number of requests in a batch request. If `None`, the
    /// batch request limit is unlimited.
    #[serde(skip_serializing_if = "crate::is_default")]
    pub batch_request_limit: Option<u32>,
    /// The interval at which to send ping messages
    #[serde(skip)]
    #[serde_as(as = "Option<crate::with::HumanDuration>")]
    pub ping_interval: Option<Duration>,
    /// Timeout for completion of an RPC request to the AggLayer node.
    #[serde_as(as = "crate::with::HumanDuration")]
    #[serde(default = "default_request_timeout")]
    pub request_timeout: Duration,
}

impl Default for RpcConfig {
    fn default() -> Self {
        Self {
            grpc_port: default_grpc_port(),
            readrpc_port: default_readrpc_port(),
            admin_port: default_admin_port(),
            host: default_host(),
            max_request_body_size: default_body_size(),
            max_response_body_size: default_body_size(),
            max_connections: default_max_connections(),
            batch_request_limit: None,
            ping_interval: None,
            request_timeout: default_request_timeout(),
        }
    }
}

/// The default maximum number of connections.
fn default_max_connections() -> u32 {
    100
}

/// The default size of the request and response bodies in bytes.
fn default_body_size() -> u32 {
    TEN_MB_SIZE_BYTES
}

fn default_grpc_port() -> u16 {
    from_env_or_default("AGGLAYER_GRPC_PORT", DEFAULT_GRPC_PORT)
}

fn default_readrpc_port() -> u16 {
    from_env_or_default("AGGLAYER_READRPC_PORT", DEFAULT_READRPC_PORT)
}

fn default_admin_port() -> u16 {
    from_env_or_default("AGGLAYER_ADMIN_PORT", DEFAULT_ADMIN_PORT)
}

/// The default host for the local RPC server.
const fn default_host() -> Ipv4Addr {
    Ipv4Addr::new(0, 0, 0, 0)
}

/// Default timeout for completion of an RPC request to the AggLayer node.
const fn default_request_timeout() -> Duration {
    Duration::from_secs(180)
}

fn deserialize_grpc_port<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: Deserializer<'de>,
{
    let port = u16::deserialize(deserializer)?;

    Ok(from_env_or_default("AGGLAYER_GRPC_PORT", port))
}

fn deserialize_readrpc_port<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: Deserializer<'de>,
{
    let port = u16::deserialize(deserializer)?;

    Ok(from_env_or_default("AGGLAYER_READRPC_PORT", port))
}

fn deserialize_admin_port<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: Deserializer<'de>,
{
    let port = u16::deserialize(deserializer)?;

    Ok(from_env_or_default("AGGLAYER_ADMIN_PORT", port))
}

/// Get an environment variable or a default value if it is not set.
fn from_env_or_default<T: FromStr>(key: &str, default: T) -> T {
    std::env::var(key)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

fn same_as_default_body_size(size: &u32) -> bool {
    *size == default_body_size()
}

fn same_as_default_max_connections(max_connections: &u32) -> bool {
    *max_connections == default_max_connections()
}
