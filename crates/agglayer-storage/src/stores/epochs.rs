use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use agglayer_types::{CertificateId, EpochNumber, Height, NetworkId, Proof};
use parking_lot::RwLock;

use super::{
    interfaces::reader::PerEpochReader, per_epoch::PerEpochStore, EpochStoreReader, EpochStoreWriter, MetadataWriter,
    PendingCertificateReader, PendingCertificateWriter, StateReader, StateWriter,
};
use crate::{error::Error, storage::backup::BackupClient};

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
    fn open(&self, epoch_number: EpochNumber) -> Result<PerEpochStore<PendingStore, StateStore>, Error> {
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
    PendingStore: PendingCertificateReader + PendingCertificateWriter,
    StateStore: StateWriter + MetadataWriter + StateReader,
{
    /// Get the proof for a certificate by certificate ID from the epoch store
    fn get_proof(&self, certificate_id: CertificateId) -> Result<Option<Proof>, Error> {
        // Get the certificate header to find which epoch it belongs to and its index
        let certificate_header = self.state_store
            .get_certificate_header(&certificate_id)?;
        
        if let Some(header) = certificate_header {
            if let (Some(epoch_number), Some(certificate_index)) = (header.epoch_number, header.certificate_index) {
                // Open the specific epoch store and get the proof by certificate index
                let epoch_store = self.open(epoch_number)?;
                return epoch_store.get_proof_at_index(certificate_index);
            }
        }
        
        // If not found in any epoch, return None
        Ok(None)
    }
}
