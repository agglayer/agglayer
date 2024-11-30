use rocksdb::ColumnFamilyDescriptor;

pub mod debug;
pub mod epochs;
pub mod pending;
pub mod state;

fn default_db_cf_definitions(cfs: &[&'static str]) -> Vec<ColumnFamilyDescriptor> {
    cfs.iter()
        .map(|cf| {
            let mut cfg = rocksdb::Options::default();

            cfg.set_compression_type(rocksdb::DBCompressionType::Lz4);
            cfg.create_if_missing(true);

            ColumnFamilyDescriptor::new(*cf, cfg)
        })
        .collect()
}
