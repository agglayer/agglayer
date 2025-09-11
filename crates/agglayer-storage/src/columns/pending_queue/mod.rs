use agglayer_types::{Certificate, Height, NetworkId};
use serde::{Deserialize, Serialize};

use super::{ColumnSchema, PENDING_QUEUE_CF};

/// Column family containing the pending certificates queue.
///
/// ## Column definition
///
/// | key                     | value           |
/// | --                      | --              |
/// | (`NetworkId`, `Height`) | `Certificate`   |
pub(crate) struct PendingQueueColumn;

#[derive(Debug, Serialize, Deserialize)]
pub struct PendingQueueKey(pub(crate) NetworkId, pub(crate) Height);

crate::columns::impl_codec_using_bincode_for!(PendingQueueKey);

impl ColumnSchema for PendingQueueColumn {
    type Key = PendingQueueKey;
    type Value = Certificate;

    const COLUMN_FAMILY_NAME: &'static str = PENDING_QUEUE_CF;
}
