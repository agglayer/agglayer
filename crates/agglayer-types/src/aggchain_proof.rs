use agglayer_primitives::Signature;
use serde::{Deserialize, Serialize};
use sp1_core_machine::reduce::SP1ReduceProof;
use sp1_prover::InnerSC;

use crate::Digest;

// Aggchain data submitted via the [`Certificate`].
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum AggchainData {
    ECDSA {
        signature: Signature,
    },
    Generic {
        /// proof of the aggchain proof.
        proof: Proof,
        /// Chain-specific commitment forwarded through the PP.
        aggchain_params: Digest,
    },
}

pub type SP1StarkProof = SP1ReduceProof<InnerSC>;

/// Proof that is part of the aggchain proof submitted via the [`Certificate`].
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Proof {
    SP1Stark(Box<SP1StarkProof>),
}
