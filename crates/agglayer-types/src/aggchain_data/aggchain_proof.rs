use agglayer_interop_types::aggchain_proof::Proof;
use agglayer_primitives::Digest;
use agglayer_sp1::{ProofError, ProofExt as _};
use pessimistic_proof::core;
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
    pub fn aggchain_vkey_hash_bytes(&self) -> Result<[u8; 32], ProofError> {
        self.proof.vkey_hash_bytes()
    }

    pub fn aggchain_vkey_hash_u32(&self) -> Result<[u32; 8], ProofError> {
        self.proof.vkey_hash_u32()
    }

    pub fn aggchain_vkey_hashes(&self) -> Result<([u8; 32], [u32; 8]), ProofError> {
        self.proof.vkey_hashes()
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

#[cfg(test)]
mod tests {
    use super::Payload;
    use crate::{certificate::dummy_sp1_stark_proof_with_version, Digest};

    #[test]
    fn aggchain_vkey_hashes_returns_consistent_hash_formats() {
        let payload = Payload {
            proof: dummy_sp1_stark_proof_with_version("v5.2.2"),
            aggchain_params: Digest::default(),
            public_values: None,
        };

        let (bytes, words) = payload.aggchain_vkey_hashes().unwrap();

        assert_eq!(bytes, payload.aggchain_vkey_hash_bytes().unwrap());
        assert_eq!(words, payload.aggchain_vkey_hash_u32().unwrap());
    }
}
