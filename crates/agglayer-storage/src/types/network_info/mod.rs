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

#[cfg(test)]
mod tests {
    use strum::EnumCount;

    use super::Key;
    use crate::types::network_info::v0::network_info_value::ValueDiscriminants;

    impl<'a> arbitrary::Arbitrary<'a> for Key {
        fn arbitrary(input: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
            let kind_index = <u8 as arbitrary::Arbitrary>::arbitrary(input)? as usize
                % ValueDiscriminants::COUNT;

            Ok(Self {
                network_id: <u32 as arbitrary::Arbitrary>::arbitrary(input)?,
                kind: ValueDiscriminants::from_repr(kind_index)
                    .expect("modulo keeps discriminant in range"),
            })
        }
    }

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

    crate::types::codec_tests::codec_tests!(Key {
        network_id: 0x01020304,
        kind: ValueDiscriminants::SettledClaim,
    });
}
