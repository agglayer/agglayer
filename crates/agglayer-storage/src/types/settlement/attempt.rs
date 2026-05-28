use std::io;

use agglayer_types::SettlementJobId;
use serde::{Deserialize, Serialize};

use crate::schema::{Codec, CodecError};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Key {
    pub(crate) settlement_job_id: SettlementJobId,
    pub(crate) attempt_sequence_number: u64,
}

impl Key {
    pub(crate) const PREFIX_LEN: usize = SettlementJobId::BYTE_LEN;
    pub(crate) const LEN: usize = Self::PREFIX_LEN + crate::schema::U64_LEN;
}

pub type Value = crate::types::generated::agglayer::storage::v0::SettlementAttempt;

impl Codec for Key {
    fn encode_into<W: io::Write>(&self, mut writer: W) -> Result<(), CodecError> {
        writer.write_all(&self.settlement_job_id.to_be_bytes())?;
        writer.write_all(&self.attempt_sequence_number.to_be_bytes())?;

        Ok(())
    }

    fn decode(buf: &[u8]) -> Result<Self, CodecError> {
        let key = crate::schema::fixed_bytes::<{ Self::LEN }>(buf, "settlement attempt key")?;
        let (settlement_job_id, attempt_sequence_number) = key.split_at(Self::PREFIX_LEN);

        let settlement_job_id = SettlementJobId::from(crate::schema::decode_u128_be(
            settlement_job_id,
            "settlement job id",
        )?);
        let attempt_sequence_number = crate::schema::decode_u64_be(
            attempt_sequence_number,
            "settlement attempt sequence number",
        )?;

        Ok(Self {
            settlement_job_id,
            attempt_sequence_number,
        })
    }
}

crate::schema::impl_codec_using_protobuf_for!(Value);

#[cfg(test)]
mod tests {
    use agglayer_types::SettlementJobId;

    use super::Key;

    impl<'a> arbitrary::Arbitrary<'a> for Key {
        fn arbitrary(input: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
            Ok(Self {
                settlement_job_id: <SettlementJobId as arbitrary::Arbitrary>::arbitrary(input)?,
                attempt_sequence_number: <u64 as arbitrary::Arbitrary>::arbitrary(input)?,
            })
        }
    }

    crate::types::codec_tests::codec_tests!(Key {
        settlement_job_id: SettlementJobId::from(0x0102030405060708090a0b0c0d0e0f10_u128),
        attempt_sequence_number: 0x1112131415161718,
    });
}
