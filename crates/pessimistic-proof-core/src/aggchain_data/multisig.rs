use agglayer_primitives::{Address, Digest, Signature};
use alloy_primitives::{keccak256, B256, U256};
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

    #[error("At least one signer comes duplicated. signer: {signer}.")]
    HasDuplicateSigner { signer: Address },

    #[error("Out of bounds signer index: signer index: {idx}, committee size: {total}.")]
    OutOfBoundSignerIndex { idx: usize, total: usize },
}

impl MultiSignature {
    /// Commitment on the signers and threshold.
    pub fn multisig_hash(&self) -> Digest {
        const ADDRESS_BYTES: usize = core::mem::size_of::<Address>(); // 20-bytes
        const THRESHOLD_BYTES: usize = u32::BITS as usize; // 32-bytes

        let mut buf =
            Vec::with_capacity(THRESHOLD_BYTES + ADDRESS_BYTES * self.expected_signers.len());

        // 32-bytes threshold
        buf.extend(U256::from(self.threshold).to_be_bytes::<32>());

        // 20-bytes per signer (no padding)
        for a in &self.expected_signers {
            buf.extend_from_slice(&a.into_array());
        }

        keccak256(&buf).into()
    }

    /// Ensure that the signatures refer to unique and existing signer.
    fn unique_signers(&self) -> Result<(), MultisigError> {
        let total = self.expected_signers.len();
        let mut seen = vec![false; total];

        for &(idx, _) in &self.signatures {
            if idx >= total {
                return Err(MultisigError::OutOfBoundSignerIndex { idx, total });
            }

            if seen[idx] {
                return Err(MultisigError::HasDuplicateSigner {
                    signer: self.expected_signers[idx],
                });
            }

            seen[idx] = true;
        }

        Ok(())
    }

    /// Verify signatures and ensure they are all from the expected set.
    pub fn verify(&self, commitment: B256) -> Result<(), MultisigError> {
        self.unique_signers()?;

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

        if self.signatures.len() < self.threshold {
            return Err(MultisigError::UnderThreshold {
                got: self.signatures.len(),
                expected: self.threshold,
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use alloy::{
        primitives::keccak256,
        signers::{local::PrivateKeySigner, SignerSync as _},
    };
    use rstest::rstest;

    use super::*;

    fn prehash() -> B256 {
        let h = keccak256(b"prehash");
        B256::new(h.0)
    }

    fn wallet(i: usize) -> PrivateKeySigner {
        let seed = keccak256(i.to_be_bytes());
        PrivateKeySigner::from_slice(seed.as_slice()).unwrap()
    }

    #[rstest]
    #[case(vec![0, 1], 2, Ok(()))]
    #[case(vec![0], 2, Err(MultisigError::UnderThreshold { got: 1, expected: 2 }))]
    #[case(vec![2], 1, Err(MultisigError::OutOfBoundSignerIndex { idx: 2, total: 2 }))]
    #[case(
        vec![0, 0],
        2,
        Err(MultisigError::HasDuplicateSigner { signer: wallet(0).address().into() })
    )]
    fn verify_cases(
        #[case] signer_indices: Vec<usize>,
        #[case] threshold: usize,
        #[case] expected: Result<(), MultisigError>,
    ) {
        let wallets: Vec<PrivateKeySigner> = (0..3).map(wallet).collect();
        let prehash = prehash();

        let expected_signers: Vec<Address> = wallets
            .iter()
            .take(2)
            .map(|sk| sk.address().into())
            .collect();

        let signatures: Vec<(usize, Signature)> = signer_indices
            .iter()
            .map(|&idx| {
                let sig = wallet(idx).sign_hash_sync(&prehash).unwrap().into();
                (idx, sig)
            })
            .collect();

        let ms = MultiSignature {
            signatures,
            expected_signers,
            threshold,
        };

        assert_eq!(ms.verify(prehash).map(|_| ()), expected);
    }
}
