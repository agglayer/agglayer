use agglayer_types::{
    primitives::Digest, CertificateHeader, CertificateId, CertificateIndex, EpochNumber, Height,
    NetworkId, Proof,
};
use serde::{Deserialize, Serialize};

mod certificate;

#[derive(Debug, Serialize, Deserialize)]
pub enum MetadataKey {
    LatestSettledEpoch,
    EpochSynchronization, // Actually unused, kept for storage backward compatibility
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MetadataValue {
    LatestSettledEpoch(EpochNumber),
    EpochSynchronization(u64), // Actually unused, kept for storage backward compatibility
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PerEpochMetadataKey {
    SettlementTxHash,
    Packed,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PerEpochMetadataValue {
    SettlementTxHash(Digest),
    Packed(bool),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct SmtKey {
    pub(crate) network_id: u32,
    pub(crate) key_type: SmtKeyType,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum SmtKeyType {
    Root,
    Node(Digest),
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum SmtValue {
    Node(Digest, Digest),
    Leaf(Digest),
}

impl_codec_using_bincode_for!(
    u64,
    u32,
    CertificateId,
    CertificateIndex,
    CertificateHeader,
    Digest,
    Height,
    MetadataKey,
    MetadataValue,
    NetworkId,
    PerEpochMetadataKey,
    PerEpochMetadataValue,
    Proof,
    SmtKey,
    SmtValue,
);
