use std::io;

use prost::{bytes::BytesMut, Message as _};

use crate::columns::Codec;

pub type Value = super::generated::agglayer::storage::v0::DisabledNetwork;
pub use super::generated::agglayer::storage::v0::DisabledBy;

impl Codec for Value {
    fn encode_into<W: io::Write>(&self, mut writer: W) -> Result<(), crate::columns::CodecError> {
        let len = self.encoded_len();

        let mut buf = BytesMut::new();
        buf.reserve(len);

        <Value as prost::Message>::encode(self, &mut buf)?;

        writer.write_all(&buf)?;

        Ok(())
    }

    fn decode(buf: &[u8]) -> Result<Self, crate::columns::CodecError> {
        <Value as prost::Message>::decode(buf).map_err(Into::into)
    }
}
