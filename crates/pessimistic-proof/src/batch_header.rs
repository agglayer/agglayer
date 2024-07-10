use serde::{Deserialize, Serialize};

use crate::{bridge_exit::NetworkId, keccak::Digest, BridgeExit};

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
    pub origin_network: NetworkId,

    /// The start slot height and end block height for the batch. Unsure if required
    /// pub start_block_height: u32,
    /// pub end_block_height u32,

    /// The state root produced by the current block
    pub state_root: Option<Digest>,

    /// The initial local exit root.
    pub prev_local_exit_root: Option<Digest>,
    /// The updated local exit root
    pub local_exit_root: Digest,

    /// A commitment to the set of imported bridge exits for which the origin network is the target.
    pub imported_exits_root: Option<Digest>,

    /// A commitment to the set of imported local exit roots
    pub imported_lers_root: Option<Digest>,

    /// An imported global exit root, used to process deposits from Ethereum
    pub imported_global_exit_root: Option<Digest>,

    /// A validity proof verifying transaction execution
    ///pub validity_proof: Option<ValidityProof>,

    /// A consensus proof for the latest block
    ///pub consensus_proof: Option<ConsensusProof>,

    /// The signature that commits to the state transition.
    ///pub signature: (),
}

impl BatchHeader {
    /// Creates a new [`BatchHeader`].
    pub fn new(
        origin_network: NetworkId,
        state_root: Option<Digest>,
        prev_local_exit_root: Option<Digest>,
        local_exit_root: Digest,
        imported_exits_root: Option<Digest>,
        imported_lers_root: Option<Digest>,
        imported_global_exit_root: Option<Digest>,
    ) -> Self {
        Self {
            origin_network,
            state_root,
            prev_local_exit_root,
            local_exit_root,
            imported_exits_root,
            imported_lers_root,
            imported_global_exit_root,
        }
    }
}
