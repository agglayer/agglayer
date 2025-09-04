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
}

/// Multisig data from the chain.
#[derive(Clone, Debug)]
pub struct Payload {
    signatures: Vec<Option<Signature>>,
}

impl From<Vec<Option<Signature>>> for Payload {
    fn from(signatures: Vec<Option<Signature>>) -> Self {
        Self { signatures }
    }
}

impl From<&[Option<Signature>]> for Payload {
    fn from(signatures: &[Option<Signature>]) -> Self {
        Self {
            signatures: signatures.to_vec(),
        }
    }
}

impl From<&agglayer_interop_types::aggchain_proof::MultisigPayload> for Payload {
    fn from(value: &agglayer_interop_types::aggchain_proof::MultisigPayload) -> Self {
        Self {
            signatures: value.0.clone(),
        }
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

// Generate the prover inputs from the chain payload and the L1 context.
impl TryInto<core::MultiSignature> for PayloadWithCtx<Payload, Ctx> {
    type Error = MultisigError;

    fn try_into(self) -> Result<core::MultiSignature, Self::Error> {
        let PayloadWithCtx(Payload { signatures }, multisig) = self;

        Ok(core::MultiSignature {
            signatures,
            expected_signers: multisig.signers,
            threshold: multisig.threshold,
        })
    }
}
