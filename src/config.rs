//! Agglayer configuration.
//!
//! The agglayer is configured via its TOML configuration file, `agglayer.toml`
//! by default, which is deserialized into the [`Config`] struct.

use std::{collections::HashMap, net::Ipv4Addr, path::PathBuf};

use alloy::network::EthereumSigner;
use alloy::primitives::Address;
use alloy::signers::gcp::{GcpKeyRingRef, GcpSigner, KeySpecifier};
use alloy::signers::wallet::{LocalWallet, Wallet, WalletError};
use gcloud_sdk::{
    google::cloud::kms::v1::key_management_service_client::KeyManagementServiceClient, GoogleApi,
};
use serde::{
    de::{MapAccess, Visitor},
    Deserialize, Deserializer,
};
use serde_with::{serde_as, NoneAsEmptyString};
use thiserror::Error;
use tracing::debug;
use tracing_subscriber::fmt::writer::BoxMakeWriter;
use url::Url;

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
}

/// The log level.
#[derive(Deserialize, Debug, Default, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub(crate) enum LogLevel {
    Trace,
    Debug,
    #[default]
    Info,
    Warn,
    Error,
    Fatal,
}

impl LogLevel {
    /// Get the log level as a string.
    ///
    /// This is used to set the `RUST_LOG` environment variable.
    pub(crate) fn as_str(&self) -> &str {
        match self {
            LogLevel::Trace => "trace",
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
            LogLevel::Fatal => "fatal",
        }
    }
}

/// The log output.
///
/// This can be either `stdout`, `stderr`, or a file path.
///
/// The [`Deserialize`] implementation allows for the configuration file to
/// specify the output location as a string, which is then parsed into the
/// appropriate enum variant. If the string is not recognized to be either
/// `stdout` or `stderr`, it is assumed to be a file path.
#[derive(Debug, Clone, Default)]
pub(crate) enum LogOutput {
    #[default]
    Stdout,
    Stderr,
    File(PathBuf),
}

impl<'de> Deserialize<'de> for LogOutput {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        // If the string is not recognized to be either `stdout` or `stderr`,
        // it is assumed to be a file path.
        match s.as_str() {
            "stdout" => Ok(LogOutput::Stdout),
            "stderr" => Ok(LogOutput::Stderr),
            _ => Ok(LogOutput::File(PathBuf::from(s))),
        }
    }
}

impl LogOutput {
    /// Get a [`BoxMakeWriter`] for the log output.
    ///
    /// This can be used to plug the log output into the tracing subscriber.
    pub(crate) fn as_make_writer(&self) -> BoxMakeWriter {
        match self {
            LogOutput::Stdout => BoxMakeWriter::new(std::io::stdout),
            LogOutput::Stderr => BoxMakeWriter::new(std::io::stderr),
            LogOutput::File(path) => {
                let appender = tracing_appender::rolling::never(".", path);
                BoxMakeWriter::new(appender)
            }
        }
    }
}

/// The log configuration.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct Log {
    #[serde(default)]
    pub(crate) level: LogLevel,
    pub(crate) outputs: Vec<LogOutput>,
}

#[derive(Debug, Error)]
pub(crate) enum ConfigError {
    #[error("no private keys specified in the configuration")]
    NoPk,
    #[error("keystore error: {0}")]
    Wallet(#[from] WalletError),
    #[error("KMS Provider error: {0}")]
    KmsProvider(#[from] gcloud_sdk::error::Error),
    #[error("KMS configuration error: missing key or env {0}")]
    KmsConfig(String),
}

impl Config {
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
        Ok(Wallet::decrypt_keystore(&pk.path, &pk.password)?)
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
    pub(crate) async fn gcp_kms_signer(&self) -> Result<GcpSigner, ConfigError> {
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
        let keyring_name = std::env::var("GOOGLE_KEYRING").or_else(|_| {
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
        let key_version: u64 = std::env::var("GOOGLE_KEY_VERSION").map_or_else(
            |_| {
                self.eth_tx_manager
                    .kms_key_version
                    .ok_or(ConfigError::KmsConfig("GOOGLE_KEY_VERSION".to_string()))
            },
            |v| {
                v.parse().map_err(|_| {
                    ConfigError::KmsConfig("Unable to parse GOOGLE_KEY_VERSION to u64".to_string())
                })
            },
        )?;

        let google_cloud_api: String = std::env::var("GOOGLE_CLOUD_API")
            .or_else(|_| Ok::<_, ConfigError>(self.eth_tx_manager.google_cloud_api.clone()))?;

        // Create a GCP KMS keyring reference base on the configured values.
        let keyring = GcpKeyRingRef::new(&project_id, &location, &keyring_name);

        // Create a GCP KMS client (without prefix metadata)
        let client =
            GoogleApi::from_function(KeyManagementServiceClient::new, google_cloud_api, None)
                .await?;

        // Target the right key in the keyring by specifying the key name and version.
        let key_specifier = KeySpecifier::new(keyring, &key_name, key_version);

        // Create the GcpSigner using both the client and the key specifier.
        GcpSigner::new(client, key_specifier, Some(self.l1.chain_id))
            .await
            .map_err(|_| ConfigError::KmsConfig("Unable to create GcpSigner".to_string()))
    }

    /// Get either a local wallet or GCP KMS signer based on the configuration.
    ///
    /// The logic here that determines which signer to use is as follows:
    /// 1. If a GCP KMS key name is specified, attempt to use the GCP KMS
    ///    signer.
    /// 2. Otherwise, attempt use the local wallet.
    ///
    /// This logic is ported directly from the original agglayer Go codebase.
    pub(crate) async fn get_configured_signer(&self) -> Result<EthereumSigner, ConfigError> {
        if self.eth_tx_manager.kms_key_name.is_some() {
            debug!("Using GCP KMS signer");
            Ok(EthereumSigner::from(self.gcp_kms_signer().await?))
        } else {
            debug!("Using local wallet signer");
            let signer = self.local_wallet()?;
            Ok(EthereumSigner::from(signer))
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

/// The local gRPC server configuration.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct RpcConfig {
    pub(crate) port: u16,
    pub(crate) host: Ipv4Addr,
}

/// The L1 configuration.
#[derive(Deserialize, Debug)]
pub(crate) struct L1 {
    #[serde(rename = "ChainID")]
    #[allow(dead_code)]
    pub(crate) chain_id: u64,
    #[serde(rename = "NodeURL")]
    pub(crate) node_url: Url,
    #[serde(rename = "RollupManagerContract")]
    pub(crate) rollup_manager_contract: Address,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct PrivateKey {
    pub(crate) path: PathBuf,
    pub(crate) password: String,
}

/// The transaction management configuration.
///
/// Generally allows specification of transaction signing behavior.
///
/// If a KMS key name is specified, the program will attempt to use a GCP KMS
/// signer. Otherwise, the program will attempt to use a local keystore signer.
/// The program will first attempt to populate the KMS specific configuration
/// values from the canonical environment variables, and if they are not set, it
/// will fall back to the values specified configuration file.
///
/// The `ethers_gcp_kms_signer` library will attempt to load credentials in
/// the typical fashion for GCP:
/// - If the application is running in a k8s cluster, it should automatically
///   pick up credentials.
/// - If the `GOOGLE_APPLICATION_CREDENTIALS` environment is set, attempt to
///   load a service account JSON from this path.
#[serde_as]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct EthTxManager {
    pub(crate) private_keys: Vec<PrivateKey>,
    #[serde(rename = "KMSProjectId")]
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default)]
    pub(crate) kms_project_id: Option<String>,
    #[serde(rename = "KMSLocation")]
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default)]
    pub(crate) kms_location: Option<String>,
    #[serde(rename = "KMSKeyring")]
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default)]
    pub(crate) kms_keyring: Option<String>,
    #[serde(rename = "KMSKeyName")]
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default)]
    pub(crate) kms_key_name: Option<String>,
    #[serde(rename = "KMSKeyVersion")]
    pub(crate) kms_key_version: Option<u64>,
    #[serde(rename = "KMSGoogleCloudAPI", default = "default_google_cloud_api")]
    pub(crate) google_cloud_api: String,
}

/// The default Google Cloud KMS API entrypoint.
fn default_google_cloud_api() -> String {
    "https://cloudkms.googleapis.com".to_string()
}

/// Deserialize a map of RPCs from a TOML file, where the keys are integers and
/// the values are URLs.
fn deserialize_rpc_map<'de, D>(deserializer: D) -> Result<HashMap<u32, Url>, D::Error>
where
    D: Deserializer<'de>,
{
    struct RpcMapVisitor;

    impl<'de> Visitor<'de> for RpcMapVisitor {
        type Value = HashMap<u32, Url>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a map of string keys to string values")
        }

        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut map = HashMap::with_capacity(access.size_hint().unwrap_or(0));
            while let Some((key, value)) = access.next_entry::<String, String>()? {
                let key: u32 = key.parse().map_err(serde::de::Error::custom)?;
                let value = Url::parse(&value).map_err(serde::de::Error::custom)?;
                map.insert(key, value);
            }
            Ok(map)
        }
    }

    deserializer.deserialize_map(RpcMapVisitor)
}
