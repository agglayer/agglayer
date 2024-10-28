use agglayer_types::NetworkId;
use rocksdb::ColumnFamilyDescriptor;

pub const CFS: [&str; 3] = [
    crate::columns::PER_EPOCH_CERTIFICATES_CF,
    crate::columns::PER_EPOCH_METADATA_CF,
    crate::columns::PER_EPOCH_PROOFS_CF,
];

const CHECKPOINTS: [&str; 2] = [
    crate::columns::PER_EPOCH_START_CHECKPOINT_CF,
    crate::columns::PER_EPOCH_END_CHECKPOINT_CF,
];

/// Definitions for the column families in the epochs storage.
pub fn epochs_db_cf_definitions() -> Vec<ColumnFamilyDescriptor> {
    let mut vec = super::default_db_cf_definitions(&CFS);

    let mut cfg = rocksdb::Options::default();

    cfg.set_compression_type(rocksdb::DBCompressionType::Lz4);
    cfg.create_if_missing(true);
    cfg.set_prefix_extractor(rocksdb::SliceTransform::create_fixed_prefix(
        NetworkId::BITS,
    ));

    for cf in &CHECKPOINTS {
        vec.push(ColumnFamilyDescriptor::new(*cf, cfg.clone()));
    }

    vec
}
