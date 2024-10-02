use std::collections::HashMap;

use ethers::types::Address;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    outbound::OutboundConfig, shutdown::ShutdownConfig, telemetry::TelemetryConfig, AuthConfig,
    Epoch, Log, RateLimitingConfig, RpcConfig, L1,
};

/// The v0.1 Agglayer configuration.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    /// A map of Zkevm node RPC endpoints for each rollup.
    ///
    /// The key is the rollup ID, and the value is the URL of the associated RPC
    /// endpoint.
    #[serde(
        alias = "FullNodeRPCs",
        deserialize_with = "crate::deserialize_rpc_map"
    )]
    pub full_node_rpcs: HashMap<u32, Url>,
    #[serde(
        alias = "ProofSigners",
        deserialize_with = "crate::deserialize_signers_map",
        default
    )]
    pub proof_signers: HashMap<u32, Address>,
    /// The log configuration.
    #[serde(alias = "Log")]
    pub log: Log,
    /// The local RPC server configuration.
    #[serde(alias = "RPC")]
    pub rpc: RpcConfig,
    /// Rate limiting configuration.
    #[serde(default)]
    pub rate_limiting: RateLimitingConfig,
    /// The configuration for every outbound network component.
    #[serde(default)]
    pub outbound: OutboundConfig,
    /// The L1 configuration.
    #[serde(alias = "L1")]
    pub l1: L1,
    /// The authentication configuration.
    #[serde(
        alias = "EthTxManager",
        default,
        deserialize_with = "crate::deserialize_auth"
    )]
    pub auth: AuthConfig,
    /// Telemetry configuration.
    #[serde(alias = "Telemetry")]
    pub telemetry: TelemetryConfig,

    /// The Epoch configuration.
    #[serde(alias = "Epoch", default = "crate::Epoch::default")]
    pub epoch: Epoch,

    /// The list of configuration options used during shutdown.
    #[serde(default)]
    pub shutdown: ShutdownConfig,
}
