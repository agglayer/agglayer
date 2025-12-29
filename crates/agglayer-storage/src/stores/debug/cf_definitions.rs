use rocksdb::ColumnFamilyDescriptor;

pub const CFS: [&str; 1] = [crate::columns::DEBUG_CERTIFICATES_CF];

/// Definitions for the column families in the debug storage.
pub fn debug_db_cf_definitions() -> Vec<ColumnFamilyDescriptor> {
    crate::schema::default_db_cf_definitions(&CFS)
}
