use std::{
    collections::{btree_map::Entry, BTreeMap},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use agglayer_types::{
    Certificate, CertificateId, CertificateIndex, EpochNumber, ExecutionMode, Height, NetworkId,
    Proof,
};
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use rocksdb::ReadOptions;
use tracing::{debug, error, warn};

use super::{
    interfaces::reader::PerEpochReader, MetadataWriter, PendingCertificateReader,
    PendingCertificateWriter, PerEpochWriter, StateReader, StateWriter,
};
use crate::{
    columns::epochs::{
        certificates::CertificatePerIndexColumn, end_checkpoint::EndCheckpointColumn,
        metadata::PerEpochMetadataColumn, proofs::ProofPerIndexColumn,
        start_checkpoint::StartCheckpointColumn,
    },
    error::{CertificateCandidateError, Error},
    storage::{backup::BackupClient, DB},
    types::{PerEpochMetadataKey, PerEpochMetadataValue},
};

mod cf_definitions;

#[cfg(test)]
mod tests;

const MAX_CERTIFICATE_PER_EPOCH: u64 = 1;

/// A logical store for an Epoch.
pub struct PerEpochStore<PendingStore, StateStore> {
    pub epoch_number: Arc<EpochNumber>,
    db: Arc<DB>,
    pending_store: Arc<PendingStore>,
    state_store: Arc<StateStore>,
    next_certificate_index: AtomicU64,
    start_checkpoint: BTreeMap<NetworkId, Height>,
    end_checkpoint: RwLock<BTreeMap<NetworkId, Height>>,
    packing_lock: RwLock<bool>,
    backup_client: BackupClient,
}

impl<PendingStore, StateStore> PerEpochStore<PendingStore, StateStore> {
    pub fn init_db(path: &std::path::Path) -> Result<DB, crate::storage::DBOpenError> {
        DB::open_cf(path, cf_definitions::epochs_db_cf_definitions())
    }

    pub fn init_db_readonly(path: &std::path::Path) -> Result<DB, crate::storage::DBError> {
        DB::open_cf_readonly(path, cf_definitions::epochs_db_cf_definitions())
    }

    #[tracing::instrument(skip_all, fields(store = "epoch", %epoch_number))]
    pub fn try_open(
        config: Arc<agglayer_config::Config>,
        epoch_number: EpochNumber,
        pending_store: Arc<PendingStore>,
        state_store: Arc<StateStore>,
        optional_start_checkpoint: Option<BTreeMap<NetworkId, Height>>,
        backup_client: BackupClient,
    ) -> Result<Self, Error> {
        let db = Arc::new(
            Self::init_db(&config.storage.epoch_db_path(epoch_number))
                .map_err(Error::DBOpenError)?,
        );

        Self::try_open_with_db(
            db,
            epoch_number,
            pending_store,
            state_store,
            optional_start_checkpoint,
            backup_client,
            false, // readonly mode
        )
    }

    /// Open a PerEpochStore in read-only mode to prevent concurrency issues.
    /// This is useful for operations that only need to read data from the database.
    #[tracing::instrument(skip_all, fields(store = "epoch", %epoch_number))]
    pub fn try_open_readonly(
        config: Arc<agglayer_config::Config>,
        epoch_number: EpochNumber,
        pending_store: Arc<PendingStore>,
        state_store: Arc<StateStore>,
    ) -> Result<Self, Error> {
        let db = Arc::new(Self::init_db_readonly(
            &config.storage.epoch_db_path(epoch_number),
        )?);

        Self::try_open_with_db(
            db,
            epoch_number,
            pending_store,
            state_store,
            None,                 // No start checkpoint for readonly
            BackupClient::noop(), // No backup needed for readonly access
            true,                 // readonly mode
        )
    }

    /// Common initialization logic for both read-write and read-only modes
    fn try_open_with_db(
        db: Arc<DB>,
        epoch_number: EpochNumber,
        pending_store: Arc<PendingStore>,
        state_store: Arc<StateStore>,
        optional_start_checkpoint: Option<BTreeMap<NetworkId, Height>>,
        backup_client: BackupClient,
        readonly: bool,
    ) -> Result<Self, Error> {
        // Check if the epoch is already packed, if no value is found, the epoch is not packed
        let packed = db
            .get::<PerEpochMetadataColumn>(&PerEpochMetadataKey::Packed)?
            .map(|value| match value {
                PerEpochMetadataValue::SettlementTxHash(_digest) => Err(Error::Unexpected(
                    "Tried to retrieve the status of an epoch, retrieve another unexpected value"
                        .to_string(),
                )),
                PerEpochMetadataValue::Packed(value) => Ok(value),
            })
            .transpose()?
            .unwrap_or_default();

        let start_checkpoint = {
            let checkpoint = db
                .iter_with_direction::<StartCheckpointColumn>(
                    ReadOptions::default(),
                    rocksdb::Direction::Forward,
                )?
                .filter_map(|v| v.ok())
                .collect::<BTreeMap<NetworkId, Height>>();

            if readonly {
                // For readonly access, we just use the existing checkpoint
                checkpoint
            } else {
                // For read-write access, handle optional_start_checkpoint
                match optional_start_checkpoint {
                    Some(expected_start_checkpoint) => {
                        if checkpoint.is_empty() {
                            db.multi_insert::<StartCheckpointColumn>(&expected_start_checkpoint)?;
                            expected_start_checkpoint
                        } else if checkpoint != expected_start_checkpoint {
                            warn!(
                                "Start checkpoint doesn't match the expected one; refusing to open epoch \
                                 due to inconsistent state",
                            );
                            return Err(Error::Unexpected(
                                "Start checkpoint doesn't match the expected one; inconsistent epoch \
                                 state in DB"
                                    .to_string(),
                            ))?;
                        } else {
                            checkpoint
                        }
                    }
                    None => checkpoint,
                }
            }
        };

        let next_certificate_index = if readonly {
            // For readonly access, we don't need to track the next index
            AtomicU64::new(0)
        } else {
            // For read-write access, calculate the next index from existing certificates
            if let Some(Ok((index, _))) = db
                .iter_with_direction::<CertificatePerIndexColumn>(
                    ReadOptions::default(),
                    rocksdb::Direction::Reverse,
                )?
                .next()
            {
                // We're starting from the next index after the last one found in the database.
                AtomicU64::new(index.as_u64() + 1)
            } else {
                AtomicU64::new(0)
            }
        };

        let end_checkpoint = {
            let checkpoint = db
                .iter_with_direction::<EndCheckpointColumn>(
                    ReadOptions::default(),
                    rocksdb::Direction::Forward,
                )?
                .filter_map(|v| v.ok())
                .collect::<BTreeMap<NetworkId, Height>>();

            if readonly {
                // For readonly access, just use the existing checkpoint
                checkpoint
            } else {
                // For read-write access, handle empty checkpoint
                if checkpoint.is_empty() {
                    if next_certificate_index.load(Ordering::Relaxed) != 0 {
                        return Err(Error::Unexpected(
                            "End checkpoint is empty, but there are certificates in the DB"
                                .to_string(),
                        ))?;
                    }

                    db.multi_insert::<EndCheckpointColumn>(&start_checkpoint)?;
                    start_checkpoint.clone()
                } else {
                    checkpoint
                }
            }
        };

        Ok(Self {
            epoch_number: Arc::new(epoch_number),
            db,
            next_certificate_index,
            pending_store,
            state_store,
            start_checkpoint,
            end_checkpoint: RwLock::new(end_checkpoint),
            packing_lock: RwLock::new(packed),
            backup_client,
        })
    }

    fn lock_for_adding_certificate(&self) -> RwLockReadGuard<'_, bool> {
        self.packing_lock.read()
    }
    fn lock_for_packing(&self) -> RwLockWriteGuard<'_, bool> {
        self.packing_lock.write()
    }
}

impl<PendingStore, StateStore> PerEpochWriter for PerEpochStore<PendingStore, StateStore>
where
    PendingStore: PendingCertificateReader + PendingCertificateWriter,
    StateStore: MetadataWriter + StateWriter + StateReader,
{
    fn add_certificate(
        &self,
        certificate_id: CertificateId,
        mode: ExecutionMode,
    ) -> Result<(EpochNumber, CertificateIndex), Error> {
        let lock = self.lock_for_adding_certificate();

        if *lock {
            return Err(Error::AlreadyPacked(*self.epoch_number))?;
        }

        let certificate_header = self
            .state_store
            .get_certificate_header(&certificate_id)?
            .ok_or(Error::NoCertificateHeader)?;

        let network_id = certificate_header.network_id;
        let height = certificate_header.height;

        let certificate = self
            .pending_store
            .get_certificate(network_id, height)?
            .ok_or(Error::NoCertificate)?;

        let certificate_id = if certificate_id != certificate.hash() {
            error!(
                "Inconsistent certificate context for network {} and certificate {}",
                network_id, certificate_id
            );

            return Err(Error::CertificateCandidateError(
                CertificateCandidateError::InconsistentCertificateContext(
                    network_id,
                    certificate_id,
                ),
            ))?;
        } else {
            certificate_id
        };
        // Check for network rate limiting
        let start_checkpoint = self.start_checkpoint.get(&network_id);
        let mut end_checkpoint = self.end_checkpoint.write();

        debug!(
            "{}Try adding certificate for network {} at height {} in epoch {}",
            mode.prefix(),
            network_id,
            height,
            self.epoch_number
        );
        let end_checkpoint_entry = end_checkpoint.entry(network_id);

        let end_checkpoint_entry_assigment;

        // Fetch the network current point for this epoch
        match (start_checkpoint, &end_checkpoint_entry) {
            // If the network is not found in the end checkpoint, but is present in
            // the start checkpoint, this is an invalid state.
            (Some(_), Entry::Vacant(_)) => {
                warn!(
                    "{}Network {} is present in the start checkpoint but not in the end checkpoint",
                    mode.prefix(),
                    network_id
                );
                return Err(Error::Unexpected(format!(
                    "{}Network {} is present in the start checkpoint but not in the end checkpoint",
                    mode.prefix(),
                    network_id
                )));
            }
            // If the network is not found in the end checkpoint and the height is 0,
            // this is the first certificate for this network.
            (None, Entry::Vacant(_entry)) if height == Height::ZERO => {
                debug!(
                    "{}First certificate for network {}",
                    mode.prefix(),
                    network_id
                );
                // Adding the network to the end checkpoint.
                end_checkpoint_entry_assigment = Some(Height::ZERO);

                // Adding the certificate to the DB
            }
            // If the network is not found in the end checkpoint and the height is not 0,
            // this is an invalid certificate candidate and the operation should fail.
            (None, Entry::Vacant(_)) => {
                return Err(CertificateCandidateError::Invalid(network_id, height))?
            }
            // If the network is found in the end checkpoint and the height is 0,
            // this is an invalid certificate candidate and the operation should fail.
            (Some(_start_height), Entry::Occupied(ref current_height))
                if height == Height::ZERO =>
            {
                return Err(CertificateCandidateError::UnexpectedHeight(
                    network_id,
                    height,
                    *current_height.get(),
                ))?
            }
            // If the network is found in the end checkpoint and the height minus one is equal to
            // the current network height. We can add the certificate.
            (Some(start_height), Entry::Occupied(current_height))
                if current_height.get().next() == height
                    && height.distance_since(start_height) <= MAX_CERTIFICATE_PER_EPOCH =>
            {
                debug!(
                    "{}Certificate candidate for network {} at height {} accepted",
                    mode.prefix(),
                    network_id,
                    height
                );

                end_checkpoint_entry_assigment = Some(height);
            }

            (_, Entry::Occupied(current_height)) => {
                return Err(CertificateCandidateError::UnexpectedHeight(
                    network_id,
                    height,
                    *current_height.get(),
                ))?
            }
        }

        if mode == ExecutionMode::DryRun {
            // If this is a dry run, we don't want to add the certificate to the DB
            // The certificate index is informal
            return Ok((
                *self.epoch_number,
                CertificateIndex::new(self.next_certificate_index.load(Ordering::Relaxed)),
            ));
        }

        let proof = self
            .pending_store
            .get_proof(certificate_id)?
            .ok_or(Error::NoProof)
            .inspect_err(|_| {
                error!(
                    "CRITICAL: No proof found for certificate {} manual action may be needed",
                    certificate_id
                )
            })?;

        let certificate_index =
            CertificateIndex::new(self.next_certificate_index.fetch_add(1, Ordering::SeqCst));

        // TODO: all of this need to be batched

        // Adding the certificate and proof to the current epoch store
        self.db
            .put::<CertificatePerIndexColumn>(&certificate_index, &certificate)?;

        self.db
            .put::<ProofPerIndexColumn>(&certificate_index, &proof)?;

        // Removing the certificate and proof from the pending store
        self.pending_store.remove_generated_proof(&certificate_id)?;

        self.pending_store
            .remove_pending_certificate(network_id, height)?;
        debug!(
            %certificate_id,
            "Certificate and proof removed from pending store"
        );

        self.state_store.assign_certificate_to_epoch(
            &certificate_id,
            &self.epoch_number,
            &certificate_index,
        )?;

        debug!(
            %certificate_id,
            epoch_number = %self.epoch_number,
            "Certificate assigned to epoch"
        );
        if let Some(height) = end_checkpoint_entry_assigment {
            let entry = end_checkpoint_entry.or_default();
            *entry = height;

            debug!(
                "Updating end checkpoint for network {} to height {}",
                network_id, height
            );

            self.db.put::<EndCheckpointColumn>(&network_id, &height)?;
        }

        drop(lock);

        Ok((*self.epoch_number, certificate_index))
    }

    fn start_packing(&self) -> Result<(), Error> {
        let mut lock = self.lock_for_packing();

        if *lock {
            return Err(Error::AlreadyPacked(*self.epoch_number))?;
        }

        self.db.put::<PerEpochMetadataColumn>(
            &PerEpochMetadataKey::Packed,
            &PerEpochMetadataValue::Packed(true),
        )?;

        if let Err(error) = self
            .backup_client
            .backup(crate::storage::backup::BackupRequest {
                epoch_db: Some((self.db.clone(), *self.epoch_number)),
            })
        {
            error!("Couldn't trigger the backup of the epoch DB: {}", error);
        }

        *lock = true;
        match self
            .state_store
            .set_latest_settled_epoch(*self.epoch_number)
        {
            Err(Error::UnprocessedAction(error)) => {
                warn!("Couldn't define the latest settled epoch: {}", error)
            }
            Err(error) => return Err(error),
            Ok(_) => (),
        }

        Ok(())
    }
}

impl<PendingStore, StateStore> PerEpochReader for PerEpochStore<PendingStore, StateStore>
where
    PendingStore: Send + Sync,
    StateStore: Send + Sync,
{
    fn is_epoch_packed(&self) -> bool {
        *self.lock_for_adding_certificate()
    }

    fn get_epoch_number(&self) -> EpochNumber {
        *self.epoch_number
    }
    fn get_certificate_at_index(
        &self,
        index: CertificateIndex,
    ) -> Result<Option<Certificate>, Error> {
        Ok(self.db.get::<CertificatePerIndexColumn>(&index)?)
    }

    fn get_proof_at_index(&self, index: CertificateIndex) -> Result<Option<Proof>, Error> {
        Ok(self.db.get::<ProofPerIndexColumn>(&index)?)
    }

    fn get_start_checkpoint(&self) -> &BTreeMap<NetworkId, Height> {
        &self.start_checkpoint
    }

    fn get_end_checkpoint(&self) -> BTreeMap<NetworkId, Height> {
        self.end_checkpoint.read().clone()
    }

    fn get_end_checkpoint_height_per_network(
        &self,
        network_id: NetworkId,
    ) -> Result<Option<Height>, Error> {
        Ok(self.end_checkpoint.read().get(&network_id).copied())
    }
}
