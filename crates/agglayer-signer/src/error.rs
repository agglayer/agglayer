use agglayer_gcp_kms::Error as KmsError;
use ethers::signers::WalletError;
use thiserror::Error;

/// Errors that can occur when using a [`ConfiguredSigner`].
///
/// This is simply a union of either a [`WalletError`] or a [`KmsError`].
#[derive(Debug, Error)]
pub enum Error {
    #[error("no private keys specified in the configuration")]
    NoPk,
    #[error("wallet error: {0}")]
    Wallet(#[from] WalletError),
    #[error("KMS error: {0}")]
    Kms(#[from] KmsError),
}
