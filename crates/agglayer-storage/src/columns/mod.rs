use bincode::{
    config::{BigEndian, FixintEncoding, WithOtherEndian, WithOtherIntEncoding},
    DefaultOptions, Options,
};
use serde::{de::DeserializeOwned, Serialize};

use crate::error::Error;

pub fn default_bincode_options(
) -> WithOtherIntEncoding<WithOtherEndian<DefaultOptions, BigEndian>, FixintEncoding> {
    DefaultOptions::new()
        .with_big_endian()
        .with_fixint_encoding()
}

// State related CFs
pub const CERTIFICATE_PER_NETWORK_CF: &str = "certificate_per_network_cf";
pub const NULLIFIER_TREE_PER_NETWORK_CF: &str = "nullifier_tree_per_network_cf";
pub const BALANCE_TREE_PER_NETWORK_CF: &str = "balance_tree_per_network_cf";
pub const LOCAL_EXIT_TREE_PER_NETWORK_CF: &str = "local_exit_tree_per_network_cf";

// Metadata CFs
pub const CERTIFICATE_HEADER_CF: &str = "certificate_header_cf";
pub const LATEST_PROVEN_CERTIFICATE_PER_NETWORK_CF: &str =
    "latest_proven_certificate_per_network_cf";
pub const LATEST_SETTLED_CERTIFICATE_PER_NETWORK_CF: &str =
    "latest_settled_certificate_per_network_cf";
pub const METADATA_CF: &str = "metadata_cf";

// epochs related CFs
pub const PER_EPOCH_CERTIFICATES_CF: &str = "per_epoch_certificates_cf";
pub const PER_EPOCH_METADATA_CF: &str = "per_epoch_metadata_cf";
pub const PER_EPOCH_PROOFS_CF: &str = "per_epoch_proofs_cf";

// Pending related CFs
pub const PENDING_QUEUE_CF: &str = "pending_queue_cf";
pub const PROOF_PER_CERTIFICATE_CF: &str = "proof_per_certificate_cf";

pub trait Codec: Sized + Serialize + DeserializeOwned {
    fn encode(&self) -> Result<Vec<u8>, Error> {
        Ok(default_bincode_options().serialize(self)?)
    }

    fn decode(buf: &[u8]) -> Result<Self, Error> {
        Ok(default_bincode_options().deserialize(buf)?)
    }
}

pub trait ColumnSchema {
    type Key: Codec;
    type Value: Codec;

    const COLUMN_FAMILY_NAME: &'static str;
}

// State
pub(crate) mod balance_tree_per_network;
pub(crate) mod certificate_per_network;
pub(crate) mod local_exit_tree_per_network;
pub(crate) mod nullifier_tree_per_network;

// Pending
pub(crate) mod pending_queue;
pub(crate) mod proof_per_certificate;

// Metadata
pub(crate) mod certificate_header;
pub mod latest_proven_certificate_per_network;
pub mod latest_settled_certificate_per_network;
pub(crate) mod metadata;

// PerEpoch
pub mod epochs {
    pub(crate) mod certificates;
    pub(crate) mod metadata;
    pub(crate) mod proofs;
}
