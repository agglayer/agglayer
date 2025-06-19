//! This crate provides a [`Signer`](trait@alloy_signer::Signer)
//! implementation that can house either a local keystore or a GCP KMS signer.
//! (more signers can be added in the future)
//!
//! See: [`ConfiguredSigner`](enum@ConfiguredSigner)

use std::sync::Arc;

use agglayer_config::{AuthConfig, Config, LocalConfig};
use agglayer_gcp_kms::{KmsSigner, KMS};
use alloy_consensus::TypedTransaction;
use alloy_network::TxSigner;
use alloy_primitives::{Address, ChainId, Signature, B256};
use alloy_signer::Signer;
use alloy_signer_local::{LocalSigner, PrivateKeySigner};
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
#[derive(Debug)]
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
        let signer =
            LocalSigner::decrypt_keystore(&pk.path, &pk.password)?.with_chain_id(Some(chain_id));
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
}

/// [`Signer`] implementation for [`ConfiguredSigner`].
///
/// This implementation simply delegates to the underlying signer.
#[async_trait]
impl Signer for ConfiguredSigner {
    async fn sign_hash(&self, hash: &B256) -> Result<Signature, alloy_signer::Error> {
        match self {
            ConfiguredSigner::Local(wallet) => wallet.sign_hash(hash).await,
            ConfiguredSigner::Kms(signer) => signer
                .sign_message(hash.as_slice())
                .await
                .map_err(alloy_signer::Error::other),
        }
    }

    async fn sign_message(&self, message: &[u8]) -> Result<Signature, alloy_signer::Error> {
        match self {
            ConfiguredSigner::Local(wallet) => wallet.sign_message(message).await,
            ConfiguredSigner::Kms(signer) => signer
                .sign_message(message)
                .await
                .map_err(alloy_signer::Error::other),
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
    fn chain_id(&self) -> Option<ChainId> {
        match self {
            ConfiguredSigner::Local(wallet) => wallet.chain_id(),
            ConfiguredSigner::Kms(signer) => signer.chain_id(),
        }
    }

    /// Sets the signer's chain id
    fn set_chain_id(&mut self, chain_id: Option<ChainId>) {
        match self {
            ConfiguredSigner::Local(wallet) => {
                wallet.set_chain_id(chain_id);
            }
            ConfiguredSigner::Kms(_signer) => {
                // KMS signer doesn't support mutable chain ID changes in the
                // current implementation This is a limitation
                // of the KmsSigner wrapper
            }
        }
    }
}

/// [`TxSigner`] implementation for [`ConfiguredSigner`].
///
/// This implementation provides transaction signing functionality.
#[async_trait]
impl TxSigner<Signature> for ConfiguredSigner {
    fn address(&self) -> Address {
        match self {
            ConfiguredSigner::Local(wallet) => wallet.address(),
            ConfiguredSigner::Kms(signer) => signer.address(),
        }
    }

    async fn sign_transaction(
        &self,
        tx: &mut dyn alloy_consensus::SignableTransaction<Signature>,
    ) -> Result<Signature, alloy_signer::Error> {
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
    pub async fn sign_transaction_typed(&self, tx: &TypedTransaction) -> Result<Signature, Error> {
        match self {
            ConfiguredSigner::Local(wallet) => {
                // Convert TypedTransaction to signable transaction for local wallet
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
