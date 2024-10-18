use std::{collections::BTreeSet, sync::Arc};

use arc_swap::ArcSwap;
use parking_lot::RwLock;

use super::{per_epoch::PerEpochStore, EpochStoreReader, EpochStoreWriter};
use crate::{error::Error, storage::DB};

pub struct EpochsStore {
    config: Arc<agglayer_config::Config>,
    current_epoch: Arc<ArcSwap<PerEpochStore>>,
    #[allow(dead_code)]
    open_epochs: RwLock<BTreeSet<u64>>,
    pending_db: Arc<DB>,
}

impl EpochsStore {
    pub fn new(
        config: Arc<agglayer_config::Config>,
        epoch_number: u64,
        pending_db: Arc<DB>,
    ) -> Result<Self, Error> {
        let current_epoch = Arc::new(ArcSwap::new(Arc::new(PerEpochStore::try_open(
            config.clone(),
            epoch_number,
            pending_db.clone(),
        )?)));

        let open_epochs = RwLock::new(BTreeSet::new());
        open_epochs.write().insert(epoch_number);

        Ok(Self {
            config,
            open_epochs,
            current_epoch,
            pending_db,
        })
    }
}
impl EpochStoreWriter for EpochsStore {
    type PerEpochStore = PerEpochStore;

    fn open(&self, epoch_number: u64) -> Result<PerEpochStore, Error> {
        PerEpochStore::try_open(self.config.clone(), epoch_number, self.pending_db.clone())
    }
}

impl EpochStoreReader for EpochsStore {
    type PerEpochStore = PerEpochStore;

    fn get_current_epoch(&self) -> Arc<ArcSwap<Self::PerEpochStore>> {
        self.current_epoch.clone()
    }
}
