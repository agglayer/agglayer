use agglayer_primitives::{Address, Digest, Signature, B256};
use pessimistic_proof::core;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::aggchain_data::PayloadWithCtx;

#[derive(Clone, Debug, Error, Deserialize, Serialize, PartialEq, Eq)]
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
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Payload {
    signatures: Vec<Signature>,
}

/// Multisig data from the L1 and enforced by the agglayer.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Ctx {
    /// Ordered list of all possible signers.
    pub signers: Vec<Address>,
    /// Inclusive threshold.
    pub threshold: usize,
    /// Prehash expected to be signed.
    pub prehash: B256,
}

impl Ctx {
    /// Returns the ide,
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

        let mut indexed_signatures: Vec<(usize, Signature)> = signatures
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

        indexed_signatures.sort_unstable_by_key(|(i, _)| *i); // contiguous access in the prover

        Ok(core::MultiSignature {
            signatures: indexed_signatures,
            expected_signers: multisig.signers,
            threshold: multisig.threshold,
        })
    }
}
