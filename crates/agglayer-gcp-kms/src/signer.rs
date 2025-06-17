//! The [`KmsSigner`] struct is a wrapper around [`GcpSigner`] providing
//! additional functionality for signing messages and transactions.

use alloy_consensus::{SignableTransaction, TypedTransaction};
use alloy_network::TxSigner;
use alloy_primitives::{Address, ChainId, Signature, B256};
use alloy_signer::Signer;
use alloy_signer_gcp::GcpSigner;
use async_trait::async_trait;

use crate::Error;

/// A wrapper around [`GcpSigner`] providing additional functionality
/// for signing messages and transactions.
#[derive(Debug, Clone)]
pub struct KmsSigner {
    signer: GcpSigner,
}

impl KmsSigner {
    /// Creates a new [`KmsSigner`] instance.
    pub fn new(signer: GcpSigner) -> Self {
        Self { signer }
    }

    /// Signs a message using the internal signer, this method can fail if the
    /// signer fails to create the digest.
    pub async fn sign_message<S: Send + Sync + AsRef<[u8]>>(
        &self,
        message: S,
    ) -> Result<Signature, Error> {
        self.signer
            .sign_message(message.as_ref())
            .await
            .map_err(|e| Error::KmsError(e.to_string()))
    }

    /// Signs a transaction using the internal signer, this method can fail if
    /// the signer fails to create the digest.
    pub async fn sign_transaction(&self, tx: &TypedTransaction) -> Result<Signature, Error> {
        // Convert the TypedTransaction to a mutable dyn SignableTransaction
        let mut tx_clone = tx.clone();
        self.signer
            .sign_transaction(&mut tx_clone)
            .await
            .map_err(|e| Error::KmsError(e.to_string()))
    }

    /// Returns the address associated with the signer.
    pub fn address(&self) -> Address {
        alloy_signer::Signer::address(&self.signer)
    }

    /// Returns the chain ID associated with the signer.
    pub fn chain_id(&self) -> Option<ChainId> {
        self.signer.chain_id()
    }

    /// Sets a new chain ID for the signer.
    pub fn with_chain_id<T: Into<u64>>(mut self, chain_id: T) -> Self {
        self.signer = self.signer.with_chain_id(Some(chain_id.into()));
        self
    }

    /// Sets the chain ID on this signer (mutable version).
    pub fn set_chain_id(&mut self, chain_id: Option<ChainId>) {
        if let Some(chain_id) = chain_id {
            self.signer = self.signer.clone().with_chain_id(Some(chain_id));
        }
    }
}

/// Implementation of alloy's [`Signer`] trait for [`KmsSigner`].
///
/// This allows the KmsSigner to be used anywhere an alloy Signer is expected.
#[async_trait]
impl Signer for KmsSigner {
    async fn sign_hash(&self, hash: &B256) -> Result<Signature, alloy_signer::Error> {
        self.signer
            .sign_hash(hash)
            .await
            .map_err(|e| alloy_signer::Error::other(e))
    }

    async fn sign_message(&self, message: &[u8]) -> Result<Signature, alloy_signer::Error> {
        self.signer
            .sign_message(message)
            .await
            .map_err(|e| alloy_signer::Error::other(e))
    }

    fn address(&self) -> Address {
        alloy_signer::Signer::address(&self.signer)
    }

    fn chain_id(&self) -> Option<ChainId> {
        self.signer.chain_id()
    }

    fn set_chain_id(&mut self, chain_id: Option<ChainId>) {
        self.set_chain_id(chain_id);
    }
}

/// Implementation of alloy's [`TxSigner`] trait for [`KmsSigner`].
///
/// This allows the KmsSigner to be used for transaction signing with the
/// standard alloy interface.
#[async_trait]
impl TxSigner<Signature> for KmsSigner {
    fn address(&self) -> Address {
        alloy_signer::Signer::address(&self.signer)
    }

    async fn sign_transaction(
        &self,
        tx: &mut dyn SignableTransaction<Signature>,
    ) -> Result<Signature, alloy_signer::Error> {
        self.signer
            .sign_transaction(tx)
            .await
            .map_err(|e| alloy_signer::Error::other(e))
    }
}
