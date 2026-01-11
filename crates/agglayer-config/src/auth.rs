use std::path::PathBuf;

use serde::{Deserialize, Deserializer, Serialize};
use serde_with::{serde_as, NoneAsEmptyString};
use tracing::warn;

/// The transaction management configuration.
///
/// Generally allows specification of transaction signing behavior.
///
/// The program will first attempt to populate the KMS specific configuration
/// values from the canonical environment variables, and if they are not set, it
/// will fall back to the values specified configuration file.
///
/// The `alloy-signer` library will attempt to load credentials in
/// the typical fashion for GCP:
/// - If the application is running in a k8s cluster, it should automatically
///   pick up credentials.
/// - If the `GOOGLE_APPLICATION_CREDENTIALS` environment is set, attempt to
///   load a service account JSON from this path.
#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
#[serde(try_from = "IntermediateAuthConfig")]
pub enum AuthConfig {
    Local(LocalConfig),
    GcpKms(GcpKmsConfig),
}

impl Default for AuthConfig {
    fn default() -> Self {
        AuthConfig::Local(LocalConfig::default())
    }
}

/// Local configuration.
///
/// It includes private keys for a local wallet.
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct LocalConfig {
    pub private_keys: Vec<PrivateKey>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "testutils", derive(Default))]
pub struct PrivateKey {
    #[serde(alias = "Path")]
    pub path: PathBuf,
    #[serde(alias = "Password")]
    pub password: String,
}

/// GCP KMS configuration.
///
/// It includes kms config.
#[serde_as]
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "testutils", derive(Default))]
#[serde(rename_all = "kebab-case")]
pub struct GcpKmsConfig {
    /// The GCP project ID to use.
    #[serde(alias = "ProjectId")]
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default)]
    pub project_id: Option<String>,

    /// The geographical region where the Cloud KMS resource is stored.
    #[serde(alias = "Location")]
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default)]
    pub location: Option<String>,

    /// The GCP KMS key ring to use.
    #[serde(alias = "Keyring")]
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default)]
    pub keyring: Option<String>,

    // Added to support distinct keys for cert and tx signing, falling back to
    // the older single key if not specified
    // ------------------------------------------------------------
    /// The key name for PP certificate settlement.
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default)]
    pub pp_settlement_key_name: Option<String>,

    /// The key version for PP certificate settlement.
    #[serde(default)]
    pub pp_settlement_key_version: Option<u64>,

    /// The key name for Tx certificate settlement.
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default)]
    pub tx_settlement_key_name: Option<String>,

    /// The key version for Tx certificate settlement.
    #[serde(default)]
    pub tx_settlement_key_version: Option<u64>,
}

impl<'de> Deserialize<'de> for GcpKmsConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[serde_as]
        #[derive(Deserialize)]
        #[cfg_attr(feature = "testutils", derive(Default))]
        #[serde(rename_all = "kebab-case")]
        pub struct Intermediate {
            #[serde(alias = "ProjectId")]
            #[serde_as(as = "NoneAsEmptyString")]
            #[serde(default)]
            pub project_id: Option<String>,
            #[serde(alias = "Location")]
            #[serde_as(as = "NoneAsEmptyString")]
            #[serde(default)]
            pub location: Option<String>,
            #[serde(alias = "Keyring")]
            #[serde_as(as = "NoneAsEmptyString")]
            #[serde(default)]
            pub keyring: Option<String>,
            #[serde_as(as = "NoneAsEmptyString")]
            #[serde(default)]
            pub pp_settlement_key_name: Option<String>,
            #[serde(default)]
            pub pp_settlement_key_version: Option<u64>,
            #[serde_as(as = "NoneAsEmptyString")]
            #[serde(default)]
            pub tx_settlement_key_name: Option<String>,
            #[serde(default)]
            pub tx_settlement_key_version: Option<u64>,
            #[serde(alias = "KeyName")]
            #[serde_as(as = "NoneAsEmptyString")]
            #[serde(default)]
            pub key_name: Option<String>,
            #[serde(alias = "KeyVersion")]
            #[serde(default)]
            pub key_version: Option<u64>,
        }

        let d = Intermediate::deserialize(deserializer)?;

        let (
            pp_settlement_key_name,
            pp_settlement_key_version,
            tx_settlement_key_name,
            tx_settlement_key_version,
        ) = match (
            d.pp_settlement_key_name,
            d.pp_settlement_key_version,
            d.tx_settlement_key_name,
            d.tx_settlement_key_version,
            d.key_name,
            d.key_version,
        ) {
            (Some(pp_k), Some(pp_v), Some(tx_k), Some(tx_v), _, _) => {
                (Some(pp_k), Some(pp_v), Some(tx_k), Some(tx_v))
            }
            (None, None, None, None, Some(k), Some(v)) => {
                warn!(
                    "'key-name' and 'key-version' are deprecated. Please use \
                     'pp-settlement-key-name','pp-settlement-key-version', \
                     'tx-settlement-key-name', and 'tx-settlement-key-version' instead."
                );
                (Some(k.clone()), Some(v), Some(k), Some(v))
            }
            _ => {
                return Err(serde::de::Error::custom(
                    "Either both \
                     ['pp-settlement-key-name','pp-settlement-key-version','\
                     tx-settlement-key-name','tx-settlement-key-version'] or 'key-name' and \
                     'key-version' must be specified",
                ));
            }
        };

        Ok(GcpKmsConfig {
            project_id: d.project_id,
            location: d.location,
            keyring: d.keyring,
            pp_settlement_key_name,
            pp_settlement_key_version,
            tx_settlement_key_name,
            tx_settlement_key_version,
        })
    }
}

// This is a workaround to support `EthTxManager` for PrivateKeys as it is used
// by kurtosis.
#[derive(Deserialize)]
struct IntermediateAuthConfig {
    #[serde(default)]
    local: Option<LocalConfig>,
    #[serde(default)]
    gcpkms: Option<GcpKmsConfig>,
    #[serde(default, alias = "PrivateKeys")]
    private_keys: Option<Vec<PrivateKey>>,
    #[serde(flatten)]
    kms: Option<GcpKmsConfig>,
}

impl TryFrom<IntermediateAuthConfig> for AuthConfig {
    type Error = &'static str;

    fn try_from(intermediate: IntermediateAuthConfig) -> Result<Self, Self::Error> {
        if let Some(local) = intermediate.local {
            Ok(AuthConfig::Local(local))
        } else if let Some(gcpkms) = intermediate.gcpkms {
            Ok(AuthConfig::GcpKms(gcpkms))
        } else if let Some(private_keys) = intermediate.private_keys {
            Ok(AuthConfig::Local(LocalConfig { private_keys }))
        } else if let Some(kms) = intermediate.kms {
            Ok(AuthConfig::GcpKms(kms))
        } else {
            Err("Invalid auth configuration")
        }
    }
}
