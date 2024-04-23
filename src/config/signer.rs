use async_trait::async_trait;
use ethers::{
    abi::Address,
    signers::{LocalWallet, Signer},
    types::{
        transaction::{eip2718::TypedTransaction, eip712::Eip712},
        Signature,
    },
};
use ethers_gcp_kms_signer::GcpKmsSigner;

use super::error::ConfiguredSignerError;

/// A an ethers [`Signer`] that can house either a local keystore or a GCP KMS
/// signer.
///
/// An ethers [`Provider`][ethers::prelude::Provider] using a
/// [`SignerMiddleware`][ethers::prelude::SignerMiddleware] must have its
/// [`Signer`] type specified at compile time, and the Signer type is not object
/// safe, so we cannot use a `Box<dyn Signer>`. As such, we define this enum to
/// accommodate a runtime configured signer.
#[derive(Debug)]
pub(crate) enum ConfiguredSigner {
    Local(LocalWallet),
    GcpKms(GcpKmsSigner),
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
            ConfiguredSigner::GcpKms(signer) => signer
                .sign_message(message)
                .await
                .map_err(ConfiguredSignerError::Kms),
        }
    }

    /// Signs the transaction
    async fn sign_transaction(&self, message: &TypedTransaction) -> Result<Signature, Self::Error> {
        match self {
            ConfiguredSigner::Local(wallet) => wallet
                .sign_transaction(message)
                .await
                .map_err(ConfiguredSignerError::Wallet),
            ConfiguredSigner::GcpKms(signer) => signer
                .sign_transaction(message)
                .await
                .map_err(ConfiguredSignerError::Kms),
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
            ConfiguredSigner::GcpKms(signer) => signer
                .sign_typed_data(payload)
                .await
                .map_err(ConfiguredSignerError::Kms),
        }
    }

    /// Returns the signer's Ethereum Address
    fn address(&self) -> Address {
        match self {
            ConfiguredSigner::Local(wallet) => wallet.address(),
            ConfiguredSigner::GcpKms(signer) => signer.address(),
        }
    }

    /// Returns the signer's chain id
    fn chain_id(&self) -> u64 {
        match self {
            ConfiguredSigner::Local(wallet) => wallet.chain_id(),
            ConfiguredSigner::GcpKms(signer) => signer.chain_id(),
        }
    }

    /// Sets the signer's chain id
    #[must_use]
    fn with_chain_id<T: Into<u64>>(self, chain_id: T) -> Self {
        match self {
            ConfiguredSigner::Local(wallet) => {
                ConfiguredSigner::Local(wallet.with_chain_id(chain_id))
            }
            ConfiguredSigner::GcpKms(signer) => {
                ConfiguredSigner::GcpKms(signer.with_chain_id(chain_id))
            }
        }
    }
}
