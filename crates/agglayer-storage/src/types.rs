use std::collections::BTreeMap;

use agglayer_types::Hash;
use agglayer_types::{Certificate, CertificateHeader, CertificateId, Height, NetworkId, Proof};
use serde::{Deserialize, Serialize};

use crate::columns::Codec;

macro_rules! default_codec_impl {
    ($($ident: ident),+) => {
        $(impl crate::columns::Codec for $ident {})+
    };
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MetadataKey {
    LatestSettledEpoch,
    EpochSynchronization,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MetadataValue {
    LatestSettledEpoch(u64),
    EpochSynchronization(u64),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PerEpochMetadataKey {
    SettlementTxHash,
    StartCheckpoint,
    EndCheckpoint,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PerEpochMetadataValue {
    SettlementTxHash([u8; 32]),
    StartCheckpoint(BTreeMap<NetworkId, Height>),
    EndCheckpoint(BTreeMap<NetworkId, Height>),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct SmtKey {
    pub(crate) network_id: u32,
    pub(crate) key_type: SmtKeyType,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum SmtKeyType {
    Root,
    Node(Hash),
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum SmtValue {
    Node(Hash, Hash),
    Leaf(Hash),
}

impl Codec for SmtKey {}
impl Codec for SmtValue {}

default_codec_impl!(
    u64,
    u32,
    Certificate,
    CertificateId,
    CertificateHeader,
    MetadataKey,
    MetadataValue,
    NetworkId,
    PerEpochMetadataKey,
    PerEpochMetadataValue,
    Proof
);
