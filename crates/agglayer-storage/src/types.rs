use std::collections::BTreeMap;

use agglayer_types::{Certificate, CertificateHeader, CertificateId, Height, NetworkId, Proof};
use serde::{Deserialize, Serialize};

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
