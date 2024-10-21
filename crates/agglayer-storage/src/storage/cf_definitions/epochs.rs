use rocksdb::ColumnFamilyDescriptor;

pub const CFS: [&str; 3] = [
    crate::columns::PER_EPOCH_CERTIFICATES_CF,
    crate::columns::PER_EPOCH_METADATA_CF,
    crate::columns::PER_EPOCH_PROOFS_CF,
];

/// Definitions for the column families in the epochs storage.
pub fn epochs_db_cf_definitions() -> Vec<ColumnFamilyDescriptor> {
    super::default_db_cf_definitions(&CFS)
}
