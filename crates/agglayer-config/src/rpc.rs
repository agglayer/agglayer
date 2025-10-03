use std::{net::Ipv4Addr, time::Duration};

use jsonrpsee::core::TEN_MB_SIZE_BYTES;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::{Port, PortDefaults};

pub enum GrpcService {}
impl PortDefaults for GrpcService {
    const DEFAULT: u16 = 9089;
    const ENV_VAR: Option<&str> = Some("AGGLAYER_GRPC_PORT");
}

pub enum ReadRpcService {}
impl PortDefaults for ReadRpcService {
    const DEFAULT: u16 = 9090;
    const ENV_VAR: Option<&str> = Some("AGGLAYER_READRPC_PORT");
}

pub enum AdminService {}
impl PortDefaults for AdminService {
    const DEFAULT: u16 = 9091;
    const ENV_VAR: Option<&str> = Some("AGGLAYER_ADMIN_PORT");
}

pub enum HealthService {}
impl PortDefaults for HealthService {
    const DEFAULT: u16 = 9092;
    const ENV_VAR: Option<&str> = Some("AGGLAYER_HEALTH_PORT");
}

/// The local RPC server configuration.
#[serde_as]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct RpcConfig {
    /// The default port for the local gRPC server.
    /// Overridden by `AGGLAYER_GRPC_PORT` environment variable, defaults to
    /// 9089.
    #[serde(default)]
    pub grpc_port: Port<GrpcService>,

    /// The default port for the local ReadRPC server.
    /// Overridden by `AGGLAYER_READRPC_PORT` environment variable, defaults to
    /// 9090.
    #[serde(default)]
    pub readrpc_port: Port<ReadRpcService>,

    /// The default port for the local AdminRPC server.
    /// Overridden by `AGGLAYER_ADMIN_PORT` environment variable, defaults to
    /// 9091.
    #[serde(default)]
    pub admin_port: Port<AdminService>,

    /// The default port for the local HealthRPC server.
    /// Overridden by `AGGLAYER_HEALTH_PORT` environment variable, defaults to
    /// 9092.
    #[serde(default)]
    pub health_port: Port<HealthService>,

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
            grpc_port: Default::default(),
            readrpc_port: Default::default(),
            admin_port: Default::default(),
            health_port: Default::default(),
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

/// The default host for the local RPC server.
const fn default_host() -> Ipv4Addr {
    Ipv4Addr::new(0, 0, 0, 0)
}

/// Default timeout for completion of an RPC request to the AggLayer node.
const fn default_request_timeout() -> Duration {
    Duration::from_secs(180)
}

fn same_as_default_body_size(size: &u32) -> bool {
    *size == default_body_size()
}

fn same_as_default_max_connections(max_connections: &u32) -> bool {
    *max_connections == default_max_connections()
}
