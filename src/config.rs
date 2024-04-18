//! Agglayer configuration.
//!
//! The agglayer is configured via its TOML configuration file, `agglayer.toml`
//! by default, which is deserialized into the [`Config`] struct.

use std::{collections::HashMap, net::Ipv4Addr, path::PathBuf};

use async_trait::async_trait;
use ethers::{
    signers::{LocalWallet, Signer, WalletError},
    types::{
        transaction::{eip2718::TypedTransaction, eip712::Eip712},
        Address, Signature,
    },
};
use ethers_gcp_kms_signer::{CKMSError, GcpKeyRingRef, GcpKmsProvider, GcpKmsSigner};
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
    KmsProvider(#[from] CKMSError),
    #[error("KMS configuration error: missing key or env {0}")]
    KmsConfig(String),
}

/// A an ethers [`Signer`] that can house either a local keystore or a GCP KMS
/// signer.
///
/// An ethers [`Provider`][ethers::prelude::Provider] using a
/// [`SignerMiddleware`][ethers::prelude::SignerMiddleware] must have its
/// [`Signer`] type specified at compile time, and the Signer type is not object
/// safe, so we cannot use a `Box<dyn Signer>`. As such, we define this enum to
/// accommodate a runtime configured signer.
#[derive(Debug)]
pub(crate) enum ConfiguredSigner {
    Local(LocalWallet),
    GcpKms(GcpKmsSigner),
}

/// Errors that can occur when using a [`ConfiguredSigner`].
///
/// This is simply a union of either a [`WalletError`] or a [`CKMSError`].
#[derive(Debug, Error)]
pub(crate) enum ConfiguredSignerError {
    #[error("wallet error: {0}")]
    Wallet(WalletError),
    #[error("KMS error: {0}")]
    Kms(CKMSError),
}

/// [`Signer`] implementation for [`ConfiguredSigner`].
///
/// This implementation simply delegates to the underlying signer.
#[async_trait]
impl Signer for ConfiguredSigner {
    type Error = ConfiguredSignerError;

    async fn sign_message<S: Send + Sync + AsRef<[u8]>>(
        &self,
        message: S,
    ) -> Result<Signature, Self::Error> {
        match self {
            ConfiguredSigner::Local(wallet) => wallet
                .sign_message(message)
                .await
                .map_err(ConfiguredSignerError::Wallet),
            ConfiguredSigner::GcpKms(signer) => signer
                .sign_message(message)
                .await
                .map_err(ConfiguredSignerError::Kms),
        }
    }

    /// Signs the transaction
    async fn sign_transaction(&self, message: &TypedTransaction) -> Result<Signature, Self::Error> {
        match self {
            ConfiguredSigner::Local(wallet) => wallet
                .sign_transaction(message)
                .await
                .map_err(ConfiguredSignerError::Wallet),
            ConfiguredSigner::GcpKms(signer) => signer
                .sign_transaction(message)
                .await
                .map_err(ConfiguredSignerError::Kms),
        }
    }

    /// Encodes and signs the typed data according EIP-712.
    /// Payload must implement Eip712 trait.
    async fn sign_typed_data<T: Eip712 + Send + Sync>(
        &self,
        payload: &T,
    ) -> Result<Signature, Self::Error> {
        match self {
            ConfiguredSigner::Local(wallet) => wallet
                .sign_typed_data(payload)
                .await
                .map_err(ConfiguredSignerError::Wallet),
            ConfiguredSigner::GcpKms(signer) => signer
                .sign_typed_data(payload)
                .await
                .map_err(ConfiguredSignerError::Kms),
        }
    }

    /// Returns the signer's Ethereum Address
    fn address(&self) -> Address {
        match self {
            ConfiguredSigner::Local(wallet) => wallet.address(),
            ConfiguredSigner::GcpKms(signer) => signer.address(),
        }
    }

    /// Returns the signer's chain id
    fn chain_id(&self) -> u64 {
        match self {
            ConfiguredSigner::Local(wallet) => wallet.chain_id(),
            ConfiguredSigner::GcpKms(signer) => signer.chain_id(),
        }
    }

    /// Sets the signer's chain id
    #[must_use]
    fn with_chain_id<T: Into<u64>>(self, chain_id: T) -> Self {
        match self {
            ConfiguredSigner::Local(wallet) => {
                ConfiguredSigner::Local(wallet.with_chain_id(chain_id))
            }
            ConfiguredSigner::GcpKms(signer) => {
                ConfiguredSigner::GcpKms(signer.with_chain_id(chain_id))
            }
        }
    }
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
