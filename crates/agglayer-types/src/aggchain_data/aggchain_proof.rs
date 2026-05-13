use agglayer_interop_types::aggchain_proof::Proof;
use agglayer_primitives::Digest;
use pessimistic_proof::core;
use sp1_sdk::SP1VerifyingKey;
use unified_bridge::AggchainProofPublicValues;

use crate::aggchain_data::PayloadWithCtx;

/// Aggchain proof with aggchain params and optional public values for debug
/// purposes.
#[derive(Clone, Debug)]
pub struct Payload {
    /// STARK of the aggchain proof.
    pub proof: Proof,
    /// Chain-specific commitment forwarded through the PP.
    pub aggchain_params: Digest,
    /// Optional aggchain proof public values.
    pub public_values: Option<Box<AggchainProofPublicValues>>,
}

impl Payload {
    pub fn aggchain_vkey_from_proof(&self) -> SP1VerifyingKey {
        let Proof::SP1Stark(sp1_reduce_proof) = &self.proof;
        sp1_reduce_proof.vkey.clone()
    }
}

/// Aggchain proof data from the L1 and enforced by the agglayer.
#[derive(Clone, Debug)]
pub struct Context {
    pub aggchain_vkey: [u32; 8],
}

impl From<PayloadWithCtx<Payload, Context>> for core::AggchainProof {
    fn from(val: PayloadWithCtx<Payload, Context>) -> Self {
        let PayloadWithCtx(
            Payload {
                aggchain_params, ..
            },
            Context { aggchain_vkey },
        ) = val;

        core::AggchainProof {
            aggchain_params,
            aggchain_vkey,
        }
    }
}
