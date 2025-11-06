use std::io;

use prost::{bytes::BytesMut, Message as _};

use crate::columns::Codec;

pub type Value = super::generated::agglayer::storage::v0::DisabledNetwork;

impl Codec for Value {
    fn encode(&self) -> Result<Vec<u8>, crate::columns::CodecError> {
        let len = self.encoded_len();

        let mut buf = BytesMut::new();
        buf.reserve(len);
        <Value as prost::Message>::encode(self, &mut buf)?;

        Ok(buf.to_vec())
    }

    fn encode_into<W: io::Write>(&self, _writer: W) -> Result<(), crate::columns::CodecError> {
        unimplemented!()
    }

    fn decode(buf: &[u8]) -> Result<Self, crate::columns::CodecError> {
        <Value as prost::Message>::decode(buf).map_err(Into::into)
    }
}
