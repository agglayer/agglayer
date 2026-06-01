use std::io;

use agglayer_types::SettlementJobId;

use crate::schema::{Codec, CodecError};

pub type Key = SettlementJobId;

pub type Value = crate::types::generated::agglayer::storage::v0::SettlementJob;

impl Codec for SettlementJobId {
    fn encode_into<W: io::Write>(&self, mut writer: W) -> Result<(), CodecError> {
        writer.write_all(&self.to_be_bytes())?;

        Ok(())
    }

    fn decode(buf: &[u8]) -> Result<Self, CodecError> {
        Ok(Self::from(crate::schema::decode_u128_be(
            buf,
            "settlement job id",
        )?))
    }
}

crate::schema::impl_codec_using_protobuf_for!(Value);

#[cfg(test)]
mod tests {
    use super::Key;

    crate::types::codec_tests::codec_tests!(Key::from(0x0102030405060708090a0b0c0d0e0f10_u128));
}
