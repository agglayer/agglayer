use rocksdb::ColumnFamilyDescriptor;

pub const CFS: [&str; 3] = [
    crate::columns::LOCAL_EXIT_TREE_PER_NETWORK_CF,
    crate::columns::BALANCE_TREE_PER_NETWORK_CF,
    crate::columns::NULLIFIER_TREE_PER_NETWORK_CF,
];

/// Definitions for the column families in the state storage.
pub fn local_network_state_db_cf_definitions() -> Vec<ColumnFamilyDescriptor> {
    super::default_db_cf_definitions(&CFS)
}
