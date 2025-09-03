use agglayer_types::{
    primitives::Digest, CertificateHeader, CertificateId, CertificateIndex, EpochNumber, Height,
    NetworkId, Proof,
};
use serde::{Deserialize, Serialize};

use crate::columns::Codec;

mod certificate;
mod generated;
pub(crate) mod network_info;

macro_rules! default_codec_impl {
    ($($ident: ident),+) => {
        $(impl crate::columns::Codec for $ident {})+
    };
}

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

impl Codec for SmtKey {}
impl Codec for SmtValue {}

default_codec_impl!(
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
    Proof
);
