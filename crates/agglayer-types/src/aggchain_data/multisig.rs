use agglayer_primitives::{Address, Digest, Signature, B256};
use pessimistic_proof::core;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::aggchain_data::PayloadWithCtx;

#[derive(Clone, Debug, Error, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename = "agglayer_types::aggchain_data::MultisigError")]
pub enum MultisigError {
    #[error("Multisig is under the required threshold. got: {got}, expected: {expected}")]
    UnderThreshold { got: usize, expected: usize },

    #[error("Multisig contains at least one invalid signature.")]
    InvalidSignature,

    #[error("Unknown signer or invalid prehash. prehash: {prehash:?}, recovered: {recovered}")]
    UnknownRecoveredSigner { recovered: Address, prehash: Digest },

    #[error("At least one signer comes duplicated. signer: {signer}.")]
    HasDuplicateSigner { signer: Address },
}

/// Multisig data from the chain.
#[derive(Clone, Debug)]
pub struct Payload {
    signatures: Vec<Signature>,
}

impl From<Vec<Signature>> for Payload {
    fn from(signatures: Vec<Signature>) -> Self {
        Self { signatures }
    }
}

/// Multisig data from the L1 and enforced by the agglayer.
#[derive(Clone, Debug)]
pub struct Ctx {
    /// Ordered list of all possible signers.
    pub signers: Vec<Address>,
    /// Inclusive threshold.
    pub threshold: usize,
    /// Prehash expected to be signed.
    pub prehash: B256,
}

impl Ctx {
    /// Returns the index of the signer from the provided signature.
    fn signer_from_signature(&self, signature: &Signature) -> Result<usize, MultisigError> {
        let recovered = signature
            .recover_address_from_prehash(&self.prehash)
            .map_err(|_| MultisigError::InvalidSignature)?;

        self.signers.iter().position(|s| s == &recovered).ok_or(
            MultisigError::UnknownRecoveredSigner {
                recovered,
                prehash: self.prehash.into(),
            },
        )
    }
}

// Generate the prover inputs from the chain payload and the L1 context.
impl TryInto<core::MultiSignature> for PayloadWithCtx<Payload, Ctx> {
    type Error = MultisigError;

    fn try_into(self) -> Result<core::MultiSignature, Self::Error> {
        let PayloadWithCtx(Payload { signatures }, multisig) = self;

        let mut seen = vec![false; multisig.signers.len()];

        let indexed_signatures: Vec<(usize, Signature)> = signatures
            .into_iter()
            .map(|sig| {
                let idx = multisig.signer_from_signature(&sig)?;
                if seen[idx] {
                    return Err(MultisigError::HasDuplicateSigner {
                        signer: multisig.signers[idx],
                    });
                }
                seen[idx] = true;
                Ok::<_, MultisigError>((idx, sig))
            })
            .collect::<Result<_, _>>()?;

        if indexed_signatures.len() < multisig.threshold {
            return Err(MultisigError::UnderThreshold {
                got: indexed_signatures.len(),
                expected: multisig.threshold,
            });
        }

        Ok(core::MultiSignature {
            signatures: indexed_signatures,
            expected_signers: multisig.signers,
            threshold: multisig.threshold,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

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
        let seed = keccak256(&i.to_be_bytes());
        PrivateKeySigner::from_slice(seed.as_slice()).unwrap()
    }

    fn signatures_from_indices(order: &[usize], prehash: &B256) -> Vec<Signature> {
        order
            .iter()
            .map(|&i| wallet(i).sign_hash_sync(prehash).unwrap().into())
            .collect()
    }

    #[rstest]
    #[case(vec![0, 1], 2, Ok(()))]
    #[case(vec![0], 2, Err(MultisigError::UnderThreshold { got: 1, expected: 2 }))]
    #[case(
        vec![2], // registered only 2 wallets, so idx 2 (3rd signer) is unknown
        1,
        Err(MultisigError::UnknownRecoveredSigner {
            recovered: wallet(2).address().into(),
            prehash: prehash().into(),
        })
    )]
    #[case(
        vec![0, 0],
        2,
        Err(MultisigError::HasDuplicateSigner {
            signer: wallet(0).address().into(),
        })
    )]
    fn multisig_cases(
        #[case] signer_indices: Vec<usize>,
        #[case] threshold: usize,
        #[case] expected: Result<(), MultisigError>,
    ) {
        let wallets: Vec<PrivateKeySigner> = (0..2).map(wallet).collect();
        let prehash = prehash();

        let signers = wallets.iter().map(|w| w.address().into()).collect();
        let signatures = signatures_from_indices(&signer_indices, &prehash);

        let payload_with_ctx = PayloadWithCtx(
            Payload::from(signatures),
            Ctx {
                signers,
                threshold,
                prehash,
            },
        );

        let res: Result<core::MultiSignature, MultisigError> = payload_with_ctx.try_into();

        assert_eq!(res.map(|_| ()), expected);
    }
}
