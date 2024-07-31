//! The [`enum@Error`] enum represents errors that can occur in the KMS
//! operations.
//!
//! It includes errors from the KMS provider and configuration errors.

use ethers_gcp_kms_signer::CKMSError;
use thiserror::Error;

/// Represents errors that can occur in the KMS operations.
#[derive(Debug, Error)]
pub enum Error {
    /// An error occurred with the KMS provider.
    ///
    /// This variant wraps the underlying [`CKMSError`] from the
    /// `ethers_gcp_kms_signer` library.
    #[error("KMS Provider error: {0}")]
    KmsProvider(#[from] CKMSError),

    /// An error occurred with the KMS configuration.
    ///
    /// This variant is used when a required key or environment variable is
    /// missing.
    #[error("KMS configuration error: missing key or env {0}")]
    KmsConfig(&'static str),
}
