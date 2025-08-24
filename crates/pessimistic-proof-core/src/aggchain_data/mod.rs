//! Aggchain proof data structures.
//!
//! The aggchain-proof is the extra custom logic which is verified within the
//! pessimistic-proof.
//!
//! For now, this is constraint to be either one ECDSA signature, or one SP1
//! stark proof proving a specified statement which can be abstracted here.

use agglayer_primitives::{Address, Digest, Signature};
use alloy_primitives::B256;
use serde::{Deserialize, Serialize};

pub use crate::aggchain_data::{
    aggchain_hash::AggchainHashValues,
    aggchain_proof::AggchainProof,
    multisig::{MultiSignature, MultisigError},
};
use crate::{
    local_state::commitment::{
        PessimisticRootCommitmentValues, PessimisticRootCommitmentVersion,
        SignatureCommitmentValues, SignatureCommitmentVersion,
    },
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
    /// Legacy signature with migration logic
    LegacyEcdsa {
        /// Signer committing to the state transition.
        signer: Address,
        /// Signature committing to the state transition.
        signature: Signature,
    },
    /// Multisig only
    MultisigOnly(MultiSignature),
    /// Aggchain proof only (stark proof)
    AggchainProofOnly(AggchainProof),
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
        match self {
            AggchainData::LegacyEcdsa { signer, signature } => {
                // Only this case is subject to pp root migration concerns.
                return AggchainData::legacy_ecdsa(signer, signature, constrained_values);
            }
            AggchainData::MultisigOnly(multisig) => {
                let commitment =
                    SignatureCommitmentValues::new(&constrained_values, None).multisig_commitment();

                multisig
                    .verify(commitment)
                    .map_err(ProofError::InvalidMultisig)?;
            }
            AggchainData::AggchainProofOnly(aggchain_proof) => {
                // Panic upon invalid proof.
                aggchain_proof.verify_aggchain_proof(&constrained_values);
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
            }
        };

        Ok(PessimisticRootCommitmentVersion::V3)
    }
}

impl AggchainData {
    pub fn legacy_ecdsa(
        signer: &Address,
        signature: &Signature,
        constrained_values: ConstrainedValues,
    ) -> Result<PessimisticRootCommitmentVersion, ProofError> {
        let verify_signature = |prehash: B256, signature: &Signature| {
            signature
                .recover_address_from_prehash(&prehash)
                .map_err(|_| ProofError::InvalidSignature)
        };

        let signature_commitment = SignatureCommitmentValues::new(&constrained_values, None);

        let target_pp_root_version = {
            if *signer
                == verify_signature(
                    signature_commitment.commitment(SignatureCommitmentVersion::V3),
                    signature,
                )?
            {
                PessimisticRootCommitmentVersion::V3
            } else if *signer
                == verify_signature(
                    signature_commitment.commitment(SignatureCommitmentVersion::V2),
                    signature,
                )?
            {
                PessimisticRootCommitmentVersion::V2
            } else {
                return Err(ProofError::InvalidSignature);
            }
        };

        // Verify initial state commitment and PP root matches
        let base_pp_root_version = PessimisticRootCommitmentValues::from(&constrained_values)
            .infer_settled_pp_root_version(constrained_values.prev_pessimistic_root)?;

        match (base_pp_root_version, target_pp_root_version) {
            // From V2 to V2: OK
            (PessimisticRootCommitmentVersion::V2, PessimisticRootCommitmentVersion::V2) => {}
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
