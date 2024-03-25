//! Agglayer configuration.
//!
//! The agglayer is configured via its TOML configuration file, `agglayer.toml`
//! by default, which is deserialized into the [`Config`] struct.

use std::{collections::HashMap, net::Ipv4Addr, path::PathBuf};

use ethers::{
    signers::{LocalWallet, WalletError},
    types::Address,
};
use serde::{
    de::{MapAccess, Visitor},
    Deserialize, Deserializer,
};
use thiserror::Error;
use url::Url;

/// The Agglayer configuration.
#[derive(Deserialize, Debug)]
pub(crate) struct Config {
    /// A map of Zkevm node RPC endpoints for each rollup.
    ///
    /// The key is the rollup ID, and the value is the URL of the associated RPC
    /// endpoint.
    #[serde(rename = "FullNodeRPCs", deserialize_with = "deserialize_rpc_map")]
    #[allow(dead_code)]
    pub(crate) full_node_rpcs: HashMap<u32, Url>,
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

#[derive(Debug, Error)]
pub(crate) enum ConfigError {
    #[error("no private keys specified in the configuration")]
    NoPk,
    #[error("keystore error: {0}")]
    WalletError(#[from] WalletError),
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
        Ok(LocalWallet::decrypt_keystore(&pk.path, &pk.password)?)
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
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct EthTxManager {
    pub(crate) private_keys: Vec<PrivateKey>,
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
