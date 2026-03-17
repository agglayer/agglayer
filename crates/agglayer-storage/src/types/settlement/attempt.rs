use std::io;

use prost::{bytes::BytesMut, Message as _};
use serde::{Deserialize, Serialize};

use crate::schema::Codec;

#[derive(Clone, Serialize, Deserialize)]
pub struct Key {
    pub(crate) settlement_job_id: ulid::Ulid,
    pub(crate) attempt_sequence_number: u64,
}

pub type Value = crate::types::generated::agglayer::storage::v0::SettlementAttempt;

impl Codec for Value {
    fn encode_into<W: io::Write>(&self, mut writer: W) -> Result<(), crate::schema::CodecError> {
        let len = self.encoded_len();

        let mut buf = BytesMut::new();
        buf.reserve(len);

        <Value as prost::Message>::encode(self, &mut buf)?;

        writer.write_all(&buf)?;

        Ok(())
    }

    fn decode(buf: &[u8]) -> Result<Self, crate::schema::CodecError> {
        <Value as prost::Message>::decode(buf).map_err(Into::into)
    }
}
