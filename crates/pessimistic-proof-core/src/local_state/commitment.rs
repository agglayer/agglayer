//! State related commitments involved in the pessimistic proof.
//!
//! The pessimistic proof has the "pessimistic root" as part of its public
//! inputs. Some logic in this file handles the migration on its computation.
use serde::{Deserialize, Serialize};

use crate::{
    bridge_exit::NetworkId,
    keccak::{digest::Digest, keccak256_combine},
    proof::EMPTY_PP_ROOT_V2,
    ProofError,
};

/// The state commitment of one [`super::NetworkState`].
#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct StateCommitment {
    pub exit_root: Digest,
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
    ) -> Result<PPRootVersion, ProofError> {
        let computed_v3 = self.compute_pp_root(PPRootVersion::V3);
        if computed_v3 == settled_pp_root {
            return Ok(PPRootVersion::V3);
        }

        let computed_v2 = self.compute_pp_root(PPRootVersion::V2);
        if computed_v2 == settled_pp_root {
            return Ok(PPRootVersion::V2);
        }

        // NOTE: Return v2 to trigger the migration
        let is_initial_state = computed_v2 == EMPTY_PP_ROOT_V2 && self.height == 0;

        if settled_pp_root.0 == [0u8; 32] && is_initial_state {
            return Ok(PPRootVersion::V2);
        }

        Err(ProofError::InvalidPreviousPessimisticRoot {
            declared: settled_pp_root,
            computed_v2,
            computed_v3,
        })
    }

    /// Compute the pessimistic root for the provided version.
    pub fn compute_pp_root(&self, version: PPRootVersion) -> Digest {
        match version {
            PPRootVersion::V2 => keccak256_combine([
                self.balance_root.as_slice(),
                self.nullifier_root.as_slice(),
                self.ler_leaf_count.to_le_bytes().as_slice(),
            ]),
            PPRootVersion::V3 => keccak256_combine([
                self.balance_root.as_slice(),
                self.nullifier_root.as_slice(),
                self.ler_leaf_count.to_le_bytes().as_slice(),
                self.height.to_le_bytes().as_slice(),
                self.origin_network.to_le_bytes().as_slice(),
            ]),
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum PPRootVersion {
    V2,
    V3,
}

/// The parameters which composes the signature.
pub struct SignatureCommitment {
    pub new_local_exit_root: Digest,
    pub commit_imported_bridge_exits: Digest,
    pub height: u64,
}

impl SignatureCommitment {
    /// Returns the expected signed commitment for the provided version.
    pub fn commitment(&self, version: PPRootVersion) -> Digest {
        match version {
            PPRootVersion::V2 => keccak256_combine([
                self.new_local_exit_root.as_slice(),
                self.commit_imported_bridge_exits.as_slice(),
            ]),
            PPRootVersion::V3 => keccak256_combine([
                self.new_local_exit_root.as_slice(),
                self.commit_imported_bridge_exits.as_slice(),
                self.height.to_le_bytes().as_slice(),
            ]),
        }
    }
}
