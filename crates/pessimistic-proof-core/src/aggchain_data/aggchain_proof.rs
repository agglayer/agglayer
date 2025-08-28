use agglayer_primitives::Digest;
use serde::{Deserialize, Serialize};
use unified_bridge::AggchainProofPublicValues;

use crate::{
    aggchain_data::Vkey,
    proof::{ConstrainedValues, IMPORTED_BRIDGE_EXIT_COMMITMENT_VERSION},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AggchainProof {
    /// Chain-specific commitment forwarded by the PP.
    pub aggchain_params: Digest,
    /// Verifying key for the aggchain proof program.
    pub aggchain_vkey: Vkey,
}

impl AggchainProof {
    /// Verifies the next proof in the sp1 buffer.
    pub fn verify_aggchain_proof(&self, constrained_values: &ConstrainedValues) {
        let aggchain_proof_public_values = AggchainProofPublicValues {
            prev_local_exit_root: constrained_values.initial_state_commitment.exit_root.into(),
            new_local_exit_root: constrained_values.final_state_commitment.exit_root.into(),
            l1_info_root: constrained_values.l1_info_root,
            origin_network: constrained_values.origin_network,
            aggchain_params: self.aggchain_params,
            commit_imported_bridge_exits: constrained_values
                .commit_imported_bridge_exits
                .commitment(IMPORTED_BRIDGE_EXIT_COMMITMENT_VERSION),
        };

        // Panic upon invalid proof.
        sp1_zkvm::lib::verify::verify_sp1_proof(
            &self.aggchain_vkey,
            &aggchain_proof_public_values.hash(),
        );
    }
}
