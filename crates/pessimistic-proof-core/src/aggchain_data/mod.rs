//! Aggchain proof data structures.
//!
//! The aggchain-proof is the extra custom logic which is verified within the
//! pessimistic-proof.
//!
//! For now, this is constraint to be either multisig, or one SP1 stark proof
//! proving a specified statement which can be abstracted here.

use agglayer_primitives::Digest;
use serde::{Deserialize, Serialize};

pub use crate::aggchain_data::{
    aggchain_hash::AggchainHashValues,
    aggchain_proof::AggchainProof,
    multisig::{MultiSignature, MultisigError},
};
use crate::{
    local_state::commitment::{PessimisticRootCommitmentVersion, SignatureCommitmentValues},
    proof::ConstrainedValues,
    ProofError,
};

mod aggchain_hash;
mod aggchain_proof;
mod multisig;

pub type Vkey = [u32; 8];

/// Chain proof which include either multisig, aggchain proof, or both.
/// Explicit enum which forbid the case where we have none of them.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AggchainData {
    /// Multisig only
    MultisigOnly(MultiSignature),
    /// Multisig and an aggchain proof
    MultisigAndAggchainProof {
        /// Multisig
        multisig: MultiSignature,
        /// Aggchain proof
        aggchain_proof: AggchainProof,
    },
}

impl AggchainData {
    /// Returns the aggchain hash
    pub fn aggchain_hash(&self) -> Digest {
        AggchainHashValues::from(self).hash()
    }

    pub fn verify(
        &self,
        constrained_values: ConstrainedValues,
    ) -> Result<PessimisticRootCommitmentVersion, ProofError> {
        let prev_pp_root_version = constrained_values.prev_pessimistic_root_version;

        let target_pp_root_version = match self {
            AggchainData::MultisigOnly(multisig) => {
                let commitment =
                    SignatureCommitmentValues::new(&constrained_values, None).multisig_commitment();

                multisig
                    .verify(commitment)
                    .map_err(ProofError::InvalidMultisig)?;

                // Multisig is currently always on commitment v3
                PessimisticRootCommitmentVersion::V3
            }
            AggchainData::MultisigAndAggchainProof {
                multisig,
                aggchain_proof,
            } => {
                let commitment = SignatureCommitmentValues::new(
                    &constrained_values,
                    Some(aggchain_proof.aggchain_params), // signs the same used by the stark
                )
                .multisig_commitment();

                multisig
                    .verify(commitment)
                    .map_err(ProofError::InvalidMultisig)?;

                // Panic upon invalid proof.
                aggchain_proof.verify_aggchain_proof(&constrained_values);

                // Multisig is currently always on commitment v3
                PessimisticRootCommitmentVersion::V3
            }
        };

        match (prev_pp_root_version, target_pp_root_version) {
            // From V3 to V3: OK
            (PessimisticRootCommitmentVersion::V3, PessimisticRootCommitmentVersion::V3) => {}
            // From V2 to V3: OK (migration)
            (PessimisticRootCommitmentVersion::V2, PessimisticRootCommitmentVersion::V3) => {}
            // Inconsistent signed payload.
            _ => return Err(ProofError::InconsistentSignedPayload),
        }

        Ok(target_pp_root_version)
    }
}
