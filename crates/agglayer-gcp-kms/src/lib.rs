//! The [`KMS`] struct provides functionality to create a GCP KMS signer from a
//! configuration. This struct is used to initialize and configure a Google
//! Cloud KMS signer.

use std::{fmt::Display, str::FromStr};

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
use tracing::warn;

pub const GOOGLE_API_URL: &str = "https://cloudkms.googleapis.com";
const GOOGLE_PROJECT_ID: &str = "GOOGLE_PROJECT_ID";
const GOOGLE_LOCATION: &str = "GOOGLE_LOCATION";
const GOOGLE_KEYRING: &str = "GOOGLE_KEYRING";
const GOOGLE_KEY_NAME_LEGACY: &str = "GOOGLE_KEY_NAME";
const GOOGLE_KEY_VERSION_LEGACY: &str = "GOOGLE_KEY_VERSION";
const GOOGLE_KEY_NAME_CERT_SETTLEMENT: &str = "GOOGLE_KEY_NAME_CERT_SETTLEMENT";
const GOOGLE_KEY_VERSION_CERT_SETTLEMENT: &str = "GOOGLE_KEY_VERSION_CERT_SETTLEMENT";
const GOOGLE_KEY_NAME_TX_SETTLEMENT: &str = "GOOGLE_KEY_NAME_TX_SETTLEMENT";
const GOOGLE_KEY_VERSION_TX_SETTLEMENT: &str = "GOOGLE_KEY_VERSION_TX_SETTLEMENT";

#[derive(Deserialize, Debug)]
pub struct KMS {
    /// The L1 chain id.
    chain_id: u64,
    /// The GCP KMS configuration.
    config: GcpKmsConfig,
}

// Helper function to get a value from environment variables or configuration
// with fallbacks.
// The function returns the value in the following order of precedence:
// 1. The primary environment variable.
// 2. The fallback environment variable (if provided).
// 3. The primary configuration value.
// 4. The fallback configuration value (if provided).
// If none are found, it returns an error.
fn from_env_or_conf<T>(
    env_key: &str,
    env_key_fallback: Option<&str>,
    config_value: &Option<T>,
    config_value_fallback: &Option<T>,
) -> Result<T, Error>
where
    T: FromStr + Clone + Display,
{
    if let Ok(value) = std::env::var(env_key) {
        return value
            .parse()
            .map_err(|_| Error::KmsConfig(env_key.to_string()));
    } else if let Some(env_key_fallback) = env_key_fallback {
        if let Ok(value) = std::env::var(env_key_fallback) {
            warn!("Fallback KMS env env:{env_key}=>env:{env_key_fallback}");
            return value
                .parse()
                .map_err(|_| Error::KmsConfig(env_key_fallback.to_string()));
        }
    } else if let Some(config_value) = config_value {
        return Ok(config_value.clone());
    } else if let Some(config_value_fallback) = config_value_fallback {
        warn!("Fallback KMS config env:{env_key}=>config:{config_value_fallback}");
        return Ok(config_value_fallback.clone());
    }
    Err(Error::KmsConfig(env_key.to_string()))
}

#[derive(Debug)]
pub struct KmsSigners {
    // The signer for certificate settlement.
    pub cert_settlement: KmsSigner,
    // The signer for transaction settlement
    pub tx_settlement: Option<KmsSigner>,
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
    pub async fn gcp_kms_signers(&self) -> Result<KmsSigners, Error> {
        // Get configuration values from environment variables or config
        let project_id = from_env_or_conf(GOOGLE_PROJECT_ID, None, &self.config.project_id, &None)?;
        let location = from_env_or_conf(GOOGLE_LOCATION, None, &self.config.location, &None)?;
        let keyring_name = from_env_or_conf(GOOGLE_KEYRING, None, &self.config.keyring, &None)?;
        let key_name_cert_settlement = from_env_or_conf(
            GOOGLE_KEY_NAME_CERT_SETTLEMENT,
            Some(GOOGLE_KEY_NAME_LEGACY),
            &self.config.cert_settlement_key_name,
            &self.config.key_name,
        )?;
        let key_version_cert_settlement = from_env_or_conf(
            GOOGLE_KEY_VERSION_CERT_SETTLEMENT,
            Some(GOOGLE_KEY_VERSION_LEGACY),
            &self.config.cert_settlement_key_version,
            &self.config.key_version,
        )?;
        let key_name_tx_settlement = from_env_or_conf(
            GOOGLE_KEY_NAME_TX_SETTLEMENT,
            Some(GOOGLE_KEY_NAME_LEGACY),
            &self.config.tx_settlement_key_name,
            &self.config.key_name,
        )?;
        let key_version_tx_settlement = from_env_or_conf(
            GOOGLE_KEY_VERSION_TX_SETTLEMENT,
            Some(GOOGLE_KEY_VERSION_LEGACY),
            &self.config.tx_settlement_key_version,
            &self.config.key_version,
        )?;

        // create KeySpecifier for both signers
        let keyring = GcpKeyRingRef::new(&project_id, &location, &keyring_name);
        let cert_settlement_specifier = KeySpecifier::new(
            keyring.clone(),
            &key_name_cert_settlement,
            key_version_cert_settlement,
        );
        let tx_settlement_specifier =
            KeySpecifier::new(keyring, &key_name_tx_settlement, key_version_tx_settlement);

        // Create the GoogleApi client matching the type expected by GcpSigner
        let client =
            GoogleApi::from_function(KeyManagementServiceClient::new, GOOGLE_API_URL, None)
                .await
                .map_err(|e| {
                    Error::KmsError(
                        eyre::Error::new(e).wrap_err("Unable to create GoogleApiClient"),
                    )
                })?;

        // Use GcpSigner::new with the proper client type
        let cert_settlement_gcp_signer = GcpSigner::new(
            client.clone(),
            cert_settlement_specifier,
            Some(self.chain_id),
        )
        .await
        .map_err(|e| Error::KmsError(eyre::Error::new(e).wrap_err("Unable to create GcpSigner")))?;

        let is_the_same_key = key_name_cert_settlement == key_name_tx_settlement
            && key_version_cert_settlement == key_version_tx_settlement;

        let tx_settlement_gcp_signer = if is_the_same_key {
            None
        } else {
            Some(
                GcpSigner::new(client, tx_settlement_specifier, Some(self.chain_id))
                    .await
                    .map_err(|e| {
                        Error::KmsError(eyre::Error::new(e).wrap_err("Unable to create GcpSigner"))
                    })?,
            )
        };

        Ok(KmsSigners {
            cert_settlement: KmsSigner::new(cert_settlement_gcp_signer),
            tx_settlement: tx_settlement_gcp_signer.map(KmsSigner::new),
        })
    }
}
