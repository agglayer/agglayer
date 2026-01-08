pub use std::io;

use crate::schema::ColumnSchema;

// State related CFs
pub const CERTIFICATE_PER_NETWORK_CF: &str = "certificate_per_network_cf";
pub const NULLIFIER_TREE_PER_NETWORK_CF: &str = "nullifier_tree_per_network_cf";
pub const BALANCE_TREE_PER_NETWORK_CF: &str = "balance_tree_per_network_cf";
pub const LOCAL_EXIT_TREE_PER_NETWORK_CF: &str = "local_exit_tree_per_network_cf";
pub const NETWORK_INFO_CF: &str = "network_info_cf";
pub const DISABLED_NETWORKS_CF: &str = "disabled_networks_cf";
pub const PP_ROOT_TO_CERTIFICATE_IDS_CF: &str = "pp_root_to_certificate_ids_cf";

// Metadata CFs
pub const CERTIFICATE_HEADER_CF: &str = "certificate_header_cf";
pub const LATEST_PROVEN_CERTIFICATE_PER_NETWORK_CF: &str =
    "latest_proven_certificate_per_network_cf";
pub const LATEST_SETTLED_CERTIFICATE_PER_NETWORK_CF: &str =
    "latest_settled_certificate_per_network_cf";
pub const LATEST_PENDING_CERTIFICATE_PER_NETWORK_CF: &str =
    "latest_pending_certificate_per_network_cf";
pub const METADATA_CF: &str = "metadata_cf";

// epochs related CFs
pub const PER_EPOCH_CERTIFICATES_CF: &str = "per_epoch_certificates_cf";
pub const PER_EPOCH_METADATA_CF: &str = "per_epoch_metadata_cf";
pub const PER_EPOCH_PROOFS_CF: &str = "per_epoch_proofs_cf";
pub const PER_EPOCH_END_CHECKPOINT_CF: &str = "per_epoch_end_checkpoint_cf";
pub const PER_EPOCH_START_CHECKPOINT_CF: &str = "per_epoch_start_checkpoint_cf";

// Pending related CFs
pub const PENDING_QUEUE_CF: &str = "pending_queue_cf";
pub const PROOF_PER_CERTIFICATE_CF: &str = "proof_per_certificate_cf";

// debug CFs
pub const DEBUG_CERTIFICATES_CF: &str = "debug_certificates";

// State
pub(crate) mod balance_tree_per_network;
pub(crate) mod certificate_per_network;
pub(crate) mod disabled_networks;
pub(crate) mod local_exit_tree_per_network;
pub(crate) mod network_info;
pub(crate) mod nullifier_tree_per_network;
pub(crate) mod pp_root_to_certificate_ids;

// Pending
pub(crate) mod pending_queue;
pub(crate) mod proof_per_certificate;

// Metadata
pub(crate) mod certificate_header;
pub mod latest_pending_certificate_per_network;
pub mod latest_proven_certificate_per_network;
pub mod latest_settled_certificate_per_network;
pub(crate) mod metadata;

// Debug
pub(crate) mod debug_certificates;

// PerEpoch
pub mod epochs {
    pub(crate) mod certificates;
    pub mod end_checkpoint;
    pub(crate) mod metadata;
    pub(crate) mod proofs;
    pub(crate) mod start_checkpoint;
}
