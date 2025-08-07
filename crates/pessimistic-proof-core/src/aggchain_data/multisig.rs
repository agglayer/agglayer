use agglayer_primitives::{keccak::keccak256_combine, Address, Digest, Signature};
use alloy_primitives::B256;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MultiSignature {
    /// Set of the indexed signatures
    pub signatures: Vec<(usize, Signature)>,
    /// Set of all registered signers
    pub expected_signers: Vec<Address>,
    /// Inclusive minimal number of signers.
    pub threshold: usize,
}

#[derive(Clone, Debug, Error, Serialize, Deserialize, PartialEq, Eq)]
pub enum MultisigError {
    #[error("Multisig is under the required threshold. got: {got}, expected: {expected}")]
    UnderThreshold { got: usize, expected: usize },

    #[error("At least one signature is invalid or from an unregistered signer.")]
    HasInvalidSignature,
}

impl MultiSignature {
    /// Commitment on the signers and threshold.
    pub fn signers_commit(&self) -> Digest {
        // addresses should be padded in 32bytes because solidity
        let mut buf = Vec::with_capacity(self.expected_signers.len());
        buf.extend_from_slice(self.threshold.to_be_bytes().as_slice());
        for addr in &self.expected_signers {
            buf.extend_from_slice(&[0u8; 12]); // padding (12bytes)
            buf.extend_from_slice(addr.as_slice()); // address (20bytes)
        }
        keccak256_combine([buf.as_slice()])
    }

    /// Verify signatures and ensure they are all from the expected set.
    pub fn verify(&self, commitment: B256) -> Result<(), MultisigError> {
        if self.signatures.len() < self.threshold {
            return Err(MultisigError::UnderThreshold {
                got: self.signatures.len(),
                expected: self.threshold,
            });
        }

        let has_invalid_signature = self.signatures.iter().any(|&(signer_idx, signature)| {
            self.expected_signers
                .get(signer_idx)
                .and_then(|expected| {
                    signature
                        .recover_address_from_prehash(&commitment)
                        .ok()
                        .map(|recovered| recovered != *expected)
                })
                .unwrap_or(true)
        });

        if has_invalid_signature {
            return Err(MultisigError::HasInvalidSignature);
        }

        Ok(())
    }
}
