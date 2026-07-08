//! The [`KMS`] struct provides functionality to create a GCP KMS signer from a
//! configuration. This struct is used to initialize and configure a Google
//! Cloud KMS signer.

use agglayer_config::GcpKmsConfig;
use alloy::signers::gcp::{GcpKeyRingRef, GcpSigner, KeySpecifier};
use eyre::Context as _;
use gcloud_sdk::{
    google::cloud::kms::v1::key_management_service_client::KeyManagementServiceClient, GoogleApi,
};
use serde::Deserialize;

pub(crate) mod params;
pub(crate) mod signer;

pub use signer::KmsSigner;
use tracing::debug;

use crate::params::KMSParameters;

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
    /// * `eyre::Result<KmsSigner>` - A result containing the KmsSigner on
    ///   success, or an error report on failure.
    ///
    /// # Errors
    ///
    /// This function will return an error if it fails to retrieve the required
    /// environment variables or if there is an issue creating the GCP KMS
    /// signer.
    pub async fn gcp_kms_signer(&self) -> eyre::Result<KmsSigner> {
        let params = KMSParameters::try_from(&self.config)?;
        debug!("Using GCP KMS with parameters: {:?}", params);

        let keyring =
            GcpKeyRingRef::new(&params.project_id, &params.location, &params.keyring_name);
        let pp_settlement_specifier = KeySpecifier::new(
            keyring,
            &params.key_name_pp_settlement,
            params.key_version_pp_settlement,
        );

        // Create the GoogleApi client matching the type expected by GcpSigner
        let client =
            GoogleApi::from_function(KeyManagementServiceClient::new, GOOGLE_API_URL, None)
                .await
                .wrap_err("Unable to create GoogleApiClient")?;

        // Use GcpSigner::new with the proper client type
        let pp_settlement_gcp_signer =
            GcpSigner::new(client, pp_settlement_specifier, Some(self.chain_id))
                .await
                .wrap_err("Unable to create PP settlement GcpSigner")?;

        Ok(KmsSigner::new(pp_settlement_gcp_signer))
    }
}
