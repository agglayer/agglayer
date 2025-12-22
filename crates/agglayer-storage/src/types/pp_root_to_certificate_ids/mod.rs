use std::io;

use prost::{bytes::BytesMut, Message};

use crate::columns::Codec;

pub type Key = super::generated::agglayer::storage::v0::SettledPessimisticProofRoot;

impl Codec for Key {
    fn encode_into<W: io::Write>(&self, mut writer: W) -> Result<(), crate::columns::CodecError> {
        let len = self.encoded_len();

        let mut buf = BytesMut::new();
        buf.reserve(len);

        <Key as prost::Message>::encode(self, &mut buf)?;

        writer.write_all(&buf)?;

        Ok(())
    }

    fn decode(buf: &[u8]) -> Result<Self, crate::columns::CodecError> {
        <Key as prost::Message>::decode(buf).map_err(Into::into)
    }
}

pub type Value = Vec<super::generated::agglayer::storage::v0::SettledCertificateId>;

impl Codec for Value {
    fn encode_into<W: io::Write>(&self, mut writer: W) -> Result<(), crate::columns::CodecError> {
        // Write the length as u32 (little-endian)
        writer.write_all(&(self.len() as u32).to_le_bytes())?;

        // Encode each item with length prefix (u32 little-endian)
        for item in self {
            let len = item.encoded_len();
            let mut buf = BytesMut::new();
            buf.reserve(len);
            <super::generated::agglayer::storage::v0::SettledCertificateId as prost::Message>::encode(item, &mut buf)?;
            
            // Write length prefix as u32 (little-endian)
            writer.write_all(&(len as u32).to_le_bytes())?;
            writer.write_all(&buf)?;
        }

        Ok(())
    }

    fn decode(buf: &[u8]) -> Result<Self, crate::columns::CodecError> {
        if buf.len() < 4 {
            return Err(crate::columns::CodecError::ProtobufDeserialization(
                prost::DecodeError::new("buffer too short for length prefix"),
            ));
        }

        // Read the Vec length
        let len = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
        let mut result = Vec::with_capacity(len);
        let mut offset = 4;

        // Decode each item (length-delimited)
        for _ in 0..len {
            if offset + 4 > buf.len() {
                return Err(crate::columns::CodecError::ProtobufDeserialization(
                    prost::DecodeError::new("unexpected end of buffer"),
                ));
            }

            // Read item length prefix
            let item_len = u32::from_le_bytes([
                buf[offset],
                buf[offset + 1],
                buf[offset + 2],
                buf[offset + 3],
            ]) as usize;
            offset += 4;

            if offset + item_len > buf.len() {
                return Err(crate::columns::CodecError::ProtobufDeserialization(
                    prost::DecodeError::new("buffer too short for item"),
                ));
            }

            // Decode the item
            let item = <super::generated::agglayer::storage::v0::SettledCertificateId as prost::Message>::decode(
                &buf[offset..offset + item_len],
            )?;
            offset += item_len;
            result.push(item);
        }

        Ok(result)
    }
}
