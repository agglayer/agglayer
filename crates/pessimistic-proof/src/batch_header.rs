use serde::{Deserialize, Serialize};

use crate::{bridge_exit::NetworkId, keccak::Digest, BridgeExit, ImportedBridgeExit, local_balance_tree::BalancePath};
use crate::nullifier_tree::{NullifierPath, NetworkNullifierPath};

/// Represents the data submitted by the CDKs to the AggLayer.
///
/// The bridge exits plus the imported bridge exits define
/// the state transition, resp. the amount that goes out and the amount that comes in.
///
/// The bridge exits refer to the [`BridgeExit`]  emitted by
/// the origin network of the [`BatchHeader`].
///
/// The imported bridge exits refer to the [`BridgeExit`] received and imported
/// by the origin network of the [`BatchHeader].
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct BatchHeader {
    /// The origin network which emitted this BatchHeader.
    /// TODO: we should clarify naming. We use origin to refer to the issuing network of a token. Could consider using "sending" or "local" to refer to the network that created a batch.
    pub origin_network: NetworkId,

    // /// The start slot height and end block height for the batch. Unsure if required
    // pub start_block_height: u32,
    // pub end_block_height u32,

    // /// The state root produced by the current block
    // TODO: Determine if this is necessary
    // pub state_root: Option<Digest>,

    /// The initial local exit root.
    pub prev_local_exit_root: Digest,

    /// The set of bridge exits created in this batch
    /// TODO: move out of the header and into a separate struct
    pub bridge_exits: Vec<BridgeExit>,

    /// The set of imported bridge exits claimed in this batch
    /// TODO: move out of the header and into a separate struct
    pub imported_bridge_exits: Option<Vec<ImportedBridgeExit>>,

    /// A commitment to the set of imported bridge exits for which the origin network is the target.
    pub imported_exits_root: Option<Digest>,

    /// The set of imported local exit roots
    pub imported_local_exit_roots: Option<Vec<(NetworkId, Digest)>>,

    /// The previous Local Balance Root
    pub prev_balance_root: Digest,

    /// LBT paths used to verify that token balances are non-zero
    /// TODO: move out of the header and into a separate struct
    pub balance_paths: Option<Vec<BalancePath>>,

    /// The previous NullifierTree Root
    pub prev_nullifier_root: Digest,

    /// Inclusion proofs for the previous state of each NetworkNullifierTree
    /// TODO: move out of the header and into a separate struct
    pub nullifier_paths: Option<Vec<NullifierPath>>,

    /// Inclusion proofs within each NetworkNullifierTree for a given leaf_index
    /// TODO: move out of the header and into a separate struct
    pub network_nullifier_paths: Option<Vec<NetworkNullifierPath>>,

    // /// A validity proof verifying transaction execution
    //pub validity_proof: Option<ValidityProof>,

    // /// A consensus proof for the latest block
    //pub consensus_proof: Option<ConsensusProof>,

    // /// The signature that commits to the state transition.
    //pub signature: (),
}

impl BatchHeader {
    /// Creates a new [`BatchHeader`].
    pub fn new(
        origin_network: NetworkId,
        prev_local_exit_root: Digest,
        bridge_exits: Vec<BridgeExit>,
        imported_bridge_exits: Option<Vec<ImportedBridgeExit>>,
        imported_exits_root: Option<Digest>,
        imported_local_exit_roots: Option<Vec<(NetworkId, Digest)>>,
        prev_balance_root: Digest,
        balance_paths: Option<Vec<BalancePath>>,
        prev_nullifier_root: Digest,
        nullifier_paths: Option<Vec<NullifierPath>>,
        network_nullifier_paths: Option<Vec<NetworkNullifierPath>>,

    ) -> Self {
        Self {
            origin_network,
            prev_local_exit_root,
            bridge_exits,
            imported_bridge_exits,
            imported_exits_root,
            imported_local_exit_roots,
            prev_balance_root,
            balance_paths: balance_paths,
            prev_nullifier_root,
            nullifier_paths,
            network_nullifier_paths
        }
    }
}
