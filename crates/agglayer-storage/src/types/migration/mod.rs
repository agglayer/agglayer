use std::io;

use prost::{bytes::BytesMut, Message as _};

use crate::{columns::Codec, types::generated::agglayer::storage::v0};

/// Metadata recorded about each migration step performed.
///
/// Reserved for future extensions.
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct MigrationRecord(());

impl From<&MigrationRecord> for v0::MigrationRecord {
    fn from(value: &MigrationRecord) -> Self {
        let MigrationRecord(()) = value;
        v0::MigrationRecord {}
    }
}

impl From<v0::MigrationRecord> for MigrationRecord {
    fn from(value: v0::MigrationRecord) -> Self {
        let v0::MigrationRecord {} = value;
        MigrationRecord(())
    }
}

impl Codec for MigrationRecord {
    fn encode_into<W: io::Write>(&self, mut writer: W) -> Result<(), crate::columns::CodecError> {
        let proto: v0::MigrationRecord = self.into();
        let len = proto.encoded_len();

        let mut buf = BytesMut::new();
        buf.reserve(len);

        <v0::MigrationRecord as prost::Message>::encode(&proto, &mut buf)?;

        writer.write_all(&buf)?;

        Ok(())
    }

    fn decode(buf: &[u8]) -> Result<Self, crate::columns::CodecError> {
        let proto = <v0::MigrationRecord as prost::Message>::decode(buf)?;
        Ok(proto.into())
    }
}
