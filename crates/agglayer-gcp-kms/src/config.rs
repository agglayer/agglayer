use serde::Deserialize;
use serde_with::{serde_as, NoneAsEmptyString};

/// The transaction management configuration.
///
/// Generally allows specification of transaction signing behavior.
///
/// If a KMS Provider is gcp, the program will attempt to use a GCP KMS
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
#[derive(Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub struct KmsConfig {
    #[serde(rename = "Provider")]
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default)]
    pub provider: Option<String>,
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
}
