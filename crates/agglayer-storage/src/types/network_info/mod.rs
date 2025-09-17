use std::io;

use prost::{bytes::BytesMut, Message};
use serde::{Deserialize, Serialize};

pub use super::generated::agglayer::storage::v0;
use crate::{columns::Codec, types::network_info::v0::NetworkType};

#[derive(Clone, Serialize, Deserialize)]
pub struct Key {
    pub(crate) network_id: u32,
    pub(crate) kind: KeyKind,
}

impl Key {
    pub(crate) fn all_keys_for_network(
        network_id: agglayer_types::NetworkId,
    ) -> impl ExactSizeIterator<Item = Self> + Clone {
        [
            Key {
                network_id: network_id.to_u32(),
                kind: KeyKind::NetworkType,
            },
            Key {
                network_id: network_id.to_u32(),
                kind: KeyKind::LatestSettledCertificate,
            },
            Key {
                network_id: network_id.to_u32(),
                kind: KeyKind::LatestSettledClaim,
            },
            Key {
                network_id: network_id.to_u32(),
                kind: KeyKind::LatestPendingCertificate,
            },
        ]
        .into_iter()
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum KeyKind {
    NetworkType,
    LatestSettledCertificate,
    LatestSettledClaim,
    LatestPendingCertificate,
}

pub type Value = super::generated::agglayer::storage::v0::NetworkInfoValue;

impl Codec for Value {
    fn encode_into<W: io::Write>(&self, mut writer: W) -> Result<(), crate::columns::CodecError> {
        let len = self.encoded_len();

        let mut buf = BytesMut::new();
        buf.reserve(len);
        <Value as prost::Message>::encode(self, &mut buf)?;

        writer
            .write_all(&buf[..])
            .map_err(crate::columns::CodecError::UnableToWriteEncodedBytes)?;

        Ok(())
    }

    fn decode(buf: &[u8]) -> Result<Self, crate::columns::CodecError> {
        <Value as prost::Message>::decode(buf).map_err(Into::into)
    }
}

impl TryFrom<v0::NetworkType> for agglayer_types::NetworkType {
    type Error = crate::columns::CodecError;

    fn try_from(value: v0::NetworkType) -> Result<Self, Self::Error> {
        match value {
            NetworkType::Generic => Ok(agglayer_types::NetworkType::Generic),
            NetworkType::Unspecified => Ok(agglayer_types::NetworkType::Unspecified),
            NetworkType::Ecdsa => Ok(agglayer_types::NetworkType::Ecdsa),
            NetworkType::MultisigOnly => Ok(agglayer_types::NetworkType::MultisigOnly),
            NetworkType::MultisigAndAggchainProof => {
                Ok(agglayer_types::NetworkType::MultisigAndAggchainProof)
            }
        }
    }
}
