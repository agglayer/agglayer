use serde::{Deserialize, Serialize};

use crate::{bridge_exit::NetworkId, keccak::Digest, BridgeExit};

/// Represents the data submitted by the CDKs to the AggLayer.
///
/// The bridge exits plus the imported bridge exits define
/// the state transition, resp. the amount that goes out and the amount that comes in.
///
/// The bridge exits refer to the [`BridgeExit`]  emitted by
/// the origin network of the [`Certificate`].
///
/// The imported bridge exits refer to the [`BridgeExit`] received and imported
/// by the origin network of the [`Certificate`].
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Certificate {
    /// The origin network which emitted this certificate.
    pub origin_network: NetworkId,
    /// The initial local exit root.
    pub prev_local_exit_root: Digest,
    /// The set of bridge exits emitted by the origin network.
    pub bridge_exits: Vec<BridgeExit>,
    /// The set of imported bridge exits for which the origin network is the target.
    pub imported_bridge_exits: Vec<BridgeExit>,
}

impl Certificate {
    /// Creates a new [`Certificate`].
    pub fn new(
        origin_network: NetworkId,
        prev_local_exit_root: Digest,
        bridge_exits: Vec<BridgeExit>,
    ) -> Self {
        Self {
            origin_network,
            prev_local_exit_root,
            bridge_exits,
            imported_bridge_exits: Default::default(),
        }
    }
}
