use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use agglayer_types::{Certificate, CertificateIndex, EpochNumber, Height, NetworkId};
use parking_lot::RwLock;

use super::{
    interfaces::reader::PerEpochReader, per_epoch::PerEpochStore, EpochStoreReader,
    EpochStoreWriter, MetadataWriter, PendingCertificateReader, PendingCertificateWriter,
    StateReader, StateWriter,
};
use crate::{backup::BackupClient, error::Error};

pub struct EpochsStore<PendingStore, StateStore> {
    config: Arc<agglayer_config::Config>,
    #[allow(dead_code)]
    open_epochs: RwLock<BTreeSet<EpochNumber>>,
    pending_store: Arc<PendingStore>,
    state_store: Arc<StateStore>,
    backup_client: BackupClient,
}

impl<PendingStore, StateStore> EpochsStore<PendingStore, StateStore> {
    pub fn new(
        config: Arc<agglayer_config::Config>,
        epoch_number: EpochNumber,
        pending_store: Arc<PendingStore>,
        state_store: Arc<StateStore>,
        backup_client: BackupClient,
    ) -> Result<Self, Error> {
        let open_epochs = RwLock::new(BTreeSet::new());
        open_epochs.write().insert(epoch_number);

        Ok(Self {
            config,
            open_epochs,
            pending_store,
            state_store,
            backup_client,
        })
    }
}

impl<PendingStore, StateStore> EpochStoreWriter for EpochsStore<PendingStore, StateStore>
where
    PendingStore: PendingCertificateWriter + PendingCertificateReader,
    StateStore: StateWriter + StateReader + MetadataWriter,
{
    type PerEpochStore = PerEpochStore<PendingStore, StateStore>;
    fn open(
        &self,
        epoch_number: EpochNumber,
    ) -> Result<PerEpochStore<PendingStore, StateStore>, Error> {
        PerEpochStore::try_open(
            self.config.clone(),
            epoch_number,
            self.pending_store.clone(),
            self.state_store.clone(),
            None,
            self.backup_client.clone(),
        )
    }

    fn open_with_start_checkpoint(
        &self,
        epoch_number: EpochNumber,
        start_checkpoint: BTreeMap<NetworkId, Height>,
    ) -> Result<Self::PerEpochStore, Error> {
        PerEpochStore::try_open(
            self.config.clone(),
            epoch_number,
            self.pending_store.clone(),
            self.state_store.clone(),
            Some(start_checkpoint),
            self.backup_client.clone(),
        )
    }
}

impl<PendingStore, StateStore> EpochStoreReader for EpochsStore<PendingStore, StateStore>
where
    PendingStore: PendingCertificateReader,
    StateStore: StateReader,
{
    fn get_certificate(
        &self,
        epoch_number: EpochNumber,
        index: CertificateIndex,
    ) -> Result<Option<Certificate>, Error> {
        // Use readonly access to prevent concurrency issues when multiple processes
        // are accessing the database
        let per_epoch_store = PerEpochStore::try_open_readonly(
            self.config.clone(),
            epoch_number,
            self.pending_store.clone(),
            self.state_store.clone(),
        )?;
        per_epoch_store.get_certificate_at_index(index)
    }

    fn get_proof(
        &self,
        epoch_number: EpochNumber,
        index: CertificateIndex,
    ) -> Result<Option<agglayer_types::Proof>, Error> {
        // Use readonly access to prevent concurrency issues when multiple processes
        // are accessing the database
        let per_epoch_store = PerEpochStore::try_open_readonly(
            self.config.clone(),
            epoch_number,
            self.pending_store.clone(),
            self.state_store.clone(),
        )?;
        per_epoch_store.get_proof_at_index(index)
    }
}
