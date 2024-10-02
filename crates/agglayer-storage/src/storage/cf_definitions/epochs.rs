use rocksdb::{ColumnFamilyDescriptor, DBCompressionType};

pub const CFS: [&str; 3] = [
    crate::columns::PER_EPOCH_CERTIFICATES_CF,
    crate::columns::PER_EPOCH_METADATA_CF,
    crate::columns::PER_EPOCH_PROOFS_CF,
];

/// Definitions for the column families in the epochs storage.
pub fn epochs_db_cf_definitions() -> Vec<ColumnFamilyDescriptor> {
    CFS.iter()
        .map(|cf| {
            let mut cfg = rocksdb::Options::default();

            cfg.set_compression_type(DBCompressionType::Lz4);
            cfg.create_if_missing(true);

            ColumnFamilyDescriptor::new(*cf, cfg)
        })
        .collect()
}
