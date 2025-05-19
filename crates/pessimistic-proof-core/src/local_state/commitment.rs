//! State related commitments involved in the pessimistic proof.
//!
//! The pessimistic proof has the "pessimistic root" as part of its public
//! inputs. Some logic in this file handles the migration on its computation.
use agglayer_primitives::digest::Digest;
use agglayer_primitives::keccak::keccak256_combine;
use serde::{Deserialize, Serialize};
use unified_bridge::imported_bridge_exit::ImportedBridgeExitCommitmentValues;
use unified_bridge::{bridge_exit::NetworkId, CommitmentVersion};

use crate::{proof::EMPTY_PP_ROOT_V2, ProofError};

/// The state commitment of one [`super::NetworkState`].
#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct StateCommitment {
    pub exit_root: Digest,
    pub ler_leaf_count: u32,
    pub balance_root: Digest,
    pub nullifier_root: Digest,
}

/// The parameters which compose the pessimistic root.
#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PessimisticRoot {
    pub balance_root: Digest,
    pub nullifier_root: Digest,
    pub ler_leaf_count: u32,
    pub height: u64,
    pub origin_network: NetworkId,
}

impl PessimisticRoot {
    /// Infer the version of the provided settled pessimistic root.
    pub fn infer_settled_pp_root_version(
        &self,
        settled_pp_root: Digest,
    ) -> Result<CommitmentVersion, ProofError> {
        info!("Inferring PP root with: {:?}", self);
        let computed_v3 = self.compute_pp_root(CommitmentVersion::V3);
        if computed_v3 == settled_pp_root {
            return Ok(CommitmentVersion::V3);
        }

        let computed_v2 = self.compute_pp_root(CommitmentVersion::V2);
        if computed_v2 == settled_pp_root {
            return Ok(CommitmentVersion::V2);
        }

        // NOTE: Return v2 to trigger the migration
        let is_initial_state = computed_v2 == EMPTY_PP_ROOT_V2 && self.height == 0;

        if settled_pp_root.0 == [0u8; 32] && is_initial_state {
            return Ok(CommitmentVersion::V2);
        }

        Err(ProofError::InvalidPreviousPessimisticRoot {
            declared: settled_pp_root,
            computed_v2,
            computed_v3,
        })
    }

    /// Compute the pessimistic root for the provided version.
    pub fn compute_pp_root(&self, version: CommitmentVersion) -> Digest {
        match version {
            CommitmentVersion::V2 => keccak256_combine([
                self.balance_root.as_slice(),
                self.nullifier_root.as_slice(),
                self.ler_leaf_count.to_le_bytes().as_slice(),
            ]),
            CommitmentVersion::V3 => keccak256_combine([
                self.balance_root.as_slice(),
                self.nullifier_root.as_slice(),
                self.ler_leaf_count.to_le_bytes().as_slice(),
                self.height.to_le_bytes().as_slice(),
                self.origin_network.to_le_bytes().as_slice(),
            ]),
        }
    }
}

/// The values which compose the signature.
pub struct SignatureCommitmentValues {
    pub new_local_exit_root: Digest,
    pub commit_imported_bridge_exits: ImportedBridgeExitCommitmentValues,
    pub height: u64,
}

impl SignatureCommitmentValues {
    /// Returns the expected signed commitment for the provided version.
    #[inline]
    pub fn commitment(&self, version: CommitmentVersion) -> Digest {
        let imported_bridge_exit_commitment = self.commit_imported_bridge_exits.commitment(version);
        match version {
            CommitmentVersion::V2 => keccak256_combine([
                self.new_local_exit_root.as_slice(),
                imported_bridge_exit_commitment.as_slice(),
            ]),
            CommitmentVersion::V3 => keccak256_combine([
                self.new_local_exit_root.as_slice(),
                imported_bridge_exit_commitment.as_slice(),
                self.height.to_le_bytes().as_slice(),
            ]),
        }
    }
}
