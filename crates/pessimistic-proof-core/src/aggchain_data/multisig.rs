use agglayer_primitives::{keccak::keccak256_combine, Address, Digest, Signature};
use alloy_primitives::B256;
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
    #[error("Multisig is under the required threshold. got: {got}, expected: {expected}")]
    UnderThreshold { got: usize, expected: usize },

    #[error("Signature {idx} is invalid or from an unregistered signer.")]
    HasInvalidSignature { idx: usize },

    #[error("Out of bounds signer index: signer index: {idx}, committee size: {total}.")]
    OutOfBoundSignerIndex { idx: usize, total: usize },
}

impl MultiSignature {
    /// Commitment on the signers and threshold.
    pub fn signers_commit(&self) -> Digest {
        // addresses does not need to be padded in 32bytes because
        // addresses won't be decoded in solidity, just encoded from the contract
        // storage.
        let mut buf = Vec::with_capacity(self.expected_signers.len());
        buf.extend_from_slice(self.threshold.to_be_bytes().as_slice());
        for addr in &self.expected_signers {
            buf.extend_from_slice(addr.as_slice()); // address (20bytes)
        }
        keccak256_combine([buf.as_slice()])
    }

    /// Verify signatures and ensure they are all from the expected set.
    pub fn verify(&self, commitment: B256) -> Result<(), MultisigError> {
        if self.signatures.len() > self.expected_signers.len() {
            return Err(MultisigError::OutOfBoundSignerIndex {
                idx: self.signatures.len() - 1,
                total: self.expected_signers.len(),
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
            let Ok(recovered) = signature.recover_address_from_prehash(&commitment) else {
                return Err(MultisigError::HasInvalidSignature { idx }); // Failed to recover from prehash
            };
            let expected = self.expected_signers[idx];
            if recovered != expected {
                return Err(MultisigError::HasInvalidSignature { idx }); // Signer doesn't match
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
    #[case(vec![false, false, true], 1, Err(MultisigError::OutOfBoundSignerIndex { idx: 2, total: 2 }))]
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
