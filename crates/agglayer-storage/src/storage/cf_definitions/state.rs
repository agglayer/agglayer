use rocksdb::ColumnFamilyDescriptor;

pub const CFS: [&str; 4] = [
    crate::columns::CERTIFICATE_HEADER_CF,
    crate::columns::CERTIFICATE_PER_NETWORK_CF,
    crate::columns::LATEST_SETTLED_CERTIFICATE_PER_NETWORK_CF,
    crate::columns::METADATA_CF,
];

/// Definitions for the column families in the state storage.
pub fn state_db_cf_definitions() -> Vec<ColumnFamilyDescriptor> {
    super::default_db_cf_definitions(&CFS)
}
