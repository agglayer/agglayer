use std::io;

use agglayer_types::SettlementJobId;
use serde::{Deserialize, Serialize};

use crate::schema::{Codec, CodecError};

const ADDRESS_LEN: usize = 20;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Key {
    pub(crate) address: [u8; 20],
    pub(crate) nonce: u64,
    pub(crate) settlement_job_id: SettlementJobId,
    pub(crate) attempt_sequence_number: u64,
}

impl Key {
    pub(crate) const ADDRESS_LEN: usize = ADDRESS_LEN;
    pub(crate) const WALLET_PREFIX_LEN: usize = Self::ADDRESS_LEN;
    pub(crate) const LEN: usize = Self::ADDRESS_LEN
        + crate::schema::U64_LEN
        + SettlementJobId::BYTE_LEN
        + crate::schema::U64_LEN;
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Value;

impl Codec for Key {
    fn encode_into<W: io::Write>(&self, mut writer: W) -> Result<(), CodecError> {
        writer.write_all(&self.address)?;
        writer.write_all(&self.nonce.to_be_bytes())?;
        writer.write_all(&self.settlement_job_id.to_be_bytes())?;
        writer.write_all(&self.attempt_sequence_number.to_be_bytes())?;

        Ok(())
    }

    fn decode(buf: &[u8]) -> Result<Self, CodecError> {
        let key =
            crate::schema::fixed_bytes::<{ Self::LEN }>(buf, "settlement attempt per wallet key")?;

        let (address, rest) = key.split_at(Self::ADDRESS_LEN);
        let (nonce, rest) = rest.split_at(crate::schema::U64_LEN);
        let (settlement_job_id, attempt_sequence_number) = rest.split_at(SettlementJobId::BYTE_LEN);

        let address =
            crate::schema::fixed_bytes::<ADDRESS_LEN>(address, "settlement sender wallet")?;
        let nonce = crate::schema::decode_u64_be(nonce, "settlement nonce")?;
        let settlement_job_id = SettlementJobId::from(crate::schema::decode_u128_be(
            settlement_job_id,
            "settlement job id",
        )?);
        let attempt_sequence_number = crate::schema::decode_u64_be(
            attempt_sequence_number,
            "settlement attempt sequence number",
        )?;

        Ok(Self {
            address,
            nonce,
            settlement_job_id,
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

    use super::{Key, Value};

    impl<'a> arbitrary::Arbitrary<'a> for Key {
        fn arbitrary(input: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
            Ok(Self {
                address: <[u8; Key::ADDRESS_LEN] as arbitrary::Arbitrary>::arbitrary(input)?,
                nonce: <u64 as arbitrary::Arbitrary>::arbitrary(input)?,
                settlement_job_id: <SettlementJobId as arbitrary::Arbitrary>::arbitrary(input)?,
                attempt_sequence_number: <u64 as arbitrary::Arbitrary>::arbitrary(input)?,
            })
        }
    }

    crate::types::codec_tests::codec_tests!(Key {
        address: std::array::from_fn(|index| 0xa0_u8 + index as u8),
        nonce: 0x0102030405060708,
        settlement_job_id: SettlementJobId::from(0x1112131415161718191a1b1c1d1e1f20_u128),
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
