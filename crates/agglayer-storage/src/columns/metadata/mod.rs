use super::{ColumnSchema, METADATA_CF};

/// Column family that store metadata related to the node.
///
/// ## Column definition
///
/// | key           | value           |
/// | --            | --              |
/// | `MetadataKey` | `MetadataValue` |
pub struct MetadataColumn;

impl ColumnSchema for MetadataColumn {
    type Key = crate::types::MetadataKey;
    type Value = crate::types::MetadataValue;

    const COLUMN_FAMILY_NAME: &'static str = METADATA_CF;
}
