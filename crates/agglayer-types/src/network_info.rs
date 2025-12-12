use agglayer_interop_types::aggchain_proof;
use agglayer_primitives::Digest;
use agglayer_tries::roots::LocalExitRoot;
use serde::{Deserialize, Serialize};

use crate::{CertificateId, CertificateStatus, CertificateStatusError, Height, NetworkId};

/// The status of a network.
/// TODO: implement more detailed status tracking including
/// separate service that would monitor status of the network.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum NetworkStatus {
    /// Unknown status.
    Unknown = 0,
    /// The network is active and functioning normally.
    Active = 1,
    /// The network is currently syncing.
    Syncing = 2,
    /// The network is experiencing an error.
    Error = 3,
    /// The network is disabled.
    Disabled = 4,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum DisabledBy {
    /// Default value, should not be used.
    Unknown = 0,
    /// The network was disabled by an admin.
    Admin = 1,
}

// The aggchain type of network
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, strum_macros::EnumCount)]
pub enum NetworkType {
    Unspecified = 0,
    /// ECDSA-based network type.
    Ecdsa = 1,
    /// Generic network type.
    Generic = 2,
    /// Multisig-only network type.
    MultisigOnly = 3,
    /// Multisig and aggchain proof network type.
    MultisigAndAggchainProof = 4,
}

impl From<&aggchain_proof::AggchainData> for NetworkType {
    fn from(value: &aggchain_proof::AggchainData) -> Self {
        match value {
            aggchain_proof::AggchainData::ECDSA { .. } => NetworkType::Ecdsa,
            aggchain_proof::AggchainData::Generic { .. } => NetworkType::Generic,
            aggchain_proof::AggchainData::MultisigOnly { .. } => NetworkType::MultisigOnly,
            aggchain_proof::AggchainData::MultisigAndAggchainProof { .. } => {
                NetworkType::MultisigAndAggchainProof
            }
        }
    }
}

#[cfg(feature = "testutils")]
impl NetworkType {
    /// Generate a random NetworkType for testing using the provided seed.
    /// This function is resilient to changes in the enum variants.
    /// It excludes Unspecified as that shouldn't be used in normal test
    /// scenarios.
    pub fn generate_for_test(seed: u64) -> Self {
        use rand::{Rng, SeedableRng};
        use strum::EnumCount;
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

        // We have COUNT variants total, but exclude Unspecified (first variant)
        // So we generate a random index from 1 to COUNT-1
        match rng.random_range(1..Self::COUNT) {
            1 => NetworkType::Ecdsa,
            2 => NetworkType::Generic,
            3 => NetworkType::MultisigOnly,
            4 => NetworkType::MultisigAndAggchainProof,
            _ => unreachable!("Invalid network type index"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SettledClaim {
    /// Global index, indicating uniquely which tree leaf is claimed.
    pub global_index: Digest,
    /// / Hash of the claimed imported bridge exit.
    pub bridge_exit_hash: Digest,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetworkInfo {
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
    pub latest_pending_height: Option<Height>,
    /// The status of the latest pending certificate.
    pub latest_pending_status: Option<CertificateStatus>,
    /// Any error message associated with the latest pending certificate.
    pub latest_pending_error: Option<CertificateStatusError>,
    /// The epoch number of the latest settlement.
    pub latest_epoch_with_settlement: Option<u64>,
}

impl NetworkInfo {
    pub const fn from_network_id(network_id: NetworkId) -> Self {
        Self {
            network_status: NetworkStatus::Unknown,
            network_type: NetworkType::Unspecified,
            network_id,
            settled_height: None,
            settled_certificate_id: None,
            settled_pp_root: None,
            settled_ler: None,
            settled_let_leaf_count: None,
            settled_claim: None,
            latest_pending_height: None,
            latest_pending_status: None,
            latest_pending_error: None,
            latest_epoch_with_settlement: None,
        }
    }
}
