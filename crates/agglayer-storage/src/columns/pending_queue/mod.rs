use bincode::Options;
use serde::{Deserialize, Serialize};

use super::{default_bincode_options, Codec, ColumnSchema, PENDING_QUEUE_CF};
use crate::{
    error::Error,
    types::{Certificate, Height, NetworkId},
};

/// Column family containing the pending certificates queue.
///
/// | --- key ------------ |    | --- value ---------------- |
/// | network id + height  | => | Certificate bytes array    |
pub(crate) struct PendingQueueColumn;

#[derive(Debug, Serialize, Deserialize)]
pub struct PendingQueueKey(pub(crate) NetworkId, pub(crate) Height);

impl Codec for PendingQueueKey {
    fn encode(&self) -> Result<Vec<u8>, Error> {
        Ok(default_bincode_options().serialize(self)?)
    }

    fn decode(buf: &[u8]) -> Result<Self, Error> {
        Ok(default_bincode_options().deserialize(buf)?)
    }
}

impl ColumnSchema for PendingQueueColumn {
    type Key = PendingQueueKey;
    type Value = Certificate;

    const COLUMN_FAMILY_NAME: &'static str = PENDING_QUEUE_CF;
}
