use agglayer_types::{Height, NetworkId};

use crate::columns::PER_EPOCH_START_CHECKPOINT_CF;

/// Column family for the start checkpoint in an epoch.
///
/// ## Column definition
///
/// | key         | value    |
/// | --          | --       |
/// | `NetworkId` | `Height` |
pub struct StartCheckpointColumn;

impl crate::columns::ColumnSchema for StartCheckpointColumn {
    type Key = NetworkId;
    type Value = Height;

    const COLUMN_FAMILY_NAME: &'static str = PER_EPOCH_START_CHECKPOINT_CF;
}
