use agglayer_gcp_kms::Error as GcpKmsError;
use ethers::signers::WalletError;
use thiserror::Error;

/// Errors that can occur when using a
/// [`ConfiguredSigner`](enum@super::ConfiguredSigner).
///
/// This is simply a union of either a [`WalletError`] or a [`GcpKmsError`].
#[derive(Debug, Error)]
pub enum Error {
    #[error("no private keys specified in the configuration")]
    NoPk,
    #[error("wallet error: {0}")]
    Wallet(#[from] WalletError),
    #[error("GcpKMS error: {0}")]
    GcpKms(#[from] GcpKmsError),
}
