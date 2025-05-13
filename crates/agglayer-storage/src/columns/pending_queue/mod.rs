use agglayer_types::{Certificate, Height, NetworkId};
use serde::{Deserialize, Serialize};

use super::{Codec, ColumnSchema, PENDING_QUEUE_CF};

/// Column family containing the pending certificates queue.
///
/// ## Column definition
///
/// | key                     | value           |
/// | --                      | --              |
/// | (`NetworkId`, `Height`) | `Certificate`   |
pub struct PendingQueueColumn;

#[derive(Debug, Serialize, Deserialize)]
pub struct PendingQueueKey(pub(crate) NetworkId, pub(crate) Height);

impl PendingQueueKey {
    pub fn new(network_id: NetworkId, height: Height) -> Self {
        Self(network_id, height)
    }
}

impl std::fmt::Display for PendingQueueKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}
impl Codec for PendingQueueKey {}

impl ColumnSchema for PendingQueueColumn {
    type Key = PendingQueueKey;
    type Value = Certificate;

    const COLUMN_FAMILY_NAME: &'static str = PENDING_QUEUE_CF;
}
