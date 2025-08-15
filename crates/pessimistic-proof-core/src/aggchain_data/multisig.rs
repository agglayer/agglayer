use agglayer_primitives::{keccak::keccak256_combine, Address, Digest, Signature};
use alloy_primitives::B256;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MultiSignature {
    /// Set of the indexed signatures
    pub signatures: Vec<(usize, Signature)>,
    /// Set of all registered signers
    pub expected_signers: Vec<Address>,
    /// Inclusive minimal number of signers.
    pub threshold: usize,
}

impl MultiSignature {
    /// Commitment on the signers and threshold.
    pub fn signers_commit(&self) -> Digest {
        // reminder: addresses should be padded in 32bytes because solidity
        let mut buf = Vec::with_capacity(self.expected_signers.len());
        buf.extend_from_slice(self.threshold.to_be_bytes().as_slice());
        for addr in &self.expected_signers {
            let a = addr.as_slice(); // 20 bytes
            buf.extend_from_slice(&[0u8; 12]); // padded within 32 bytes
            buf.extend_from_slice(a);
        }
        keccak256_combine([buf.as_slice()])
    }

    /// Verify signatures and ensure they are all from the expected set.
    pub fn verify(&self, commitment: B256) -> Result<(), ()> {
        if self.signatures.len() < self.threshold {
            return Err(()); // todo: dedicated error
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
            return Err(()); // todo: dedicated error
        }

        Ok(())
    }
}
