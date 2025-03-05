use serde::{Deserialize, Serialize};

use super::NetworkState;
use crate::{
    bridge_exit::NetworkId,
    keccak::{digest::Digest, keccak256_combine},
    local_exit_tree::hasher::Keccak256Hasher,
    multi_batch_header::MultiBatchHeader,
    ProofError,
};

/// The state commitment of one [`super::NetworkState`].
#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct StateCommitment {
    pub exit_root: Digest,
    pub ler_leaf_count: u32,
    pub balance_root: Digest,
    pub nullifier_root: Digest,
    pub height: u64,
    pub origin_network: NetworkId,
}

impl StateCommitment {
    pub fn new(
        state: &NetworkState,
        multi_batch_header: &MultiBatchHeader<Keccak256Hasher>,
    ) -> Self {
        Self {
            exit_root: state.exit_tree.get_root(),
            ler_leaf_count: state.exit_tree.leaf_count,
            balance_root: state.balance_tree.root,
            nullifier_root: state.nullifier_tree.root,
            height: multi_batch_header.height,
            origin_network: multi_batch_header.origin_network,
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum PPRootVersion {
    V2,
    V3,
}

pub struct SignatureCommitment {
    pub(crate) new_local_exit_root: Digest,
    pub(crate) commit_imported_bridge_exits: Digest,
    pub(crate) height: u64,
}

impl SignatureCommitment {
    /// Returns the expected signed commitment for the given version.
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

impl StateCommitment {
    pub fn infer_pp_root_version(&self, pp_root: Digest) -> Result<PPRootVersion, ProofError> {
        // NOTE: Return v2 to trigger the migration
        if pp_root.0 == [0u8; 32] {
            return Ok(PPRootVersion::V2);
        }

        let computed_v3 = self.compute_pp_root(PPRootVersion::V3);
        if computed_v3 == pp_root {
            return Ok(PPRootVersion::V3);
        }

        let computed_v2 = self.compute_pp_root(PPRootVersion::V2);
        if computed_v2 == pp_root {
            return Ok(PPRootVersion::V2);
        }

        Err(ProofError::InvalidPreviousPessimisticRoot {
            declared: pp_root,
            computed_v2,
            computed_v3,
        })
    }

    /// Returns the pessimistic root for the given version.
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

    pub fn compare(
        declared: &StateCommitment,
        computed: &StateCommitment,
    ) -> Result<(), ProofError> {
        if computed.exit_root != declared.exit_root {
            return Err(ProofError::InvalidNewLocalExitRoot {
                declared: declared.exit_root,
                computed: computed.exit_root,
            });
        }

        if computed.balance_root != declared.balance_root {
            return Err(ProofError::InvalidNewBalanceRoot {
                declared: declared.balance_root,
                computed: computed.balance_root,
            });
        }

        if computed.nullifier_root != declared.nullifier_root {
            return Err(ProofError::InvalidNewNullifierRoot {
                declared: declared.nullifier_root,
                computed: computed.nullifier_root,
            });
        }

        if computed.ler_leaf_count != declared.ler_leaf_count {
            return Err(ProofError::InvalidNewLocalExitRootLeafCount {
                declared: declared.ler_leaf_count,
                computed: computed.ler_leaf_count,
            });
        }

        // TODO: compare new fields

        Ok(())
    }
}
