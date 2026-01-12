use agglayer_primitives::{Address, Signature, B256};
use pessimistic_proof::core;

use crate::aggchain_data::PayloadWithCtx;

/// Multisig data from the chain.
#[derive(Clone, Debug)]
pub struct Payload {
    pub(crate) signatures: Vec<Option<Signature>>,
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

impl From<agglayer_interop_types::aggchain_proof::MultisigPayload> for Payload {
    fn from(value: agglayer_interop_types::aggchain_proof::MultisigPayload) -> Self {
        Self {
            signatures: value.0,
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
impl From<PayloadWithCtx<Payload, Ctx>> for core::MultiSignature {
    fn from(d: PayloadWithCtx<Payload, Ctx>) -> core::MultiSignature {
        let PayloadWithCtx(Payload { signatures }, multisig) = d;

        core::MultiSignature {
            signatures,
            expected_signers: multisig.signers,
            threshold: multisig.threshold,
        }
    }
}
