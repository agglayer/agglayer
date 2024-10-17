use rocksdb::ColumnFamilyDescriptor;

pub const CFS: [&str; 2] = [
    crate::columns::METADATA_CF,
    crate::columns::LATEST_SETTLED_CERTIFICATE_PER_NETWORK_CF,
];

/// Definitions for the column families in the metadata storage.
pub fn metadata_db_cf_definitions() -> Vec<ColumnFamilyDescriptor> {
    super::default_db_cf_definitions(&CFS)
}
