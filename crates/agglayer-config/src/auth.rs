use std::path::PathBuf;

use serde::de::{self, Deserializer};
use serde::Deserialize;
use serde_with::{serde_as, NoneAsEmptyString};

/// The transaction management configuration.
///
/// Generally allows specification of transaction signing behavior.
///
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
#[derive(Deserialize, Debug)]
#[serde(untagged, rename_all = "lowercase")]
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
#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "PascalCase")]
pub struct LocalConfig {
    pub private_keys: Vec<PrivateKey>,
}

#[derive(Deserialize, Debug)]
#[cfg_attr(any(test, feature = "testutils"), derive(Default))]
#[serde(rename_all = "PascalCase")]
pub struct PrivateKey {
    pub path: PathBuf,
    pub password: String,
}

/// GCP KMS configuration.
///
/// It includes kms config.
#[serde_as]
#[derive(Deserialize, Debug, Clone)]
#[cfg_attr(any(test, feature = "testutils"), derive(Default))]
#[serde(rename_all = "PascalCase")]
pub struct GcpKmsConfig {
    #[serde(rename = "ProjectId")]
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(rename = "Location")]
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default)]
    pub location: Option<String>,
    #[serde(rename = "Keyring")]
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default)]
    pub keyring: Option<String>,
    #[serde(rename = "KeyName")]
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default)]
    pub key_name: Option<String>,
    #[serde(rename = "KeyVersion")]
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
    #[serde(default, rename = "PrivateKeys")]
    private_keys: Option<Vec<PrivateKey>>,
    #[serde(flatten)]
    kms: Option<GcpKmsConfig>,
}

pub(crate) fn deserialize_auth<'de, D>(deserializer: D) -> Result<AuthConfig, D::Error>
where
    D: Deserializer<'de>,
{
    let intermediate = IntermediateAuthConfig::deserialize(deserializer)?;

    if let Some(local) = intermediate.local {
        Ok(AuthConfig::Local(local))
    } else if let Some(gcpkms) = intermediate.gcpkms {
        Ok(AuthConfig::GcpKms(gcpkms))
    } else if let Some(private_keys) = intermediate.private_keys {
        Ok(AuthConfig::Local(LocalConfig { private_keys }))
    } else if let Some(kms) = intermediate.kms {
        Ok(AuthConfig::GcpKms(kms))
    } else {
        Err(de::Error::custom("Invalid auth configuration"))
    }
}
