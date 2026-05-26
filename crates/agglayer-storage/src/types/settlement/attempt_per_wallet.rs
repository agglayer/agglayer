use std::io;

use agglayer_types::SettlementJobId;
use serde::{Deserialize, Serialize};

use crate::schema::{Codec, CodecError};

const ADDRESS_LEN: usize = 20;
const KEY_LEN: usize =
    ADDRESS_LEN + crate::schema::U64_LEN + crate::schema::RAW_ULID_LEN + crate::schema::U64_LEN;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Key {
    pub(crate) address: [u8; 20],
    pub(crate) nonce: u64,
    pub(crate) settlement_job_id: SettlementJobId,
    pub(crate) attempt_sequence_number: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Value;

impl Codec for Key {
    fn encode_into<W: io::Write>(&self, mut writer: W) -> Result<(), CodecError> {
        writer.write_all(&self.address)?;
        writer.write_all(&self.nonce.to_be_bytes())?;
        writer.write_all(&self.settlement_job_id.as_ulid().to_bytes())?;
        writer.write_all(&self.attempt_sequence_number.to_be_bytes())?;

        Ok(())
    }

    fn decode(buf: &[u8]) -> Result<Self, CodecError> {
        let key = crate::schema::fixed_bytes::<KEY_LEN>(buf, "settlement attempt per wallet key")?;
        let address_end = ADDRESS_LEN;
        let nonce_end = address_end + crate::schema::U64_LEN;
        let job_id_end = nonce_end + crate::schema::RAW_ULID_LEN;

        let address = crate::schema::fixed_bytes::<ADDRESS_LEN>(
            &key[..address_end],
            "settlement sender wallet",
        )?;
        let nonce = crate::schema::decode_u64_be(&key[address_end..nonce_end], "settlement nonce")?;
        let settlement_job_id =
            crate::schema::decode_raw_ulid(&key[nonce_end..job_id_end], "settlement job id")?;
        let attempt_sequence_number =
            crate::schema::decode_u64_be(&key[job_id_end..], "settlement attempt sequence number")?;

        Ok(Self {
            address,
            nonce,
            settlement_job_id: SettlementJobId::from(settlement_job_id),
            attempt_sequence_number,
        })
    }
}

impl Codec for Value {
    fn encode_into<W: io::Write>(&self, _writer: W) -> Result<(), CodecError> {
        Ok(())
    }

    fn decode(buf: &[u8]) -> Result<Self, CodecError> {
        if buf.is_empty() {
            Ok(Self)
        } else {
            Err(CodecError::Conversion(format!(
                "settlement attempt per wallet value must be empty, got {} bytes",
                buf.len()
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use agglayer_types::SettlementJobId;

    use super::{Key, Value, ADDRESS_LEN};

    impl<'a> arbitrary::Arbitrary<'a> for Key {
        fn arbitrary(input: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
            Ok(Self {
                address: <[u8; ADDRESS_LEN] as arbitrary::Arbitrary>::arbitrary(input)?,
                nonce: <u64 as arbitrary::Arbitrary>::arbitrary(input)?,
                settlement_job_id: <SettlementJobId as arbitrary::Arbitrary>::arbitrary(input)?,
                attempt_sequence_number: <u64 as arbitrary::Arbitrary>::arbitrary(input)?,
            })
        }
    }

    crate::types::codec_tests::codec_tests!(Key {
        address: std::array::from_fn(|index| 0xa0_u8 + index as u8),
        nonce: 0x0102030405060708,
        settlement_job_id: SettlementJobId::from(ulid::Ulid::from(
            0x1112131415161718191a1b1c1d1e1f20_u128
        )),
        attempt_sequence_number: 0x2122232425262728,
    });

    mod value {
        use super::Value;

        impl<'a> arbitrary::Arbitrary<'a> for Value {
            fn arbitrary(_input: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
                Ok(Self)
            }
        }

        crate::types::codec_tests::codec_tests!(Value);
    }
}
