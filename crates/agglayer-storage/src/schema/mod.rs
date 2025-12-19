mod codec;
mod column;

pub(crate) use codec::impl_codec_using_bincode_for;
pub use codec::{bincode_codec, Codec, CodecError};
pub use column::ColumnSchema;

use rocksdb::ColumnFamilyDescriptor;

pub(crate) fn default_db_cf_definitions(cfs: &[&'static str]) -> Vec<ColumnFamilyDescriptor> {
    cfs.iter()
        .map(|cf| {
            let mut cfg = rocksdb::Options::default();

            cfg.set_compression_type(rocksdb::DBCompressionType::Lz4);
            cfg.create_if_missing(true);

            ColumnFamilyDescriptor::new(*cf, cfg)
        })
        .collect()
}
