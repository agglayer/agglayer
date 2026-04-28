use crate::types::generated::agglayer::storage::v0;

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

crate::schema::impl_codec_using_protobuf_for!(MigrationRecord => v0::MigrationRecord);
