use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, NoneAsEmptyString};

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
#[cfg_attr(any(test, feature = "testutils"), derive(Default))]
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
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(any(test, feature = "testutils"), derive(Default))]
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

    // To be deprecated in favor of the above distinct keys
    // ------------------------------------------------------------
    #[serde(alias = "KeyName")]
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default)]
    pub key_name: Option<String>,
    #[serde(alias = "KeyVersion")]
    #[serde(default)]
    pub key_version: Option<u64>,
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
