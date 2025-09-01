use agglayer_interop::types::NetworkId;
use agglayer_primitives::Digest;
use agglayer_tries::roots::LocalExitRoot;
use agglayer_types::{CertificateStatus, CertificateStatusError};
use serde::{Deserialize, Serialize};

use crate::{CertificateId, Height};

/// The status of a network.
/// TODO: implement more detailed status tracking including
/// separate service that would monitor status of the network.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum NetworkStatus {
    /// The network is active and functioning normally.
    Active = 0,
    /// The network is currently syncing.
    Syncing = 1,
    /// The network is experiencing an error.
    Error = 2,
}

// The aggchain type of network
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum NetworkType {
    /// ECDSA-based network type.
    Ecdsa = 0,
    /// Generic network type.
    Generic = 1,
    /// Multisig-only network type.
    MultisigOnly = 2,
    /// Multisig and aggchain proof network type.
    MultisigAndAggchainProof = 3,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SettledClaim {
    /// Global index, indicating uniquely which tree leaf is claimed.
    pub global_index: Digest,
    /// / Hash of the claimed imported bridge exit.
    pub bridge_exit_hash: Digest,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetworkState {
    /// The current status of the network (e.g., "active", "syncing", "error").
    pub network_status: NetworkStatus,
    /// The aggchain type of a network
    pub network_type: NetworkType,
    /// The unique identifier for this network.
    pub network_id: NetworkId,
    /// The height of the latest settled certificate.
    pub settled_height: Option<Height>,
    /// The ID of the latest settled certificate.
    pub settled_certificate_id: Option<CertificateId>,
    /// The pessimistic proof root of the latest settled certificate.
    pub settled_pp_root: Option<Digest>,
    /// The local exit root of the latest settled certificate.
    pub settled_ler: Option<LocalExitRoot>,
    /// The leaf count of the latest settled local exit tree.
    pub settled_let_leaf_count: Option<u64>,
    /// Info about the latest settled claim in the network.
    pub settled_claim: Option<SettledClaim>,
    /// The height of the latest pending certificate.
    pub latest_pending_height: Option<u64>,
    /// The status of the latest pending certificate.
    pub latest_pending_status: Option<CertificateStatus>,
    /// Any error message associated with the latest pending certificate.
    pub latest_pending_error: Option<CertificateStatusError>,
    /// The epoch number of the latest settlement.
    pub latest_epoch_with_settlement: Option<u64>,
}
