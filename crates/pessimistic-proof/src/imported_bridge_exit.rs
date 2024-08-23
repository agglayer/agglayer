use std::borrow::Borrow;

use serde::{Deserialize, Serialize};

use crate::{
    bridge_exit::BridgeExit,
    global_index::GlobalIndex,
    keccak::{keccak256_combine, Digest as KeccakDigest, Digest},
    local_exit_tree::{data::LETMerkleProof, hasher::Keccak256Hasher},
    ProofError,
};

/// Represents a token bridge exit originating on another network but claimed on the current network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportedBridgeExit {
    /// The bridge exit initiated on another network, called the "sending" network.
    /// Need to verify that the destination network matches the current network, and that
    /// the bridge exit is included in an imported LER
    pub bridge_exit: BridgeExit,
    /// The Imported Local Exit Root for the Local Exit Tree containing this bridge exit.
    pub imported_local_exit_root: Digest,
    /// The inclusion proof of the imported bridge exit in the sending local exit root.
    pub inclusion_proof: LETMerkleProof<Keccak256Hasher>,
    /// The inclusion proof of the LER to the Rollup Exit Root and the Rollup Exit Root.
    pub inclusion_proof_rer: Option<(LETMerkleProof<Keccak256Hasher>, Digest)>,
    /// The global index of the imported bridge exit.
    pub global_index: GlobalIndex,
}

impl ImportedBridgeExit {
    /// Creates a new [`ImportedBridgeExit`].
    pub fn new(
        bridge_exit: BridgeExit,
        imported_local_exit_root: KeccakDigest,
        global_index: GlobalIndex,
        inclusion_proof: LETMerkleProof<Keccak256Hasher>,
        inclusion_proof_rer: Option<(LETMerkleProof<Keccak256Hasher>, KeccakDigest)>,
    ) -> Self {
        Self {
            bridge_exit,
            global_index,
            inclusion_proof,
            inclusion_proof_rer,
            imported_local_exit_root,
        }
    }

    pub fn verify_leaf_inclusion(&self) -> Result<(), ProofError> {
        // Check the inclusion proof of the leaf to the LER
        if !self.inclusion_proof.verify(
            self.bridge_exit.hash(),
            self.global_index.leaf_index,
            self.imported_local_exit_root,
        ) {
            return Err(ProofError::InvalidImportedBridgeExitMerklePath);
        }

        Ok(())
    }

    pub fn verify_path_mainnet(&self, mer: Digest) -> Result<(), ProofError> {
        // Check that the inclusion proof is against the considered mainnet exit root
        if self.imported_local_exit_root != mer {
            return Err(ProofError::InvalidImportedBridgeExitRoot);
        }

        self.verify_leaf_inclusion()
    }

    pub fn verify_path_rollup(&self, rer: Digest) -> Result<(), ProofError> {
        let (rollup_inclusion_proof, rollup_exit_root) = self
            .inclusion_proof_rer
            .as_ref()
            .ok_or(ProofError::InvalidImportedBridgeExitMerklePath)?;

        // Check that the inclusion proof is against the considered rollup exit root
        if *rollup_exit_root != rer {
            return Err(ProofError::InvalidImportedBridgeExitRoot);
        }

        // Check the inclusion proof of the LER to RER
        if !rollup_inclusion_proof.verify(
            self.imported_local_exit_root,
            self.global_index.rollup_index,
            *rollup_exit_root,
        ) {
            return Err(ProofError::InvalidImportedBridgeExitMerklePath);
        }

        self.verify_leaf_inclusion()
    }

    /// Verifies that the provided inclusion path is valid and consistent with the provided LER
    pub fn verify_path(&self, mer: Digest, rer: Digest) -> Result<(), ProofError> {
        // Check that the inclusion proof and the global index both refer to mainnet or rollup
        if self.global_index.mainnet_flag != self.inclusion_proof_rer.is_none() {
            return Err(ProofError::MismatchGlobalIndexInclusionProof);
        }

        if self.global_index.mainnet_flag {
            self.verify_path_mainnet(mer)
        } else {
            self.verify_path_rollup(rer)
        }
    }

    pub fn hash(&self) -> Digest {
        self.bridge_exit.hash()
    }
}

pub fn commit_imported_bridge_exits<E: Borrow<ImportedBridgeExit>>(
    iter: impl Iterator<Item = E>,
) -> Digest {
    keccak256_combine(iter.map(|exit| exit.borrow().hash()))
}
