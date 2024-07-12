use std::ops::Deref;

use reth_primitives::{revm_primitives::bitvec::view::BitViewSized, Address, U256};
use serde::{Deserialize, Serialize};

use crate::keccak::{keccak256, keccak256_combine, Digest as KeccakDigest};

use crate::{
    BridgeExit,
    bridge_exit::{NetworkId},
};


/// Represents a token bridge exit originating on another network but claimed on the current network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportedBridgeExit {
    /// The bridge exit initiated on another network, called the "foreign" network.
    /// Need to verify that the destination network matches the current network, and that
    /// the bridge exit is included in an imported LER
    pub bridge_exit:BridgeExit,

    /// The Network ID for the foreign network. May not be strictly
    /// necessary depending on the final structure of the Imported Local Exit Root
    pub sending_network:NetworkId,

    /// The Imported Local Exit Root for the Local Exit Tree containing this bridge exit.
    pub imported_local_exit_root:KeccakDigest,

    /// The index of the bridge exit in the foreign local exit tree. Used to compute the path
    pub leaf_index: u32,

    /// The inclusion proof of the imported bridge exit in the foreign local exit root
    /// TODO: create a type for the inclusion proof of a leaf in an LET
    pub inclusion_proof:Vec<KeccakDigest>
}

impl ImportedBridgeExit {
    /// Creates a new [`ImportedBridgeExit`].
    pub fn new(
        bridge_exit: BridgeExit,
        sending_network: NetworkId,
        imported_local_exit_root:KeccakDigest,
        leaf_index: u32,
        inclusion_proof:Vec<KeccakDigest>,
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
    /// TODO: write method to verify inclusion proof in provided foreign LER
    pub fn verify_path(&mut self) -> bool {
        return true;
    }
}


