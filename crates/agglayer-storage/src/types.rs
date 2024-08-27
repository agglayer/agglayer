use std::ops::Deref;

use bincode::Options as _;
use serde::{Deserialize, Serialize};

pub struct Certificate(Vec<u8>);

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpochNumber(pub(crate) u64);

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertificateIndex(pub(crate) u64);

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertificateId(pub(crate) [u8; 32]);

impl Deref for CertificateId {
    type Target = [u8; 32];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

macro_rules! wrapper_codec_impl {
    ($($ident: ident),+) => {
        $(impl crate::columns::Codec for $ident {
            fn encode(&self) -> Result<Vec<u8>, crate::error::Error> {
                Ok(crate::columns::default_bincode_options().serialize(&self.0)?)
            }

            fn decode(buf: &[u8]) -> Result<Self, crate::error::Error> {
                Ok(Self(crate::columns::default_bincode_options().deserialize(buf)?))
            }
        })+
    };
}

macro_rules! default_codec_impl {
    ($($ident: ident),+) => {
        $(impl crate::columns::Codec for $ident {
            fn encode(&self) -> Result<Vec<u8>, crate::error::Error> {
                Ok(crate::columns::default_bincode_options().serialize(self)?)
            }

            fn decode(buf: &[u8]) -> Result<Self, crate::error::Error> {
                Ok(crate::columns::default_bincode_options().deserialize(buf)?)
            }
        })+
    };
}
wrapper_codec_impl!(
    Certificate,
    CertificateId,
    CertificateIndex,
    NetworkId,
    Proof
);

#[derive(Debug, PartialEq, Eq)]
pub struct Proof(pub(crate) Vec<u8>);

impl Deref for Proof {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Height(pub(crate) u64);
impl Deref for Height {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkId(pub(crate) u32);

impl NetworkId {
    pub fn new(network_id: u32) -> Self {
        Self(network_id)
    }
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
    LastestSettledEpoch(u64),
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
    MetadataValue,
    MetadataKey,
    CertificateHeader,
    PerEpochMetadataValue,
    PerEpochMetadataKey
);
