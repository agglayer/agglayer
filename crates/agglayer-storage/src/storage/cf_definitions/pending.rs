use rocksdb::ColumnFamilyDescriptor;

pub const CFS: [&str; 4] = [
    crate::columns::LATEST_PROVEN_CERTIFICATE_PER_NETWORK_CF,
    crate::columns::LATEST_PENDING_CERTIFICATE_PER_NETWORK_CF,
    crate::columns::PENDING_QUEUE_CF,
    crate::columns::PROOF_PER_CERTIFICATE_CF,
];

/// Definitions for the column families in the pending queue storage.
pub fn pending_db_cf_definitions() -> Vec<ColumnFamilyDescriptor> {
    super::default_db_cf_definitions(&CFS)
}
