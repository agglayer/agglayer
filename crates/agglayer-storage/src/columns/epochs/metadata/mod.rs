use crate::columns::PER_EPOCH_METADATA_CF;

/// Column family for the metadata in an epoch.
///
/// | --- key ----- |    | --- value ------ |
/// | metadata key  | => | metadata value   |
pub struct ProofPerIndex;

impl crate::columns::ColumnSchema for ProofPerIndex {
    type Key = crate::types::PerEpochMetadataKey;
    type Value = crate::types::PerEpochMetadataValue;

    const COLUMN_FAMILY_NAME: &'static str = PER_EPOCH_METADATA_CF;
}
