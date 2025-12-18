use rocksdb::ColumnFamilyDescriptor;

pub const CFS: [&str; 9] = [
    crate::columns::CERTIFICATE_HEADER_CF,
    crate::columns::CERTIFICATE_PER_NETWORK_CF,
    crate::columns::LATEST_SETTLED_CERTIFICATE_PER_NETWORK_CF,
    crate::columns::METADATA_CF,
    crate::columns::LOCAL_EXIT_TREE_PER_NETWORK_CF,
    crate::columns::BALANCE_TREE_PER_NETWORK_CF,
    crate::columns::NULLIFIER_TREE_PER_NETWORK_CF,
    crate::columns::NETWORK_INFO_CF,
    crate::columns::DISABLED_NETWORKS_CF,
];

/// Definitions for the column families in the state storage.
pub fn state_db_cf_definitions() -> Vec<ColumnFamilyDescriptor> {
    crate::storage::default_db_cf_definitions(&CFS)
}
