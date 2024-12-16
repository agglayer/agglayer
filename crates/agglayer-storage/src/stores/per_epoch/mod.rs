use std::{
    collections::{btree_map::Entry, BTreeMap},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use agglayer_types::{
    Certificate, CertificateIndex, EpochNumber, ExecutionMode, Height, NetworkId, Proof,
};
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use rocksdb::ReadOptions;
use tracing::{debug, warn};

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
    storage::{epochs_db_cf_definitions, DB},
    types::{PerEpochMetadataKey, PerEpochMetadataValue},
};

#[cfg(test)]
mod tests;

const MAX_CERTIFICATE_PER_EPOCH: u64 = 1;

/// A logical store for an Epoch.
pub struct PerEpochStore<PendingStore, StateStore> {
    pub epoch_number: Arc<u64>,
    db: Arc<DB>,
    pending_store: Arc<PendingStore>,
    state_store: Arc<StateStore>,
    next_certificate_index: AtomicU64,
    start_checkpoint: BTreeMap<NetworkId, Height>,
    end_checkpoint: RwLock<BTreeMap<NetworkId, Height>>,
    packing_lock: RwLock<bool>,
}

impl<PendingStore, StateStore> PerEpochStore<PendingStore, StateStore> {
    pub fn try_open(
        config: Arc<agglayer_config::Config>,
        epoch_number: u64,
        pending_store: Arc<PendingStore>,
        state_store: Arc<StateStore>,
        optional_start_checkpoint: Option<BTreeMap<NetworkId, Height>>,
    ) -> Result<Self, Error> {
        // TODO: refactor this
        let path = config
            .storage
            .epochs_db_path
            .join(format!("{}", epoch_number));

        let db = Arc::new(DB::open_cf(&path, epochs_db_cf_definitions())?);

        // Check if the epoch is already packed, if no value is found, the epoch is not
        // packed
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

            match optional_start_checkpoint {
                Some(expected_start_checkpoint) => {
                    if checkpoint.is_empty() {
                        db.multi_insert::<StartCheckpointColumn>(&expected_start_checkpoint)?;
                        expected_start_checkpoint
                    } else if checkpoint != expected_start_checkpoint {
                        warn!(
                            "Start checkpoint doesn't match the expected one, using the one from \
                             the DB"
                        );
                        return Err(Error::Unexpected(
                            "Start checkpoint doesn't match the expected one, using the one from \
                             the DB"
                                .to_string(),
                        ))?;
                    } else {
                        checkpoint
                    }
                }
                None => checkpoint,
            }
        };

        let next_certificate_index = if let Some(Ok((index, _))) = db
            .iter_with_direction::<CertificatePerIndexColumn>(
                ReadOptions::default(),
                rocksdb::Direction::Reverse,
            )?
            .next()
        {
            AtomicU64::new(index)
        } else {
            AtomicU64::new(0)
        };

        let end_checkpoint = {
            let checkpoint = db
                .iter_with_direction::<EndCheckpointColumn>(
                    ReadOptions::default(),
                    rocksdb::Direction::Forward,
                )?
                .filter_map(|v| v.ok())
                .collect::<BTreeMap<NetworkId, Height>>();

            if checkpoint.is_empty() {
                if next_certificate_index.load(Ordering::Relaxed) != 0 {
                    return Err(Error::Unexpected(
                        "End checkpoint is empty, but there are certificates in the DB".to_string(),
                    ))?;
                }

                db.multi_insert::<EndCheckpointColumn>(&start_checkpoint)?;

                start_checkpoint.clone()
            } else {
                checkpoint
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
        })
    }

    fn lock_for_adding_certificate(&self) -> RwLockReadGuard<bool> {
        self.packing_lock.read()
    }
    fn lock_for_packing(&self) -> RwLockWriteGuard<bool> {
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
        network_id: NetworkId,
        height: Height,
        mode: ExecutionMode,
    ) -> Result<(EpochNumber, CertificateIndex), Error> {
        let lock = self.lock_for_adding_certificate();

        if *lock {
            return Err(Error::AlreadyPacked(*self.epoch_number))?;
        }

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
            (None, Entry::Vacant(_entry)) if height == 0 => {
                debug!(
                    "{}First certificate for network {}",
                    mode.prefix(),
                    network_id
                );
                // Adding the network to the end checkpoint.
                end_checkpoint_entry_assigment = Some(0);

                // Adding the certificate to the DB
            }
            // If the network is not found in the end checkpoint and the height is not 0,
            // this is an invalid certificate candidate and the operation should fail.
            (None, Entry::Vacant(_)) => {
                return Err(CertificateCandidateError::Invalid(network_id, height))?
            }
            // If the network is found in the end checkpoint and the height is 0,
            // this is an invalid certificate candidate and the operation should fail.
            (Some(_start_height), Entry::Occupied(ref current_height)) if height == 0 => {
                return Err(CertificateCandidateError::UnexpectedHeight(
                    network_id,
                    height,
                    *current_height.get(),
                ))?
            }
            // If the network is found in the end checkpoint and the height minus one is equal to
            // the current network height. We can add the certificate.
            (Some(start_height), Entry::Occupied(current_height))
                if *current_height.get() == height - 1
                    && height - start_height <= MAX_CERTIFICATE_PER_EPOCH =>
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
                self.next_certificate_index.load(Ordering::Relaxed),
            ));
        }

        // Acquire locks
        let certificate = self
            .pending_store
            .get_certificate(network_id, height)?
            .ok_or(Error::NoCertificate)?;

        let certificate_id = certificate.hash();

        let proof = self
            .pending_store
            .get_proof(certificate_id)?
            .ok_or(Error::NoProof)?;

        let certificate_index = self.next_certificate_index.fetch_add(1, Ordering::SeqCst);

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

        self.state_store.assign_certificate_to_epoch(
            &certificate_id,
            &self.epoch_number,
            &certificate_index,
        )?;

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

    fn get_epoch_number(&self) -> u64 {
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
