use super::{ColumnSchema, METADATA_CF};

/// Column family that store metadata related to the node.
///
/// ## Column definition
///
/// | key                  | value                  |
/// | --                   | --                     |
/// | `network_id::Key`    | `network_Info::Value`  |
pub struct NetworkInfoColumn;

impl ColumnSchema for NetworkInfoColumn {
    type Key = crate::types::network_info::Key;
    type Value = crate::types::network_info::Value;

    const COLUMN_FAMILY_NAME: &'static str = METADATA_CF;
}
