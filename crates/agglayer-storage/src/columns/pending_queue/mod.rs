use agglayer_types::{Certificate, Height, NetworkId};
use serde::{Deserialize, Serialize};

use super::{ColumnSchema, PENDING_QUEUE_CF, PENDING_QUEUE_PROTO_CF};
use crate::types::LegacyCertificate;

/// Legacy column family for the pending certificates queue.
///
/// Kept readable so the proto migration can backfill existing rows. The CF
/// historically received both bincode rows and (later) proto rows, so its
/// `Value` codec is `LegacyCertificate`, which
/// accepts both. Runtime reads and writes go through
/// [`PendingQueueProtoColumn`].
///
/// **Transitional:** this CF will be dropped in a follow-up ticket once the
/// proto migration has been validated in production.
///
/// ## Column definition
///
/// | key                     | value               |
/// | --                      | --                  |
/// | (`NetworkId`, `Height`) | `LegacyCertificate` |
pub(crate) struct PendingQueueColumn;

#[derive(Debug, Serialize, Deserialize)]
pub struct PendingQueueKey(pub(crate) NetworkId, pub(crate) Height);

crate::schema::impl_codec_using_bincode_for!(PendingQueueKey);

impl ColumnSchema for PendingQueueColumn {
    type Key = PendingQueueKey;
    type Value = LegacyCertificate;

    const COLUMN_FAMILY_NAME: &'static str = PENDING_QUEUE_CF;
}

/// Proto-backed column family containing the pending certificates queue.
///
/// ## Column definition
///
/// | key                     | value         |
/// | --                      | --            |
/// | (`NetworkId`, `Height`) | `Certificate` |
pub(crate) struct PendingQueueProtoColumn;

impl ColumnSchema for PendingQueueProtoColumn {
    type Key = PendingQueueKey;
    type Value = Certificate;

    const COLUMN_FAMILY_NAME: &'static str = PENDING_QUEUE_PROTO_CF;
}
