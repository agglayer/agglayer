use agglayer_primitives::Signature;
pub use pessimistic_proof_core::aggchain_proof::{
    AggchainProofData, AggchainProofECDSAData, AggchainProofPublicValues, AggchainProofSP1Data,
    AggchainType, Vkey,
};
use pessimistic_proof_core::keccak::digest::Digest;
use serde::{Deserialize, Serialize};
use sp1_core_machine::reduce::SP1ReduceProof;
use sp1_prover::InnerSC;

// Aggchain proof values submitted via the [`Certificate`].
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum AggchainProof {
    ECDSA { signature: Signature },
    SP1 { aggchain_proof: AggchainProofSP1 },
}

pub type StarkProof = SP1ReduceProof<InnerSC>;

/// SP1 variant of the aggchain proof values submitted via the [`Certificate`].
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AggchainProofSP1 {
    /// Chain-specific commitment forwarded through the PP.
    pub aggchain_params: Digest,
    /// STARK proof.
    pub stark_proof: Box<StarkProof>,
}
