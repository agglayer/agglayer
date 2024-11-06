use std::{
    collections::{btree_map::Entry, BTreeMap},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use agglayer_types::{Certificate, CertificateIndex, EpochNumber, Height, NetworkId, Proof};
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
        proofs::ProofPerIndexColumn, start_checkpoint::StartCheckpointColumn,
    },
    error::{CertificateCandidateError, Error},
    storage::{epochs_db_cf_definitions, DB},
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
    packing_lock: RwLock<Option<EpochNumber>>,
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

        let mut closed = None;
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
                closed = Some(epoch_number);

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
            packing_lock: RwLock::new(closed),
        })
    }

    fn lock_for_adding_certificate(&self) -> RwLockReadGuard<Option<EpochNumber>> {
        self.packing_lock.read()
    }

    fn lock_for_packing(&self) -> RwLockWriteGuard<Option<EpochNumber>> {
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
    ) -> Result<(EpochNumber, CertificateIndex), Error> {
        let lock = self.lock_for_adding_certificate();

        if lock.is_some() {
            return Err(Error::AlreadyPacked(*self.epoch_number))?;
        }

        // Check for network rate limiting
        let start_checkpoint = self.start_checkpoint.get(&network_id);
        let mut end_checkpoint = self.end_checkpoint.write();

        debug!(
            "Try adding certificate for network {} at height {}",
            network_id, height
        );
        let end_checkpoint_entry = end_checkpoint.entry(network_id);

        let end_checkpoint_entry_assigment;

        // Fetch the network current point for this epoch
        match (start_checkpoint, &end_checkpoint_entry) {
            // If the network is not found in the end checkpoint, but is present in
            // the start checkpoint, this is an invalid state.
            (Some(_), Entry::Vacant(_)) => {
                warn!(
                    "Network {} is present in the start checkpoint but not in the end checkpoint",
                    network_id
                );
                return Err(Error::Unexpected(format!(
                    "Network {} is present in the start checkpoint but not in the end checkpoint",
                    network_id
                )));
            }
            // If the network is not found in the end checkpoint and the height is 0,
            // this is the first certificate for this network.
            (None, Entry::Vacant(_entry)) if height == 0 => {
                debug!("First certificate for network {}", network_id);
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
                    "Certificate candidate for network {} at height {} accepted",
                    network_id, height
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

        if let Some(epoch_number) = *lock {
            return Err(Error::AlreadyPacked(epoch_number))?;
        }

        let epoch_number = *self.epoch_number;
        // No more certificate can be added
        let iterator = self.db.iter_with_direction::<CertificatePerIndexColumn>(
            ReadOptions::default(),
            rocksdb::Direction::Forward,
        )?;
        let mut batch_status = Vec::new();

        for entry in iterator {
            if let Err(error) = entry {
                error!(
                    "CRITICAL error: Epoch {} contains a certificate that is unparsable: {}",
                    epoch_number, error
                );
                return Err(error);
            }

            let (index, certificate) = entry.unwrap();

            batch_status.push((certificate.hash(), index));
        }

        _ = *lock.insert(epoch_number);
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

        drop(lock);

        Ok(())
    }
}

impl<PendingStore, StateStore> PerEpochReader for PerEpochStore<PendingStore, StateStore>
where
    PendingStore: Send + Sync,
    StateStore: Send + Sync,
{
    fn get_epoch_number(&self) -> u64 {
        *self.epoch_number
    }
    fn get_certificate_at_index(
        &self,
        index: CertificateIndex,
    ) -> Result<Option<Certificate>, Error> {
        self.db.get::<CertificatePerIndexColumn>(&index)
    }

    fn get_proof_at_index(&self, index: CertificateIndex) -> Result<Option<Proof>, Error> {
        self.db.get::<ProofPerIndexColumn>(&index)
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
