//! Agglayer configuration.
//!
//! The agglayer is configured via its TOML configuration file, `agglayer.toml`
//! by default, which is deserialized into the [`Config`] struct.

use agglayer_config::KmsConfig;
use ethers_gcp_kms_signer::{GcpKeyRingRef, GcpKmsProvider, GcpKmsSigner};
use serde::Deserialize;

pub(crate) mod error;
pub(crate) mod signer;

pub use error::KmsError;
pub use signer::KmsSigner;

/// The Agglayer configuration.
#[derive(Deserialize, Debug)]
#[cfg_attr(any(test, feature = "testutils"), derive(Default))]
pub struct KMS {
    /// The L1 chain id.
    pub chain_id: u64,
    /// The kms configuration.
    pub config: KmsConfig,
}

impl KMS {
    pub fn new(chain_id: u64, config: KmsConfig) -> Self {
        Self { chain_id, config }
    }
    /// Create a GCP KMS signer from the configuration.
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
    pub async fn gcp_kms_signer(&self) -> Result<KmsSigner, KmsError> {
        let project_id = std::env::var("GOOGLE_PROJECT_ID").or_else(|_| {
            self.config
                .project_id
                .clone()
                .ok_or(KmsError::KmsConfig("GOOGLE_PROJECT_ID".to_string()))
        })?;
        let location = std::env::var("GOOGLE_LOCATION").or_else(|_| {
            self.config
                .location
                .clone()
                .ok_or(KmsError::KmsConfig("GOOGLE_LOCATION".to_string()))
        })?;
        let keyring = std::env::var("GOOGLE_KEYRING").or_else(|_| {
            self.config
                .keyring
                .clone()
                .ok_or(KmsError::KmsConfig("GOOGLE_KEYRING".to_string()))
        })?;
        let key_name = std::env::var("GOOGLE_KEY_NAME").or_else(|_| {
            self.config
                .key_name
                .clone()
                .ok_or(KmsError::KmsConfig("GOOGLE_KEY_NAME".to_string()))
        })?;

        let keyring = GcpKeyRingRef::new(&project_id, &location, &keyring);
        let provider = GcpKmsProvider::new(keyring).await?;
        let gcp_signer =
            GcpKmsSigner::new(provider, key_name.to_string(), 1, self.chain_id).await?;
        Ok(KmsSigner::new(gcp_signer))
    }
}
