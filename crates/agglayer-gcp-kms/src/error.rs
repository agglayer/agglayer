//! The [`enum@Error`] enum represents errors that can occur in the KMS
//! operations.
//!
//! It includes errors from the KMS provider and configuration errors.

use thiserror::Error;

/// Represents errors that can occur in the KMS operations.
#[derive(Debug, Error)]
pub enum Error {
    /// This variant is used when a required key or environment variable is
    /// missing.
    #[error("KMS configuration error: missing key or env {0}")]
    KmsConfig(&'static str),

    /// An error occurred while interacting with the KMS provider.
    #[error("KMS error: {0}")]
    KmsError(anyhow::Error),
}
