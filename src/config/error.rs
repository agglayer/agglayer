use ethers::signers::WalletError;
use ethers_gcp_kms_signer::CKMSError;
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum ConfigError {
    #[error("no private keys specified in the configuration")]
    NoPk,
    #[error("keystore error: {0}")]
    Wallet(#[from] WalletError),
    #[error("KMS Provider error: {0}")]
    KmsProvider(#[from] CKMSError),
    #[error("KMS configuration error: missing key or env {0}")]
    KmsConfig(String),
}

/// Errors that can occur when using a [`ConfiguredSigner`].
///
/// This is simply a union of either a [`WalletError`] or a [`CKMSError`].
#[derive(Debug, Error)]
pub(crate) enum ConfiguredSignerError {
    #[error("wallet error: {0}")]
    Wallet(WalletError),
    #[error("KMS error: {0}")]
    Kms(CKMSError),
}
