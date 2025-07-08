//! This crate provides a [`Signer`](trait@alloy_signer::Signer)
//! implementation that can house either a local keystore or a GCP KMS signer.
//! (more signers can be added in the future)
//!
//! See: [`ConfiguredSigner`](enum@ConfiguredSigner)

use std::sync::Arc;

use agglayer_config::{AuthConfig, Config, LocalConfig};
use agglayer_gcp_kms::{KmsSigner, KMS};
use alloy::{
    consensus::TypedTransaction,
    network::TxSigner,
    signers::{local::PrivateKeySigner, Signer},
};
use alloy_primitives::{Address, ChainId, Signature, B256};
use async_trait::async_trait;

mod error;

pub use error::Error;

/// A an alloy [`Signer`] that can house either a local keystore or a KMS
/// signer.
///
/// An alloy [`Provider`] using a signer must have its
/// [`Signer`] type specified at compile time, and the Signer type is not object
/// safe, so we cannot use a `Box<dyn Signer>`. As such, we define this enum to
/// accommodate a runtime configured signer.
#[derive(Debug, derive_more::IsVariant)]
pub enum ConfiguredSigner {
    Local(PrivateKeySigner),
    Kms(KmsSigner),
}

impl ConfiguredSigner {
    /// Decrypt the first local keystore specified in the configuration.
    #[allow(clippy::result_large_err)]
    pub(crate) fn local_wallet(
        chain_id: u64,
        local: &LocalConfig,
    ) -> Result<PrivateKeySigner, Error> {
        let pk = local.private_keys.first().ok_or(Error::NoPk)?;
        let signer = PrivateKeySigner::decrypt_keystore(&pk.path, &pk.password)?
            .with_chain_id(Some(chain_id));
        Ok(signer)
    }

    /// Get either a local wallet or GCP KMS signer based on the configuration.
    pub async fn new(config: Arc<Config>) -> Result<Self, Error> {
        match &config.auth {
            AuthConfig::GcpKms(ref kms) => {
                let kms = KMS::new(config.l1.chain_id, kms.clone());
                Ok(Self::Kms(kms.gcp_kms_signer().await?))
            }
            AuthConfig::Local(ref local) => {
                Ok(Self::Local(Self::local_wallet(config.l1.chain_id, local)?))
            }
        }
    }

    /// Create a new ConfiguredSigner from a local private key signer.
    /// This is a more efficient constructor when you already have a local
    /// signer.
    #[inline]
    pub const fn from_local(signer: PrivateKeySigner) -> Self {
        Self::Local(signer)
    }

    /// Create a new ConfiguredSigner from a KMS signer.
    /// This is a more efficient constructor when you already have a KMS signer.
    #[inline]
    pub const fn from_kms(signer: KmsSigner) -> Self {
        Self::Kms(signer)
    }
}

/// [`Signer`] implementation for [`ConfiguredSigner`].
///
/// This implementation simply delegates to the underlying signer.
#[async_trait]
impl Signer for ConfiguredSigner {
    #[inline]
    async fn sign_hash(&self, hash: &B256) -> Result<Signature, alloy::signers::Error> {
        match self {
            ConfiguredSigner::Local(signer) => signer.sign_hash(hash).await,
            ConfiguredSigner::Kms(signer) => signer.sign_hash(hash).await,
        }
    }

    #[inline]
    async fn sign_message(&self, message: &[u8]) -> Result<Signature, alloy::signers::Error> {
        match self {
            ConfiguredSigner::Local(wallet) => wallet.sign_message(message).await,
            ConfiguredSigner::Kms(signer) => signer
                .sign_message(message)
                .await
                .map_err(alloy::signers::Error::other),
        }
    }

    /// Returns the signer's Ethereum Address
    #[inline]
    fn address(&self) -> Address {
        match self {
            ConfiguredSigner::Local(wallet) => wallet.address(),
            ConfiguredSigner::Kms(signer) => signer.address(),
        }
    }

    /// Returns the signer's chain id
    #[inline]
    fn chain_id(&self) -> Option<ChainId> {
        match self {
            ConfiguredSigner::Local(wallet) => wallet.chain_id(),
            ConfiguredSigner::Kms(signer) => signer.chain_id(),
        }
    }

    /// Sets the signer's chain id
    #[inline]
    fn set_chain_id(&mut self, chain_id: Option<ChainId>) {
        match self {
            ConfiguredSigner::Local(wallet) => {
                wallet.set_chain_id(chain_id);
            }
            ConfiguredSigner::Kms(_signer) => {
                // KMS signer doesn't support mutable chain ID changes in the
                // current implementation This is a limitation
                // of the KmsSigner wrapper
                panic!("KMS signer doesn't support mutable chain ID changes");
            }
        }
    }
}

/// [`TxSigner`] implementation for [`ConfiguredSigner`].
///
/// This implementation provides transaction signing functionality.
#[async_trait]
impl TxSigner<Signature> for ConfiguredSigner {
    #[inline]
    fn address(&self) -> Address {
        match self {
            ConfiguredSigner::Local(wallet) => wallet.address(),
            ConfiguredSigner::Kms(signer) => signer.address(),
        }
    }

    #[inline]
    async fn sign_transaction(
        &self,
        tx: &mut dyn alloy::consensus::SignableTransaction<Signature>,
    ) -> Result<Signature, alloy::signers::Error> {
        match self {
            ConfiguredSigner::Local(wallet) => wallet.sign_transaction(tx).await,
            ConfiguredSigner::Kms(signer) => TxSigner::sign_transaction(signer, tx).await,
        }
    }
}

impl ConfiguredSigner {
    /// Signs a transaction using the appropriate signer.
    ///
    /// This method provides transaction signing functionality that delegates
    /// to the underlying signer implementation.
    #[inline]
    pub async fn sign_transaction_typed(&self, tx: &TypedTransaction) -> Result<Signature, Error> {
        match self {
            ConfiguredSigner::Local(wallet) => {
                let mut tx_clone = tx.clone();
                wallet
                    .sign_transaction(&mut tx_clone)
                    .await
                    .map_err(Error::Signer)
            }
            ConfiguredSigner::Kms(signer) => {
                signer.sign_transaction(tx).await.map_err(Error::GcpKms)
            }
        }
    }
}

#[cfg(test)]
mod tests;
