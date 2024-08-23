use std::borrow::Borrow;

use serde::{Deserialize, Serialize};

use crate::{
    bridge_exit::{BridgeExit, NetworkId},
    global_index::GlobalIndex,
    keccak::{keccak256_combine, Digest as KeccakDigest, Digest},
    local_exit_tree::{data::LETMerkleProof, hasher::Keccak256Hasher},
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

    /// Verifies that the provided inclusion path is valid and consistent with the provided LER
    pub fn verify_path(&self) -> bool {
        let leaf_inclusion = self.inclusion_proof.verify(
            self.bridge_exit.hash(),
            self.global_index.leaf_index,
            self.imported_local_exit_root,
        );

        self.inclusion_proof_rer.as_ref().map_or(
            leaf_inclusion,
            |(rollup_inclusion_proof, rollup_exit_root)| {
                leaf_inclusion
                    && rollup_inclusion_proof.verify(
                        self.imported_local_exit_root,
                        self.global_index.rollup_index,
                        *rollup_exit_root,
                    )
            },
        )
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
