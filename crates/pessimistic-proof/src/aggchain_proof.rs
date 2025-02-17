use agglayer_primitives::Signature;
pub use pessimistic_proof_core::aggchain_proof::{
    AggchainProofData, AggchainProofECDSAData, AggchainProofPublicValues, AggchainProofSP1Data,
    AggchainType, Vkey,
};
use pessimistic_proof_core::keccak::digest::Digest;
use serde::{Deserialize, Serialize};

// Aggchain proof values submitted via the [`Certificate`].
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum AggchainProof {
    ECDSA { signature: Signature },
    SP1 { aggchain_proof: AggchainProofSP1 },
}

// TODO: Replace with the proper format (fixed size buffer of ~7kb)
pub type StarkProof = [u8; 32];

/// SP1 variant of the aggchain proof values submitted via the [`Certificate`].
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AggchainProofSP1 {
    /// Chain-specific commitment forwarded through the PP.
    pub aggchain_params: Digest,
    /// STARK proof.
    pub stark_proof: StarkProof,
}
