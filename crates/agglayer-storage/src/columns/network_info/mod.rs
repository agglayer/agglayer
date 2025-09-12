use super::ColumnSchema;
use crate::columns::NETWORK_INFO_CF;

/// Column family that store network information.
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

    const COLUMN_FAMILY_NAME: &'static str = NETWORK_INFO_CF;
}
