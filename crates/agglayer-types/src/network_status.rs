use agglayer_primitives::Digest;
use agglayer_tries::roots::LocalExitRoot;
use serde::{Deserialize, Serialize};
use unified_bridge::{GlobalIndex, NetworkId};

use crate::{CertificateId, Height};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SettledClaim {
    /// Global index, indicating uniquely which tree leaf is claimed.
    pub global_index: GlobalIndex,
    /// / Hash of the claimed imported bridge exit.
    pub bridge_exit_hash: Digest,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkStatus {
    /// The current status of the network (e.g., "active", "syncing", "error").
    pub network_status: String,
    /// The aggchain type of network
    pub network_type: String,
    /// The unique identifier for this network.
    pub network_id: NetworkId,
    /// The height of the latest settled certificate.
    pub settled_height: Height,
    /// The ID of the latest settled certificate.
    pub settled_certificate_id: CertificateId,
    /// The pessimistic proof root of the latest settled certificate.
    pub settled_pp_root: Digest,
    /// The local exit root of the latest settled certificate.
    pub settled_ler: LocalExitRoot,
    /// The leaf count of the latest settled local exit tree.
    pub settled_let_leaf_count: u64,
    /// Info about the latest settled claim in the network.
    pub settled_claim: SettledClaim,
    /// The height of the latest pending certificate.
    pub latest_pending_height: u64,
    /// The status of the latest pending certificate (e.g., "Proven", "Pending",
    /// "InError").
    pub latest_pending_status: String,
    /// Any error message associated with the latest pending certificate.
    pub latest_pending_error: String,
    /// The epoch number of the latest settlement.
    pub latest_epoch_with_settlement: u64,
}
