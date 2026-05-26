use std::io;

use agglayer_types::SettlementJobId;
use serde::{Deserialize, Serialize};

use crate::schema::{Codec, CodecError};

const KEY_LEN: usize = crate::schema::RAW_ULID_LEN + crate::schema::U64_LEN;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Key {
    pub(crate) settlement_job_id: SettlementJobId,
    pub(crate) attempt_sequence_number: u64,
}

pub type Value = crate::types::generated::agglayer::storage::v0::SettlementAttempt;

impl Codec for Key {
    fn encode_into<W: io::Write>(&self, mut writer: W) -> Result<(), CodecError> {
        writer.write_all(&self.settlement_job_id.as_ulid().to_bytes())?;
        writer.write_all(&self.attempt_sequence_number.to_be_bytes())?;

        Ok(())
    }

    fn decode(buf: &[u8]) -> Result<Self, CodecError> {
        let key = crate::schema::fixed_bytes::<KEY_LEN>(buf, "settlement attempt key")?;
        let settlement_job_id = crate::schema::decode_raw_ulid(
            &key[..crate::schema::RAW_ULID_LEN],
            "settlement job id",
        )?;
        let attempt_sequence_number = crate::schema::decode_u64_be(
            &key[crate::schema::RAW_ULID_LEN..],
            "settlement attempt sequence number",
        )?;

        Ok(Self {
            settlement_job_id: SettlementJobId::from(settlement_job_id),
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
        settlement_job_id: SettlementJobId::from(ulid::Ulid::from(
            0x0102030405060708090a0b0c0d0e0f10_u128
        )),
        attempt_sequence_number: 0x1112131415161718,
    });
}
