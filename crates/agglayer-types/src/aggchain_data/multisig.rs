use std::collections::HashMap;

use agglayer_primitives::{Address, Signature, B256};
use pessimistic_proof::core::{self};
use serde::{Deserialize, Serialize};

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

// Generate the prover inputs from the chain payload and the L1 context.
impl TryInto<core::MultiSignature> for PayloadWithCtx<Payload, Ctx> {
    type Error = ();

    fn try_into(self) -> Result<core::MultiSignature, Self::Error> {
        let PayloadWithCtx(
            Payload { signatures },
            Ctx {
                signers,
                threshold,
                prehash,
            },
        ) = self;

        let index: HashMap<[u8; 20], usize> = signers
            .iter()
            .enumerate()
            .map(|(i, s)| (s.into_array(), i))
            .collect();

        let signatures = signatures
            .into_iter()
            .map(|sig| {
                let signer = sig.recover_address_from_prehash(&prehash).map_err(|_| ())?;
                let i = *index.get(&signer.into_array()).ok_or(())?;
                Ok::<_, ()>((i, sig))
            })
            .collect::<Result<_, _>>()?;

        Ok(core::MultiSignature {
            signatures,
            expected_signers: signers,
            threshold,
        })
    }
}
