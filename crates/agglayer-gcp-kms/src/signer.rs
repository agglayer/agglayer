//! The [`KmsSigner`] struct is a wrapper around [`GcpKmsSigner`] providing
//! additional functionality for signing messages, transactions, and typed data.

use ethers::{
    signers::Signer,
    types::{
        transaction::{eip2718::TypedTransaction, eip712::Eip712},
        Address, Signature,
    },
};
use ethers_gcp_kms_signer::GcpKmsSigner;

use crate::Error;

/// A wrapper around [`GcpKmsSigner`] providing additional functionality
/// for signing messages, transactions, and typed data.
#[derive(Debug)]
pub struct KmsSigner {
    signer: GcpKmsSigner,
}

impl KmsSigner {
    /// Creates a new [`KmsSigner`] instance.
    pub fn new(signer: GcpKmsSigner) -> Self {
        Self { signer }
    }

    /// Signs a message using the internal signer, this method can fail if the
    /// signer fails to create the digest.
    pub async fn sign_message<S: Send + Sync + AsRef<[u8]>>(
        &self,
        message: S,
    ) -> Result<Signature, Error> {
        Ok(self.signer.sign_message(message).await?)
    }

    /// Signs a transaction using the internal signer, this method can fail if
    /// the signer fails to create the digest.
    pub async fn sign_transaction(&self, tx: &TypedTransaction) -> Result<Signature, Error> {
        Ok(self.signer.sign_transaction(tx).await?)
    }

    /// Signs typed data using internal signer.
    ///
    /// This method can fail while trying to encode the payload using EIP-712 or
    /// during the digest creation.
    pub async fn sign_typed_data<T: Eip712 + Send + Sync>(
        &self,
        payload: &T,
    ) -> Result<Signature, Error> {
        Ok(self.signer.sign_typed_data(payload).await?)
    }

    /// Returns the address associated with the signer.
    pub fn address(&self) -> Address {
        self.signer.address()
    }

    /// Returns the chain ID associated with the signer.
    pub fn chain_id(&self) -> u64 {
        self.signer.chain_id()
    }

    /// Sets a new chain ID for the signer.
    pub fn with_chain_id<T: Into<u64>>(mut self, chain_id: T) -> Self {
        self.signer = self.signer.with_chain_id(chain_id);
        self
    }
}
