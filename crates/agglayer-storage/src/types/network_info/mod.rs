use std::io;

use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

pub use super::generated::agglayer::storage::v0;
use crate::{
    schema::{Codec, CodecError},
    types::network_info::v0::{
        network_info_value::{self, ValueDiscriminants},
        NetworkType,
    },
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Key {
    pub(crate) network_id: u32,
    pub(crate) kind: ValueDiscriminants,
}

impl Key {
    pub(crate) const NETWORK_ID_LEN: usize = crate::schema::U32_LEN;
    pub(crate) const LEN: usize = Self::NETWORK_ID_LEN + crate::schema::U32_LEN;

    pub(crate) fn all_keys_for_network(
        network_id: agglayer_types::NetworkId,
    ) -> impl ExactSizeIterator<Item = Self> + Clone {
        let network_id = network_id.to_u32();
        network_info_value::ValueDiscriminants::iter().map(move |kind| Key { network_id, kind })
    }
}

pub type Value = super::generated::agglayer::storage::v0::NetworkInfoValue;

impl Codec for Key {
    fn encode_into<W: io::Write>(&self, mut writer: W) -> Result<(), CodecError> {
        writer.write_all(&self.network_id.to_be_bytes())?;
        writer.write_all(&(self.kind as u32).to_be_bytes())?;

        Ok(())
    }

    fn decode(buf: &[u8]) -> Result<Self, CodecError> {
        let key = crate::schema::fixed_bytes::<{ Key::LEN }>(buf, "network info key")?;
        let network_id =
            crate::schema::decode_u32_be(&key[..Key::NETWORK_ID_LEN], "network info network id")?;
        let kind = crate::schema::decode_u32_be(&key[Key::NETWORK_ID_LEN..], "network info kind")?;
        let kind = ValueDiscriminants::from_repr(kind as usize).ok_or_else(|| {
            CodecError::InvalidEnumVariant(format!("invalid network info key kind {kind}"))
        })?;

        Ok(Self { network_id, kind })
    }
}

crate::schema::impl_codec_using_protobuf_for!(Value);

impl TryFrom<v0::NetworkType> for agglayer_types::NetworkType {
    type Error = CodecError;

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

impl From<agglayer_types::NetworkType> for v0::NetworkType {
    fn from(value: agglayer_types::NetworkType) -> Self {
        match value {
            agglayer_types::NetworkType::Unspecified => NetworkType::Unspecified,
            agglayer_types::NetworkType::Ecdsa => NetworkType::Ecdsa,
            agglayer_types::NetworkType::Generic => NetworkType::Generic,
            agglayer_types::NetworkType::MultisigOnly => NetworkType::MultisigOnly,
            agglayer_types::NetworkType::MultisigAndAggchainProof => {
                NetworkType::MultisigAndAggchainProof
            }
        }
    }
}

#[cfg(test)]
mod tests;
