use ethers::signers::WalletError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError<T> {
    #[error("no private keys specified in the configuration")]
    NoPk,
    #[error("keystore error: {0}")]
    Wallet(#[from] WalletError),
    #[error("KMS error: {0}")]
    Kms(T),
}
