use agglayer_types::{Certificate, CertificateHeader, CertificateId, Proof};
use agglayer_types::{
    Certificate, CertificateId, CertificateIndex, EpochNumber, Height, NetworkId, Proof,
};
use serde::{Deserialize, Serialize};

use crate::{
    columns::{default_bincode_options, Codec},
    error::Error,
};

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
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PerEpochMetadataValue {
    SettlementTxHash([u8; 32]),
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
