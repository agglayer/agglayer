use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use agglayer_types::{Height, NetworkId};
use parking_lot::RwLock;

use super::{
    per_epoch::PerEpochStore, EpochStoreReader, EpochStoreWriter, MetadataWriter,
    PendingCertificateReader, PendingCertificateWriter, StateReader, StateWriter,
};
use crate::error::Error;

pub struct EpochsStore<PendingStore, StateStore> {
    config: Arc<agglayer_config::Config>,
    #[allow(dead_code)]
    open_epochs: RwLock<BTreeSet<u64>>,
    pending_store: Arc<PendingStore>,
    state_store: Arc<StateStore>,
}

impl<PendingStore, StateStore> EpochsStore<PendingStore, StateStore> {
    pub fn new(
        config: Arc<agglayer_config::Config>,
        epoch_number: u64,
        pending_store: Arc<PendingStore>,
        state_store: Arc<StateStore>,
    ) -> Result<Self, Error> {
        let open_epochs = RwLock::new(BTreeSet::new());
        open_epochs.write().insert(epoch_number);

        Ok(Self {
            config,
            open_epochs,
            pending_store,
            state_store,
        })
    }
}

impl<PendingStore, StateStore> EpochStoreWriter for EpochsStore<PendingStore, StateStore>
where
    PendingStore: PendingCertificateWriter + PendingCertificateReader,
    StateStore: StateWriter + StateReader + MetadataWriter,
{
    type PerEpochStore = PerEpochStore<PendingStore, StateStore>;
    fn open(&self, epoch_number: u64) -> Result<PerEpochStore<PendingStore, StateStore>, Error> {
        PerEpochStore::try_open(
            self.config.clone(),
            epoch_number,
            self.pending_store.clone(),
            self.state_store.clone(),
            None,
        )
    }

    fn open_with_start_checkpoint(
        &self,
        epoch_number: u64,
        start_checkpoint: BTreeMap<NetworkId, Height>,
    ) -> Result<Self::PerEpochStore, Error> {
        PerEpochStore::try_open(
            self.config.clone(),
            epoch_number,
            self.pending_store.clone(),
            self.state_store.clone(),
            Some(start_checkpoint),
        )
    }
}

impl<PendingStore, StateStore> EpochStoreReader for EpochsStore<PendingStore, StateStore>
where
    PendingStore: PendingCertificateReader + PendingCertificateWriter,
    StateStore: StateWriter + MetadataWriter + StateReader,
{
}
