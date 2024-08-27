use std::sync::Arc;

use super::{MetadataReader, MetadataWriter};
use crate::{
    columns::metadata::MetadataColumn,
    error::Error,
    storage::DB,
    types::{MetadataKey, MetadataValue},
};

#[cfg(test)]
mod tests;

/// A logical store for all the metadata.
pub struct MetadataStore {
    db: Arc<DB>,
}

impl MetadataStore {
    pub fn new(db: Arc<DB>) -> Self {
        Self { db }
    }
}

impl MetadataWriter for MetadataStore {
    fn set_latest_settled_epoch(&self, value: u64) -> Result<(), Error> {
        self.db.put::<MetadataColumn>(
            &MetadataKey::LatestSettledEpoch,
            &MetadataValue::LatestSettledEpoch(value),
        )
    }
}

impl MetadataReader for MetadataStore {
    fn get_latest_settled_epoch(&self) -> Result<Option<u64>, Error> {
        self.db
            .get::<MetadataColumn>(&MetadataKey::LatestSettledEpoch)
            .and_then(|v| {
                v.map_or(Ok(None), |v| match v {
                    MetadataValue::LatestSettledEpoch(value) => Ok(Some(value)),
                    MetadataValue::EpochSynchronization(_) => Err(Error::Unexpected(
                        "Wrong value type decoded, was expecting LastSettledEpoch, decoded \
                         EpochSynchronization"
                            .to_string(),
                    )),
                })
            })
    }
}
