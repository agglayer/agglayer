//! The [`KMS`] struct provides functionality to create a GCP KMS signer from a
//! configuration. This struct is used to initialize and configure a Google
//! Cloud KMS signer.

use agglayer_config::GcpKmsConfig;
use alloy::signers::gcp::{GcpKeyRingRef, GcpSigner, KeySpecifier};
use gcloud_sdk::{
    google::cloud::kms::v1::key_management_service_client::KeyManagementServiceClient, GoogleApi,
};
use serde::Deserialize;

pub(crate) mod error;
pub(crate) mod signer;

pub use error::Error;
pub use signer::KmsSigner;

pub const GOOGLE_API_URL: &str = "https://cloudkms.googleapis.com";

#[derive(Deserialize, Debug)]
pub struct KMS {
    /// The L1 chain id.
    chain_id: u64,
    /// The GCP KMS configuration.
    config: GcpKmsConfig,
}

impl KMS {
    /// Creates a new KMS instance.
    pub fn new(chain_id: u64, config: GcpKmsConfig) -> Self {
        Self { chain_id, config }
    }

    /// Creates a GCP KMS signer from the configuration.
    ///
    /// This method will first attempt to use the environment variables, and if
    /// they are not set, it will fall back to the values specified in the
    /// configuration.
    ///
    /// The `alloy-signer-gcp` library will attempt to load credentials in
    /// the typical fashion for GCP:
    /// - If the application is running in a Kubernetes cluster, it should
    ///   automatically pick up credentials.
    /// - If the `GOOGLE_APPLICATION_CREDENTIALS` environment variable is set,
    ///   it will attempt to load a service account JSON from this path.
    ///
    /// # Returns
    ///
    /// * `Result<KmsSigner, Error>` - A result containing the KmsSigner on
    ///   success, or an Error on failure.
    ///
    /// # Errors
    ///
    /// This function will return an error if it fails to retrieve the required
    /// environment variables or if there is an issue creating the GCP KMS
    /// signer.
    pub async fn gcp_kms_signer(&self) -> Result<KmsSigner, Error> {
        let project_id = std::env::var("GOOGLE_PROJECT_ID").or_else(|_| {
            self.config
                .project_id
                .clone()
                .ok_or(Error::KmsConfig("GOOGLE_PROJECT_ID"))
        })?;
        let location = std::env::var("GOOGLE_LOCATION").or_else(|_| {
            self.config
                .location
                .clone()
                .ok_or(Error::KmsConfig("GOOGLE_LOCATION"))
        })?;
        let keyring_name = std::env::var("GOOGLE_KEYRING").or_else(|_| {
            self.config
                .keyring
                .clone()
                .ok_or(Error::KmsConfig("GOOGLE_KEYRING"))
        })?;
        let key_name = std::env::var("GOOGLE_KEY_NAME").or_else(|_| {
            self.config
                .key_name
                .clone()
                .ok_or(Error::KmsConfig("GOOGLE_KEY_NAME"))
        })?;
        let key_version: u64 = std::env::var("GOOGLE_KEY_VERSION")
            .ok()
            .and_then(|v| v.parse().ok())
            .or(self.config.key_version)
            .ok_or(Error::KmsConfig("GOOGLE_KEY_VERSION"))?;

        let keyring = GcpKeyRingRef::new(&project_id, &location, &keyring_name);
        let specifier = KeySpecifier::new(keyring, &key_name, key_version);

        // Create the GoogleApi client matching the type expected by GcpSigner
        let client =
            GoogleApi::from_function(KeyManagementServiceClient::new, GOOGLE_API_URL, None)
                .await
                .map_err(|e| {
                    Error::KmsError(
                        anyhow::Error::new(e).context("Unable to create GoogleApiClient"),
                    )
                })?;

        // Use GcpSigner::new with the proper client type
        let gcp_signer = GcpSigner::new(client, specifier, Some(self.chain_id))
            .await
            .map_err(|e| {
                Error::KmsError(anyhow::Error::new(e).context("Unable to create GcpSigner"))
            })?;

        Ok(KmsSigner::new(gcp_signer))
    }
}
