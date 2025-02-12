//! Agglayer configuration.
//!
//! The agglayer is configured via its TOML configuration file, `agglayer.toml`
//! by default, which is deserialized into the [`Config`] struct.

use std::{collections::HashMap, path::Path};

use ethers::types::Address;
use outbound::OutboundConfig;
use serde::{de::DeserializeSeed, Deserialize, Serialize};
use serde_with::DisplayFromStr;
use shutdown::ShutdownConfig;
use url::Url;

pub use self::telemetry::TelemetryConfig;

pub mod prover;

pub(crate) const DEFAULT_IP: std::net::Ipv4Addr = std::net::Ipv4Addr::new(0, 0, 0, 0);

pub(crate) mod auth;
pub mod certificate_orchestrator;
pub mod epoch;
pub(crate) mod l1;
pub(crate) mod l2;
pub mod log;
pub mod outbound;
pub mod rate_limiting;
pub(crate) mod rpc;
pub mod shutdown;
pub mod storage;
pub(crate) mod telemetry;
mod with;

pub use auth::{AuthConfig, GcpKmsConfig, LocalConfig, PrivateKey};
pub use epoch::Epoch;
pub use l1::L1;
pub use l2::L2;
pub use log::Log;
use prover::default_prover_entrypoint;
pub use rate_limiting::RateLimitingConfig;
pub use rpc::RpcConfig;

/// The Agglayer configuration.
#[serde_with::serde_as]
#[derive(Default, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct Config {
    /// A map of Zkevm node RPC endpoints for each rollup.
    ///
    /// The key is the rollup ID, and the value is the URL of the associated RPC
    /// endpoint.
    #[serde_as(as = "HashMap<DisplayFromStr, _>")]
    pub full_node_rpcs: HashMap<u32, Url>,

    #[serde(default)]
    pub l2: L2,

    #[serde_as(as = "HashMap<DisplayFromStr, _>")]
    #[serde(default)]
    pub proof_signers: HashMap<u32, Address>,
    /// The log configuration.
    #[serde(default)]
    pub log: Log,
    /// The local RPC server configuration.
    #[serde(default)]
    pub rpc: RpcConfig,
    /// Rate limiting configuration.
    #[serde(default)]
    pub rate_limiting: RateLimitingConfig,
    /// The configuration for every outbound network component.
    #[serde(default)]
    pub outbound: OutboundConfig,
    /// The L1 configuration.
    #[serde(default)]
    pub l1: L1,
    /// The authentication configuration.
    #[serde(default)]
    pub auth: AuthConfig,

    /// Telemetry configuration.
    #[serde(default)]
    pub telemetry: TelemetryConfig,

    /// The Epoch configuration.
    #[serde(default)]
    pub epoch: Epoch,

    /// The list of configuration options used during shutdown.
    #[serde(default)]
    pub shutdown: ShutdownConfig,

    /// The certificate orchestrator configuration.
    #[serde(default)]
    pub certificate_orchestrator: certificate_orchestrator::CertificateOrchestrator,

    /// The storage configuration.
    #[serde(default)]
    pub storage: storage::StorageConfig,

    /// AggLayer prover entrypoint.
    #[serde(default = "default_prover_entrypoint")]
    #[serde(skip_serializing_if = "String::is_empty")]
    pub prover_entrypoint: String,

    #[serde(default, skip_serializing_if = "crate::default")]
    pub prover: agglayer_prover_config::ClientProverConfig,

    #[serde(default)]
    #[serde(skip_serializing_if = "is_false")]
    pub debug_mode: bool,

    #[serde(default)]
    #[serde(skip_serializing_if = "is_false")]
    pub mock_verifier: bool,
}

impl Config {
    pub fn try_load(path: &Path) -> Result<Self, ConfigurationError> {
        let reader = std::fs::read_to_string(path).map_err(|source| {
            ConfigurationError::UnableToReadConfigFile {
                path: path.to_path_buf(),
                source,
            }
        })?;

        let path = path
            .parent()
            .ok_or_else(|| ConfigurationError::UnableToReadConfigFile {
                path: path.to_path_buf(),
                source: std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Unable to determine the parent folder of the configuration file",
                ),
            })?;

        let config_with_path = ConfigDeserializer { path };

        let deserializer = toml::de::Deserializer::new(&reader);

        config_with_path
            .deserialize(deserializer)
            .map_err(ConfigurationError::DeserializationError)
    }
}

impl Config {
    pub fn new(base_path: &Path) -> Self {
        Self {
            storage: storage::StorageConfig::new_from_path(base_path),
            full_node_rpcs: Default::default(),
            proof_signers: Default::default(),
            log: Default::default(),
            rpc: Default::default(),
            rate_limiting: Default::default(),
            outbound: Default::default(),
            l1: Default::default(),
            l2: Default::default(),
            auth: Default::default(),
            telemetry: Default::default(),
            epoch: Default::default(),
            shutdown: Default::default(),
            certificate_orchestrator: Default::default(),
            prover_entrypoint: default_prover_entrypoint(),
            prover: Default::default(),
            debug_mode: false,
            mock_verifier: false,
        }
    }

    /// Get the target RPC socket address from the configuration.
    pub fn rpc_addr(&self) -> std::net::SocketAddr {
        std::net::SocketAddr::from((self.rpc.host, self.rpc.port))
    }

    pub fn path_contextualized(mut self, base_path: &Path) -> Self {
        self.storage = self.storage.path_contextualized(base_path);

        self
    }

    pub(crate) fn validate(self) -> Result<Self, ConfigurationError> {
        Ok(self)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigurationError {
    #[error("Unable to read the configuration file: {source}")]
    UnableToReadConfigFile {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to deserialize the configuration: {0}")]
    DeserializationError(#[from] toml::de::Error),
}

#[cfg(any(test, feature = "testutils"))]
impl Config {
    pub fn new_for_test() -> Self {
        Config::new(Path::new("/tmp/agglayer"))
    }
}

struct ConfigDeserializer<'a> {
    path: &'a Path,
}

impl<'de> DeserializeSeed<'de> for ConfigDeserializer<'_> {
    type Value = Config;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut config_candidate: Config = serde::Deserialize::deserialize(deserializer)?;

        config_candidate.storage =
            config_candidate
                .storage
                .path_contextualized(&self.path.canonicalize().map_err(|error| {
                    serde::de::Error::custom(format!(
                        "Unable to canonicalize the storage path: {}",
                        error
                    ))
                })?);

        config_candidate
            .validate()
            .map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}

fn is_false(b: &bool) -> bool {
    !*b
}

pub(crate) fn default<T: Default + PartialEq>(t: &T) -> bool {
    *t == Default::default()
}
