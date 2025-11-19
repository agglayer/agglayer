use std::io;

use agglayer_types::{Digest, SettlementTxHash, SettlementTxRecord};
use prost::{bytes::BytesMut, Message as _};

use crate::{
    columns::{Codec, CodecError},
    types::generated::agglayer::storage::v0,
};

// Conversion from SettlementTxHashRecord to protobuf type
impl From<&SettlementTxRecord> for v0::SettlementTxRecord {
    fn from(record: &SettlementTxRecord) -> Self {
        v0::SettlementTxRecord {
            hashes: Some(v0::TxHashHistory {
                hashes: record
                    .hashes()
                    .iter()
                    .map(|h| prost::bytes::Bytes::from(Digest::from(*h).to_vec()))
                    .collect(),
            }),
        }
    }
}

// Conversion from protobuf type to SettlementTxRecord
impl TryFrom<v0::SettlementTxRecord> for SettlementTxRecord {
    type Error = CodecError;

    fn try_from(proto: v0::SettlementTxRecord) -> Result<Self, Self::Error> {
        let hashes = proto
            .hashes
            .ok_or_else(|| {
                CodecError::ProtobufDeserialization(prost::DecodeError::new("Hash history missing"))
            })?
            .hashes
            .into_iter()
            .map(|bytes| {
                let hash_array: [u8; 32] = bytes.as_ref().try_into().map_err(|_| {
                    CodecError::ProtobufDeserialization(prost::DecodeError::new(
                        "Invalid hash length: expected 32 bytes",
                    ))
                })?;
                Ok(SettlementTxHash::from(Digest::from(hash_array)))
            })
            .collect::<Result<Vec<_>, CodecError>>()?;

        Ok(SettlementTxRecord::from_vec(hashes))
    }
}

impl Codec for SettlementTxRecord {
    fn encode(&self) -> Result<Vec<u8>, CodecError> {
        let proto: v0::SettlementTxRecord = self.into();
        let len = proto.encoded_len();

        let mut buf = BytesMut::new();
        buf.reserve(len);
        proto.encode(&mut buf)?;

        Ok(buf.to_vec())
    }

    fn encode_into<W: io::Write>(&self, mut writer: W) -> Result<(), CodecError> {
        writer.write(self.encode()?.as_slice())?;
        Ok(())
    }

    fn decode(buf: &[u8]) -> Result<Self, CodecError> {
        v0::SettlementTxRecord::decode(buf)?.try_into()
    }
}
