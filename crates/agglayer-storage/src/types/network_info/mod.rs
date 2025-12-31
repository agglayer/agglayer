use std::io;

use prost::{bytes::BytesMut, Message};
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

#[derive(Clone, Serialize, Deserialize)]
pub struct Key {
    pub(crate) network_id: u32,
    pub(crate) kind: ValueDiscriminants,
}

impl Key {
    pub(crate) fn all_keys_for_network(
        network_id: agglayer_types::NetworkId,
    ) -> impl ExactSizeIterator<Item = Self> + Clone {
        let network_id = network_id.to_u32();
        network_info_value::ValueDiscriminants::iter().map(move |kind| Key { network_id, kind })
    }
}

pub type Value = super::generated::agglayer::storage::v0::NetworkInfoValue;

impl Codec for Value {
    fn encode_into<W: io::Write>(&self, mut writer: W) -> Result<(), CodecError> {
        let len = self.encoded_len();

        let mut buf = BytesMut::new();
        buf.reserve(len);

        <Value as prost::Message>::encode(self, &mut buf)?;

        writer.write_all(&buf)?;

        Ok(())
    }

    fn decode(buf: &[u8]) -> Result<Self, CodecError> {
        <Value as prost::Message>::decode(buf).map_err(Into::into)
    }
}

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

#[cfg(test)]
mod tests {
    use strum::EnumCount;

    use crate::types::network_info::v0::network_info_value::ValueDiscriminants;

    #[test]
    fn test_discriminant_from_u32() {
        assert_eq!(ValueDiscriminants::COUNT, 5, "Expected 5 discriminants");
        for i in 0..ValueDiscriminants::COUNT {
            let discriminant = ValueDiscriminants::from_repr(i);
            assert!(discriminant.is_some());

            match discriminant.unwrap() {
                ValueDiscriminants::NetworkType => {
                    assert_eq!(i, 0, "NetworkType should be at index 0")
                }
                ValueDiscriminants::LatestSettledCertificate => {
                    assert_eq!(i, 1, "LatestSettledCertificate should be at index 1")
                }
                ValueDiscriminants::LatestSettledClaim => {
                    assert_eq!(i, 2, "LatestSettledClaim should be at index 2")
                }
                ValueDiscriminants::LatestPendingCertificateInfo => {
                    assert_eq!(i, 3, "LatestPendingCertificateInfo should be at index 3")
                }
                ValueDiscriminants::LatestProvenCertificateInfo => {
                    assert_eq!(i, 4, "LatestProvenCertificateInfo should be at index 4")
                }
            }
        }
    }
}
