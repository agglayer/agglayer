use crate::columns::PER_EPOCH_METADATA_CF;

/// Column family for the metadata in an epoch.
///
/// ## Column definition
///
/// | key                   | value                   |
/// | --                    | --                      |
/// | `PerEpochMetadataKey` | `PerEpochMetadataValue` |
pub struct ProofPerIndex;

impl crate::columns::ColumnSchema for ProofPerIndex {
    type Key = crate::types::PerEpochMetadataKey;
    type Value = crate::types::PerEpochMetadataValue;

    const COLUMN_FAMILY_NAME: &'static str = PER_EPOCH_METADATA_CF;
}
