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
    ///
    /// # Arguments
    ///
    /// * `signer` - An instance of [`GcpKmsSigner`].
    ///
    /// # Returns
    ///
    /// * `Self` - A new instance of [`KmsSigner`].
    pub fn new(signer: GcpKmsSigner) -> Self {
        Self { signer }
    }

    /// Signs a message asynchronously.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to be signed.
    ///
    /// # Returns
    ///
    /// * `Result<Signature, Error>` - A result containing the signature on
    ///   success, or an error on failure.
    pub async fn sign_message<S: Send + Sync + AsRef<[u8]>>(
        &self,
        message: S,
    ) -> Result<Signature, Error> {
        Ok(self.signer.sign_message(message).await?)
    }

    /// Signs a transaction asynchronously.
    ///
    /// # Arguments
    ///
    /// * `tx` - A reference to the typed transaction to be signed.
    ///
    /// # Returns
    ///
    /// * `Result<Signature, Error>` - A result containing the signature on
    ///   success, or an error on failure.
    pub async fn sign_transaction(&self, tx: &TypedTransaction) -> Result<Signature, Error> {
        Ok(self.signer.sign_transaction(tx).await?)
    }

    /// Signs typed data asynchronously.
    ///
    /// # Arguments
    ///
    /// * `payload` - A reference to the typed data payload to be signed.
    ///
    /// # Returns
    ///
    /// * `Result<Signature, Error>` - A result containing the signature on
    ///   success, or an error on failure.
    pub async fn sign_typed_data<T: Eip712 + Send + Sync>(
        &self,
        payload: &T,
    ) -> Result<Signature, Error> {
        Ok(self.signer.sign_typed_data(payload).await?)
    }

    /// Returns the address associated with the signer.
    ///
    /// # Returns
    ///
    /// * `Address` - The address associated with the signer.
    pub fn address(&self) -> Address {
        self.signer.address()
    }

    /// Returns the chain ID associated with the signer.
    ///
    /// # Returns
    ///
    /// * `u64` - The chain ID associated with the signer.
    pub fn chain_id(&self) -> u64 {
        self.signer.chain_id()
    }

    /// Sets a new chain ID for the signer.
    ///
    /// # Arguments
    ///
    /// * `chain_id` - The new chain ID to set.
    ///
    /// # Returns
    ///
    /// * `Self` - The `KmsSigner` instance with the updated chain ID.
    pub fn with_chain_id<T: Into<u64>>(mut self, chain_id: T) -> Self {
        self.signer = self.signer.with_chain_id(chain_id);
        self
    }
}
