use std::io;

use agglayer_types::SettlementJobId;
use prost::{bytes::BytesMut, Message as _};

use crate::{
    schema::{Codec, CodecError},
    types::generated::agglayer::storage::v0,
};

pub type Key = SettlementJobId;

pub type Value = v0::SettlementJobResult;

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

#[cfg(test)]
mod tests {
    use super::{Key, Value};

    mod key {
        use super::Key;

        crate::types::codec_tests::codec_tests!(Key::from(ulid::Ulid::from(
            0x0102030405060708090a0b0c0d0e0f10_u128
        )));
    }

    impl<'a> arbitrary::Arbitrary<'a> for Value {
        fn arbitrary(input: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
            Ok(Self::contract_call_success_for_test(
                <u8 as arbitrary::Arbitrary>::arbitrary(input)?,
            ))
        }
    }

    crate::types::codec_tests::codec_tests!(Value::contract_call_success_for_test(0x17));
}
