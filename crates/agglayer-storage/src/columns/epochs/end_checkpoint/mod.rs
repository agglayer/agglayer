use agglayer_types::{Height, NetworkId};

use crate::columns::PER_EPOCH_END_CHECKPOINT_CF;

/// Column family for the end checkpoint in an epoch.
///
/// ## Column definition
///
/// | key         | value    |
/// | --          | --       |
/// | `NetworkId` | `Height` |
pub struct EndCheckpointColumn;

impl crate::columns::ColumnSchema for EndCheckpointColumn {
    type Key = NetworkId;
    type Value = Height;

    const COLUMN_FAMILY_NAME: &'static str = PER_EPOCH_END_CHECKPOINT_CF;
}
