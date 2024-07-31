use serde::{Deserialize, Serialize};

use crate::{
    bridge_exit::{BridgeExit, NetworkId},
    keccak::{Digest as KeccakDigest, Digest},
    local_exit_tree::{data::LETMerkleProof, hasher::Keccak256Hasher},
};

/// Represents a token bridge exit originating on another network but claimed on the current network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportedBridgeExit {
    /// The bridge exit initiated on another network, called the "sending" network.
    /// Need to verify that the destination network matches the current network, and that
    /// the bridge exit is included in an imported LER
    pub bridge_exit: BridgeExit,

    /// The Network ID for the sending network. May not be strictly
    /// necessary depending on the final structure of the Imported Local Exit Root
    pub sending_network: NetworkId,

    /// The Imported Local Exit Root for the Local Exit Tree containing this bridge exit.
    pub imported_local_exit_root: Digest,

    /// The index of the bridge exit in the sending local exit tree. Used to compute the path
    pub leaf_index: u32,

    /// The inclusion proof of the imported bridge exit in the sending local exit root
    pub inclusion_proof: LETMerkleProof<Keccak256Hasher>,
}

impl ImportedBridgeExit {
    /// Creates a new [`ImportedBridgeExit`].
    pub fn new(
        bridge_exit: BridgeExit,
        sending_network: NetworkId,
        imported_local_exit_root: KeccakDigest,
        leaf_index: u32,
        inclusion_proof: LETMerkleProof<Keccak256Hasher>,
    ) -> Self {
        Self {
            bridge_exit,
            sending_network,
            imported_local_exit_root,
            leaf_index,
            inclusion_proof,
        }
    }

    /// Verifies that the provided inclusion path is valid and consistent with the provided LER
    pub fn verify_path(&self) -> bool {
        self.inclusion_proof.verify(
            self.bridge_exit.hash(),
            self.leaf_index,
            self.imported_local_exit_root,
        )
    }
}
