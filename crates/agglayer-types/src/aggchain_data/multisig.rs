use std::collections::HashMap;

use agglayer_primitives::{Address, Digest, Signature, B256};
use pessimistic_proof::core::{self};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::aggchain_data::PayloadWithCtx;

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

#[derive(Clone, Debug, Error, Deserialize, Serialize, PartialEq, Eq)]
pub enum MultisigError {
    #[error("Multisig is under the required threshold. got: {got}, expected: {expected}")]
    UnderThreshold { got: usize, expected: usize },

    #[error("Multisig contains at least one invalid signature.")]
    InvalidSignature,

    #[error("Unknown signer or invalid prehash. prehash: {prehash:?}, recovered: {recovered} ")]
    UnknownRecoveredSigner { recovered: Address, prehash: Digest },
}

// Generate the prover inputs from the chain payload and the L1 context.
impl TryInto<core::MultiSignature> for PayloadWithCtx<Payload, Ctx> {
    type Error = MultisigError;

    fn try_into(self) -> Result<core::MultiSignature, Self::Error> {
        let PayloadWithCtx(
            Payload { signatures },
            Ctx {
                signers,
                threshold,
                prehash,
            },
        ) = self;

        if signatures.len() < threshold {
            return Err(MultisigError::UnderThreshold {
                got: signatures.len(),
                expected: threshold,
            });
        }

        let index: HashMap<[u8; 20], usize> = signers
            .iter()
            .enumerate()
            .map(|(i, s)| (s.into_array(), i))
            .collect();

        let signatures = signatures
            .into_iter()
            .map(|sig| {
                let recovered = sig
                    .recover_address_from_prehash(&prehash)
                    .map_err(|_| MultisigError::InvalidSignature)?;
                let i = *index.get(&recovered.into_array()).ok_or(
                    MultisigError::UnknownRecoveredSigner {
                        recovered,
                        prehash: prehash.into(),
                    },
                )?;
                Ok::<_, Self::Error>((i, sig))
            })
            .collect::<Result<_, _>>()?;

        Ok(core::MultiSignature {
            signatures,
            expected_signers: signers,
            threshold,
        })
    }
}
