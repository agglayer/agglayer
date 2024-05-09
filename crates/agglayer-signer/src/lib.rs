//! This crate provides a [`Signer`](trait@ethers::signers::Signer)
//! implementation that can house either a local keystore or a KMS signer.
//! (more signers can be added in the future)
//!
//! See: [`ConfiguredSigner`](enum@ConfiguredSigner)
use agglayer_gcp_kms::{KmsError, KmsSigner};
use async_trait::async_trait;
use ethers::{
    abi::Address,
    signers::{LocalWallet, Signer, WalletError},
    types::{
        transaction::{eip2718::TypedTransaction, eip712::Eip712},
        Signature,
    },
};
use thiserror::Error;

/// Errors that can occur when using a [`ConfiguredSigner`].
///
/// This is simply a union of either a [`WalletError`] or a [`KmsError`].
#[derive(Debug, Error)]
pub enum ConfiguredSignerError {
    #[error("wallet error: {0}")]
    Wallet(WalletError),
    #[error("KMS error: {0}")]
    Kms(KmsError),
}

/// A an ethers [`Signer`] that can house either a local keystore or a KMS
/// signer.
///
/// An ethers [`Provider`][ethers::prelude::Provider] using a
/// [`SignerMiddleware`][ethers::prelude::SignerMiddleware] must have its
/// [`Signer`] type specified at compile time, and the Signer type is not object
/// safe, so we cannot use a `Box<dyn Signer>`. As such, we define this enum to
/// accommodate a runtime configured signer.
#[derive(Debug)]
pub enum ConfiguredSigner {
    Local(LocalWallet),
    Kms(KmsSigner),
}

/// [`Signer`] implementation for [`ConfiguredSigner`].
///
/// This implementation simply delegates to the underlying signer.
#[async_trait]
impl Signer for ConfiguredSigner {
    type Error = ConfiguredSignerError;

    async fn sign_message<S: Send + Sync + AsRef<[u8]>>(
        &self,
        message: S,
    ) -> Result<Signature, Self::Error> {
        match self {
            ConfiguredSigner::Local(wallet) => wallet
                .sign_message(message)
                .await
                .map_err(ConfiguredSignerError::Wallet),
            ConfiguredSigner::Kms(signer) => signer
                .sign_message(message)
                .await
                .map_err(|err| ConfiguredSignerError::Kms(err)),
        }
    }

    /// Signs the transaction
    async fn sign_transaction(&self, message: &TypedTransaction) -> Result<Signature, Self::Error> {
        match self {
            ConfiguredSigner::Local(wallet) => wallet
                .sign_transaction(message)
                .await
                .map_err(ConfiguredSignerError::Wallet),
            ConfiguredSigner::Kms(signer) => signer
                .sign_transaction(message)
                .await
                .map_err(|err| ConfiguredSignerError::Kms(err)),
        }
    }

    /// Encodes and signs the typed data according EIP-712.
    /// Payload must implement Eip712 trait.
    async fn sign_typed_data<T: Eip712 + Send + Sync>(
        &self,
        payload: &T,
    ) -> Result<Signature, Self::Error> {
        match self {
            ConfiguredSigner::Local(wallet) => wallet
                .sign_typed_data(payload)
                .await
                .map_err(ConfiguredSignerError::Wallet),
            ConfiguredSigner::Kms(signer) => signer
                .sign_typed_data(payload)
                .await
                .map_err(|err| ConfiguredSignerError::Kms(err)),
        }
    }

    /// Returns the signer's Ethereum Address
    fn address(&self) -> Address {
        match self {
            ConfiguredSigner::Local(wallet) => wallet.address(),
            ConfiguredSigner::Kms(signer) => signer.address(),
        }
    }

    /// Returns the signer's chain id
    fn chain_id(&self) -> u64 {
        match self {
            ConfiguredSigner::Local(wallet) => wallet.chain_id(),
            ConfiguredSigner::Kms(signer) => signer.chain_id(),
        }
    }

    /// Sets the signer's chain id
    #[must_use]
    fn with_chain_id<T: Into<u64>>(self, chain_id: T) -> Self {
        match self {
            ConfiguredSigner::Local(wallet) => {
                ConfiguredSigner::Local(wallet.with_chain_id(chain_id))
            }
            ConfiguredSigner::Kms(signer) => ConfiguredSigner::Kms(signer.with_chain_id(chain_id)),
        }
    }
}
