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
    /// necessary depending on the final structure of the Imprted Local Exit Root
    pub origin_network_id:NetworkId,

    /// The Imported Local Exit Root for the Local Exit Tree containing this bridge exit.
    pub imported_local_exit_root:KeccakDigest,

    /// The index of the bridge exit in the foreign local exit tree. Used to compute the path
    pub leaf_index: u32,

    /// The inclusion proof of the imported bridge exit in the foreign local exit root
    /// TODO: create a type for the inclusion proof of a leaf in an LET
    pub inclusion_proof:Vec<KeccakDigest>
}


