//! Agglayer configuration.
//!
//! The agglayer is configured via its TOML configuration file, `agglayer.toml`
//! by default, which is deserialized into the [`Config`] struct.

use std::collections::HashMap;

use ethers::signers::{LocalWallet, Signer};
use ethers_gcp_kms_signer::{GcpKeyRingRef, GcpKmsProvider, GcpKmsSigner};
use serde::Deserialize;
use tracing::debug;
use url::Url;

use self::{
    eth_tx_manager::PrivateKey, rpc::deserialize_rpc_map, signer::ConfiguredSigner,
    telemetry::TelemetryConfig,
};

pub(crate) const DEFAULT_IP: std::net::Ipv4Addr = std::net::Ipv4Addr::new(0, 0, 0, 0);

pub(crate) mod error;
pub(crate) mod eth_tx_manager;
pub(crate) mod l1;
pub(crate) mod log;
pub(crate) mod rpc;
pub(crate) mod signer;
pub(crate) mod telemetry;

pub(crate) use error::ConfigError;
pub(crate) use eth_tx_manager::EthTxManager;
pub(crate) use l1::L1;
pub(crate) use log::Log;
pub(crate) use rpc::RpcConfig;

/// The Agglayer configuration.
#[derive(Deserialize, Debug)]
pub(crate) struct Config {
    /// A map of Zkevm node RPC endpoints for each rollup.
    ///
    /// The key is the rollup ID, and the value is the URL of the associated RPC
    /// endpoint.
    #[serde(rename = "FullNodeRPCs", deserialize_with = "deserialize_rpc_map")]
    pub(crate) full_node_rpcs: HashMap<u32, Url>,
    /// The log configuration.
    #[serde(rename = "Log")]
    pub(crate) log: Log,
    /// The local gRPC server configuration.
    #[serde(rename = "RPC")]
    pub(crate) grpc: RpcConfig,
    /// The L1 configuration.
    #[serde(rename = "L1")]
    pub(crate) l1: L1,
    /// The transaction management configuration.
    #[serde(rename = "EthTxManager")]
    pub(crate) eth_tx_manager: EthTxManager,

    /// Telemetry configuration.
    #[serde(rename = "Telemetry")]
    pub(crate) telemetry: TelemetryConfig,
}

impl Config {
    /// Get the port from the environment variable, or the configuration file.
    ///
    /// If the `PORT` environment variable is set, it will take precedence over
    /// the configuration file.
    pub(crate) fn port(&self) -> u16 {
        std::env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(self.grpc.port)
    }

    /// Get the target gRPC socket address from the configuration.
    pub(crate) fn grpc_addr(&self) -> std::net::SocketAddr {
        std::net::SocketAddr::from((self.grpc.host, self.port()))
    }

    /// Get the first local private key specified in the configuration.
    fn local_pk(&self) -> Result<&PrivateKey, ConfigError> {
        self.eth_tx_manager
            .private_keys
            .first()
            .ok_or(ConfigError::NoPk)
    }

    /// Decrypt the first local keystore specified in the configuration.
    pub(crate) fn local_wallet(&self) -> Result<LocalWallet, ConfigError> {
        let pk = self.local_pk()?;
        Ok(LocalWallet::decrypt_keystore(&pk.path, &pk.password)?.with_chain_id(self.l1.chain_id))
    }

    /// Create a GCP KMS signer from the configuration.
    ///
    /// This will first attempt to use the environment variables, and if they
    /// are not set, it will fall back to the values specified configuration
    /// file.
    ///
    /// The `ethers_gcp_kms_signer` library will attempt to load credentials in
    /// the typical fashion for GCP:
    /// - If the application is running in a k8s cluster, it should
    ///   automatically pick up credentials.
    /// - If the `GOOGLE_APPLICATION_CREDENTIALS` environment is set, attempt to
    ///   load a service account JSON from this path.
    pub(crate) async fn gcp_kms_signer(&self) -> Result<GcpKmsSigner, ConfigError> {
        let project_id = std::env::var("GOOGLE_PROJECT_ID").or_else(|_| {
            self.eth_tx_manager
                .kms_project_id
                .clone()
                .ok_or(ConfigError::KmsConfig("GOOGLE_PROJECT_ID".to_string()))
        })?;
        let location = std::env::var("GOOGLE_LOCATION").or_else(|_| {
            self.eth_tx_manager
                .kms_location
                .clone()
                .ok_or(ConfigError::KmsConfig("GOOGLE_LOCATION".to_string()))
        })?;
        let keyring = std::env::var("GOOGLE_KEYRING").or_else(|_| {
            self.eth_tx_manager
                .kms_keyring
                .clone()
                .ok_or(ConfigError::KmsConfig("GOOGLE_KEYRING".to_string()))
        })?;
        let key_name = std::env::var("GOOGLE_KEY_NAME").or_else(|_| {
            self.eth_tx_manager
                .kms_key_name
                .clone()
                .ok_or(ConfigError::KmsConfig("GOOGLE_KEY_NAME".to_string()))
        })?;

        let keyring = GcpKeyRingRef::new(&project_id, &location, &keyring);
        let provider = GcpKmsProvider::new(keyring).await?;
        Ok(GcpKmsSigner::new(provider, key_name.to_string(), 1, self.l1.chain_id).await?)
    }

    /// Get either a local wallet or GCP KMS signer based on the configuration.
    ///
    /// The logic here that determines which signer to use is as follows:
    /// 1. If a GCP KMS key name is specified, attempt to use the GCP KMS
    ///    signer.
    /// 2. Otherwise, attempt use the local wallet.
    ///
    /// This logic is ported directly from the original agglayer Go codebase.
    pub(crate) async fn get_configured_signer(&self) -> Result<ConfiguredSigner, ConfigError> {
        if self.eth_tx_manager.kms_key_name.is_some() {
            debug!("Using GCP KMS signer");
            Ok(ConfiguredSigner::GcpKms(self.gcp_kms_signer().await?))
        } else {
            debug!("Using local wallet signer");
            Ok(ConfiguredSigner::Local(self.local_wallet()?))
        }
    }

    /// Set the `RUST_LOG` environment variable relative to the configuration,
    /// if not already set.
    ///
    /// The `RUST_LOG` environment variable will take precedence over the
    /// configuration file.
    pub(crate) fn set_log_env(&self) {
        if std::env::var("RUST_LOG").is_err() {
            std::env::set_var("RUST_LOG", self.log.level.as_str());
        }
    }
}
