use rocksdb::{ColumnFamilyDescriptor, DBCompressionType};

pub const CFS: [&str; 3] = [
    crate::columns::LATEST_PROVEN_CERTIFICATE_PER_NETWORK_CF,
    crate::columns::PENDING_QUEUE_CF,
    crate::columns::PROOF_PER_CERTIFICATE_CF,
];

/// Definitions for the column families in the pending queue storage.
pub fn pending_db_cf_definitions() -> Vec<ColumnFamilyDescriptor> {
    CFS.iter()
        .map(|cf| {
            let mut cfg = rocksdb::Options::default();

            cfg.set_compression_type(DBCompressionType::Lz4);
            cfg.create_if_missing(true);

            ColumnFamilyDescriptor::new(*cf, cfg)
        })
        .collect()
}
