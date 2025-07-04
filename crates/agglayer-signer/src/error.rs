use agglayer_gcp_kms::Error as GcpKmsError;
use alloy::signers::{local::LocalSignerError, Error as SignerError};
use thiserror::Error;

/// Errors that can occur when using a
/// [`ConfiguredSigner`](enum@super::ConfiguredSigner).
///
/// This is a union of signer errors, local signer errors, and GCP KMS errors.
#[derive(Debug, Error)]
pub enum Error {
    #[error("no private keys specified in the configuration")]
    NoPk,
    #[error("signer error: {0}")]
    Signer(#[from] SignerError),
    #[error("local signer error: {0}")]
    LocalSigner(#[from] LocalSignerError),
    #[error("GcpKMS error: {0}")]
    GcpKms(#[from] GcpKmsError),
}
