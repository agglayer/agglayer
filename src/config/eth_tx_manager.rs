use std::path::PathBuf;

use serde::Deserialize;
use serde_with::{serde_as, NoneAsEmptyString};

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

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct PrivateKey {
    pub(crate) path: PathBuf,
    pub(crate) password: String,
}
