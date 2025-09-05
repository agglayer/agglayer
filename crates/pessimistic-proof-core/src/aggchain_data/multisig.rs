use agglayer_primitives::{Address, Digest, Signature};
use alloy_primitives::{keccak256, B256, U256};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MultiSignature {
    /// Set of the indexed signatures
    pub signatures: Vec<Option<Signature>>,
    /// Set of all registered signers
    pub expected_signers: Vec<Address>,
    /// Inclusive minimal number of signers.
    pub threshold: usize,
}

#[derive(Clone, Debug, Error, Serialize, Deserialize, PartialEq, Eq)]
pub enum MultisigError {
    #[error("Too many signers provided. got: {num}, committee size: {max}.")]
    TooManySigners { num: usize, max: usize },

    #[error("Multisig is under the required threshold. got: {got}, expected: {expected}")]
    UnderThreshold { got: usize, expected: usize },

    #[error("Signature claimed to be from signer {idx} is invalid.")]
    InvalidSignature { idx: usize },

    #[error("Signature claimed to be from signer {idx} is from another signer.")]
    InvalidSigner { idx: usize },
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

    /// Verify signatures and ensure they are all from the expected set.
    pub fn verify(&self, commitment: B256) -> Result<(), MultisigError> {
        if self.signatures.len() > self.expected_signers.len() {
            return Err(MultisigError::TooManySigners {
                num: self.signatures.len(),
                max: self.expected_signers.len(),
            });
        }

        let nb_signatures = self.signatures.iter().filter(|s| s.is_some()).count();
        if nb_signatures < self.threshold {
            return Err(MultisigError::UnderThreshold {
                got: nb_signatures,
                expected: self.threshold,
            });
        }

        for (idx, signature) in self.signatures.iter().enumerate() {
            let Some(signature) = signature else {
                continue; // No signature is a valid signature
            };
            let recovered = signature
                .recover_address_from_prehash(&commitment)
                .map_err(|_| MultisigError::InvalidSignature { idx })?;
            let expected = self.expected_signers[idx];
            if recovered != expected {
                return Err(MultisigError::InvalidSigner { idx });
            }
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
    #[case(vec![true, true], 2, Ok(()))]
    #[case(vec![true, false], 2, Err(MultisigError::UnderThreshold { got: 1, expected: 2 }))]
    #[case(vec![false, false, true], 1, Err(MultisigError::TooManySigners { num: 3, max: 2 }))]
    fn verify_cases(
        #[case] signers: Vec<bool>,
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

        let signatures: Vec<Option<Signature>> = signers
            .iter()
            .enumerate()
            .map(|(idx, enabled)| {
                enabled.then(|| wallets[idx].sign_hash_sync(&prehash).unwrap().into())
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
