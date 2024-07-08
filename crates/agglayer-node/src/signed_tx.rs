//! The core input of the agglayer.
//!
//! Systems that wish to submit proofs to the agglayer must produce a
//! [`SignedTx`] conforming to the type definitions specified herein.
use ethers::{prelude::*, utils::keccak256};
use serde::{Deserialize, Deserializer};
use serde_with::{serde_as, DisplayFromStr};
use thiserror::Error;

pub(crate) const HASH_LENGTH: usize = 32;
pub(crate) const PROOF_LENGTH: usize = 24;

/// Raw proof bytes.
///
/// This is a fixed-size array of fixed-size arrays, where each inner array is a
/// 32-byte hash.
#[derive(Debug)]
pub(crate) struct Proof([[u8; HASH_LENGTH]; PROOF_LENGTH]);

#[derive(Error, Debug)]
pub(crate) enum ProofEncodingError {
    #[error("invalid proof length: expected {expected}, got {got}")]
    InvalidLength { expected: usize, got: usize },
    #[error("invalid hash at index {index}")]
    InvalidHash { index: usize },
}

impl Proof {
    /// Convert the proof into a byte array.
    pub(crate) fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(HASH_LENGTH * PROOF_LENGTH);
        for hash in &self.0 {
            bytes.extend_from_slice(&hash[..]);
        }
        bytes
    }

    /// Convert the proof into a fixed-size array of byte arrays.
    pub(crate) fn to_fixed_bytes(&self) -> [[u8; HASH_LENGTH]; PROOF_LENGTH] {
        self.0
    }

    /// Convert a byte array into a proof.
    pub(crate) fn try_from_slice(slice: &[u8]) -> Result<Self, ProofEncodingError> {
        if slice.len() != HASH_LENGTH * PROOF_LENGTH {
            return Err(ProofEncodingError::InvalidLength {
                expected: HASH_LENGTH * PROOF_LENGTH,
                got: slice.len(),
            });
        }

        let mut proof = [[0; HASH_LENGTH]; PROOF_LENGTH];
        for (i, hash) in slice.chunks_exact(HASH_LENGTH).enumerate() {
            proof[i] = hash
                .try_into()
                .map_err(|_| ProofEncodingError::InvalidHash { index: i })?;
        }

        Ok(Self(proof))
    }
}

impl<'de> Deserialize<'de> for Proof {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Proof::try_from_slice(&Bytes::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

/// The zero-knowledge proof.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Zkp {
    pub(crate) new_state_root: H256,
    pub(crate) new_local_exit_root: H256,
    pub(crate) proof: Proof,
}

/// Proof metadata along with its zero-knowledge proof.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProofManifest {
    #[serde(rename = "RollupID")]
    pub(crate) rollup_id: u32,
    pub(crate) last_verified_batch: U64,
    pub(crate) new_verified_batch: U64,
    #[serde(rename = "ZKP")]
    pub(crate) zkp: Zkp,
}

/// A [`SignedTx`] is the core input type of the agglayer.
///
/// Systems that wish to submit proofs to the agglayer must produce a
/// [`SignedTx`] conforming to the type definitions specified herein.
#[serde_as]
#[derive(Debug, Deserialize)]
pub(crate) struct SignedTx {
    pub(crate) tx: ProofManifest,
    #[serde_as(as = "DisplayFromStr")]
    pub(crate) signature: Signature,
}

impl SignedTx {
    /// Generate a hash that uniquely identifies this proof.
    pub(crate) fn hash(&self) -> H256 {
        let last_verified_batch_hex = format!("0x{:x}", self.tx.last_verified_batch.as_u64());
        let new_verified_batch_hex = format!("0x{:x}", self.tx.new_verified_batch.as_u64());
        let proof_hex = format!("0x{}", hex::encode(self.tx.zkp.proof.as_bytes()));

        let data = [
            last_verified_batch_hex.as_bytes(),
            new_verified_batch_hex.as_bytes(),
            &self.tx.zkp.new_state_root[..],
            &self.tx.zkp.new_local_exit_root[..],
            proof_hex.as_bytes(),
        ]
        .concat();

        keccak256(data).into()
    }

    /// Attempt to recover the address of the signer.
    pub(crate) fn signer(&self) -> Result<Address, SignatureError> {
        self.signature.recover(self.hash())
    }

    #[cfg(test)]
    pub(crate) fn sign(
        &mut self,
        signer: &Wallet<k256::ecdsa::SigningKey>,
    ) -> Result<(), SignatureError> {
        self.signature = signer.sign_hash(self.hash()).unwrap();

        Ok(())
    }
}
