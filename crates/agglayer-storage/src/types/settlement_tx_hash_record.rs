use std::io;

use agglayer_types::{Digest, SettlementTxHash};
use prost::{bytes::BytesMut, Message as _};

use crate::{
    columns::{Codec, CodecError},
    types::generated::agglayer::storage::v0,
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub struct SettlementTxHashRecord {
    // Hash data, uniqued and in the order of insertion
    hashes: Vec<SettlementTxHash>,
}

impl SettlementTxHashRecord {
    pub const fn new() -> Self {
        let hashes = Vec::new();
        Self { hashes }
    }

    pub const fn len(&self) -> usize {
        self.hashes.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.hashes.is_empty()
    }

    pub fn contains(&self, hash: &SettlementTxHash) -> bool {
        self.hashes.contains(hash)
    }

    pub fn insert(&mut self, hash: SettlementTxHash) {
        // If we already have this hash, put it last.
        if let Some(orig_idx) = self.hashes.iter().position(|h| h == &hash) {
            self.hashes.remove(orig_idx);
        }
        self.hashes.push(hash);
    }

    pub fn retain(&mut self, f: impl FnMut(&SettlementTxHash) -> bool) {
        self.hashes.retain(f);
    }

    pub fn into_vec(self) -> Vec<SettlementTxHash> {
        self.hashes
    }
}

// Conversion from SettlementTxHashRecord to protobuf type
impl From<&SettlementTxHashRecord> for v0::SettlementTxHashRecord {
    fn from(record: &SettlementTxHashRecord) -> Self {
        v0::SettlementTxHashRecord {
            hashes: Some(v0::TxHashHistory {
                hashes: record
                    .hashes
                    .iter()
                    .map(|h| prost::bytes::Bytes::from(Digest::from(*h).to_vec()))
                    .collect(),
            }),
        }
    }
}

// Conversion from protobuf type to SettlementTxHashRecord
impl TryFrom<v0::SettlementTxHashRecord> for SettlementTxHashRecord {
    type Error = CodecError;

    fn try_from(proto: v0::SettlementTxHashRecord) -> Result<Self, Self::Error> {
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

        Ok(SettlementTxHashRecord { hashes })
    }
}

impl Codec for SettlementTxHashRecord {
    fn encode(&self) -> Result<Vec<u8>, CodecError> {
        let proto: v0::SettlementTxHashRecord = self.into();
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
        v0::SettlementTxHashRecord::decode(buf)?.try_into()
    }
}
