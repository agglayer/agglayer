//! Agglayer configuration.
//!
//! The agglayer is configured via its TOML configuration file, `agglayer.toml`
//! by default, which is deserialized into the [`Config`] struct.

use std::collections::HashMap;

use agglayer_gcp_kms::{KmsConfig, KMS};
use agglayer_signer::ConfiguredSigner;
use ethers::signers::{LocalWallet, Signer};
use serde::Deserialize;
use tracing::debug;
use url::Url;

use self::{local::PrivateKey, rpc::deserialize_rpc_map, telemetry::TelemetryConfig};

pub(crate) const DEFAULT_IP: std::net::Ipv4Addr = std::net::Ipv4Addr::new(0, 0, 0, 0);

pub(crate) mod error;
pub(crate) mod l1;
pub(crate) mod local;
pub mod log;
pub(crate) mod rpc;
pub(crate) mod telemetry;

pub use error::ConfigError;
pub use l1::L1;
pub use local::Local;
pub use log::Log;
pub use rpc::RpcConfig;

/// The Agglayer configuration.
#[derive(Deserialize, Debug)]
#[cfg_attr(any(test, feature = "testutils"), derive(Default))]
pub struct Config {
    /// A map of Zkevm node RPC endpoints for each rollup.
    ///
    /// The key is the rollup ID, and the value is the URL of the associated RPC
    /// endpoint.
    #[serde(rename = "FullNodeRPCs", deserialize_with = "deserialize_rpc_map")]
    pub full_node_rpcs: HashMap<u32, Url>,
    /// The log configuration.
    #[serde(rename = "Log")]
    pub log: Log,
    /// The local RPC server configuration.
    #[serde(rename = "RPC")]
    pub rpc: RpcConfig,
    /// The L1 configuration.
    #[serde(rename = "L1")]
    pub l1: L1,
    /// The local configuration.
    #[serde(rename = "Local")]
    pub local: Local,
    /// The kms configuration.
    #[serde(rename = "KMS")]
    pub kms: KmsConfig,
    /// Telemetry configuration.
    #[serde(rename = "Telemetry")]
    pub telemetry: TelemetryConfig,
}

impl Config {
    /// Get the target RPC socket address from the configuration.
    pub fn rpc_addr(&self) -> std::net::SocketAddr {
        std::net::SocketAddr::from((self.rpc.host, self.rpc.port))
    }

    /// Get the first local private key specified in the configuration.
    fn local_pk(&self) -> Result<&PrivateKey, ConfigError> {
        self.local.private_keys.first().ok_or(ConfigError::NoPk)
    }

    /// Decrypt the first local keystore specified in the configuration.
    pub(crate) fn local_wallet(&self) -> Result<LocalWallet, ConfigError> {
        let pk = self.local_pk()?;
        Ok(LocalWallet::decrypt_keystore(&pk.path, &pk.password)?.with_chain_id(self.l1.chain_id))
    }

    /// Get either a local wallet or GCP KMS signer based on the configuration.
    ///
    /// The logic here that determines which signer to use is as follows:
    /// 1. If a GCP KMS key name is specified, attempt to use the GCP KMS
    ///    signer.
    /// 2. Otherwise, attempt use the local wallet.
    ///
    /// This logic is ported directly from the original agglayer Go codebase.
    pub async fn get_configured_signer(&self) -> Result<ConfiguredSigner, ConfigError> {
        match self.kms.provider.as_deref() {
            Some("gcp") => {
                debug!("Using GCP KMS signer");
                let kms = KMS::new(self.l1.chain_id, self.kms.clone());
                Ok(ConfiguredSigner::Kms(kms.gcp_kms_signer().await?))
            }
            _ => {
                debug!("Using local wallet signer");
                Ok(ConfiguredSigner::Local(self.local_wallet()?))
            }
        }
    }
}
