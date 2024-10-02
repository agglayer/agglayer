use rocksdb::{ColumnFamilyDescriptor, DBCompressionType};

pub const CFS: [&str; 2] = [
    crate::columns::METADATA_CF,
    crate::columns::LATEST_SETTLED_CERTIFICATE_PER_NETWORK_CF,
];

/// Definitions for the column families in the metadata storage.
pub fn metadata_db_cf_definitions() -> Vec<ColumnFamilyDescriptor> {
    CFS.iter()
        .map(|cf| {
            let mut cfg = rocksdb::Options::default();

            cfg.set_compression_type(DBCompressionType::Lz4);
            cfg.create_if_missing(true);

            ColumnFamilyDescriptor::new(*cf, cfg)
        })
        .collect()
}
