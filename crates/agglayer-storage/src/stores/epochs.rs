use std::{collections::BTreeSet, sync::Arc};

use arc_swap::ArcSwap;
use parking_lot::RwLock;

use super::{
    per_epoch::PerEpochStore, EpochStoreReader, EpochStoreWriter, PendingCertificateReader,
    PendingCertificateWriter, StateWriter,
};
use crate::error::Error;

pub struct EpochsStore<PendingStore, StateStore> {
    config: Arc<agglayer_config::Config>,
    current_epoch: Arc<ArcSwap<PerEpochStore<PendingStore, StateStore>>>,
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
        let current_epoch = Arc::new(ArcSwap::new(Arc::new(PerEpochStore::try_open(
            config.clone(),
            epoch_number,
            pending_store.clone(),
            state_store.clone(),
        )?)));

        let open_epochs = RwLock::new(BTreeSet::new());
        open_epochs.write().insert(epoch_number);

        Ok(Self {
            config,
            open_epochs,
            current_epoch,
            pending_store,
            state_store,
        })
    }
}

impl<PendingStore, StateStore> EpochStoreWriter for EpochsStore<PendingStore, StateStore>
where
    PendingStore: Send + Sync,
    StateStore: Send + Sync,
{
    type PerEpochStore = PerEpochStore<PendingStore, StateStore>;

    fn open(&self, epoch_number: u64) -> Result<PerEpochStore<PendingStore, StateStore>, Error> {
        PerEpochStore::try_open(
            self.config.clone(),
            epoch_number,
            self.pending_store.clone(),
            self.state_store.clone(),
        )
    }
}

impl<PendingStore, StateStore> EpochStoreReader for EpochsStore<PendingStore, StateStore>
where
    PendingStore: PendingCertificateReader + PendingCertificateWriter,
    StateStore: StateWriter,
{
    type PerEpochStore = PerEpochStore<PendingStore, StateStore>;

    fn get_current_epoch(&self) -> Arc<ArcSwap<Self::PerEpochStore>> {
        self.current_epoch.clone()
    }
}
