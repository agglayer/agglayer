use agglayer_types::{
    Certificate, CertificateId, CertificateIndex, EpochNumber, Height, NetworkId, Proof,
};
use serde::{Deserialize, Serialize};

macro_rules! default_codec_impl {
    ($($ident: ident),+) => {
        $(impl crate::columns::Codec for $ident {})+
    };
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hash(pub(crate) [u8; 32]);

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertificateHeader {
    pub(crate) network_id: NetworkId,
    pub(crate) height: Height,
    pub(crate) epoch_number: EpochNumber,
    pub(crate) certificate_index: CertificateIndex,
    pub(crate) local_exit_root: Hash,
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
    u32,
    u64,
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
