use std::{
    collections::{btree_map::Entry, BTreeMap},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
};

use agglayer_types::{CertificateIndex, EpochNumber, Height, NetworkId};
use parking_lot::RwLock;
use rocksdb::ReadOptions;
use tracing::{debug, warn};

use super::{
    interfaces::reader::PerEpochReader, PendingCertificateReader, PendingCertificateWriter,
    PerEpochWriter, StateWriter,
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
    epoch_number: Arc<u64>,
    db: Arc<DB>,
    pending_store: Arc<PendingStore>,
    state_store: Arc<StateStore>,
    next_certificate_index: AtomicU64,
    start_checkpoint: BTreeMap<NetworkId, Height>,
    end_checkpoint: RwLock<BTreeMap<NetworkId, Height>>,
    in_packing: AtomicBool,
}

impl<PendingStore, StateStore> PerEpochStore<PendingStore, StateStore> {
    pub fn try_open(
        config: Arc<agglayer_config::Config>,
        epoch_number: u64,
        pending_store: Arc<PendingStore>,
        state_store: Arc<StateStore>,
    ) -> Result<Self, Error> {
        // TODO: refactor this
        let path = config
            .storage
            .epochs_db_path
            .join(format!("{}", epoch_number));

        let db = Arc::new(DB::open_cf(&path, epochs_db_cf_definitions())?);

        let start_checkpoint = if epoch_number == 0 {
            BTreeMap::new()
        } else {
            let checkpoint = db
                .iter_with_direction::<StartCheckpointColumn>(
                    ReadOptions::default(),
                    rocksdb::Direction::Forward,
                )?
                .filter_map(|v| v.ok())
                .collect::<BTreeMap<NetworkId, Height>>();

            if checkpoint.is_empty() {
                return Err(Error::Unexpected(format!(
                    "Epoch {} doesn't seem to have a start checkpoint...",
                    epoch_number
                )))?;
            }

            checkpoint
        };

        let mut in_packing = false;

        let end_checkpoint = {
            let checkpoint = db
                .iter_with_direction::<EndCheckpointColumn>(
                    ReadOptions::default(),
                    rocksdb::Direction::Forward,
                )?
                .filter_map(|v| v.ok())
                .collect::<BTreeMap<NetworkId, Height>>();

            if checkpoint.is_empty() {
                start_checkpoint.clone()
            } else {
                in_packing = true;

                checkpoint
            }
        };

        Ok(Self {
            epoch_number: Arc::new(epoch_number),
            db,
            pending_store,
            state_store,
            next_certificate_index: AtomicU64::new(0),
            start_checkpoint,
            end_checkpoint: RwLock::new(end_checkpoint),
            in_packing: AtomicBool::new(in_packing),
        })
    }
}

impl<PendingStore, StateStore> PerEpochWriter for PerEpochStore<PendingStore, StateStore>
where
    PendingStore: PendingCertificateReader + PendingCertificateWriter,
    StateStore: StateWriter,
{
    fn add_certificate(
        &self,
        network_id: NetworkId,
        height: Height,
    ) -> Result<(EpochNumber, CertificateIndex), Error> {
        // Check for network rate limiting
        let start_checkpoint = self.start_checkpoint.get(&network_id);
        let mut end_checkpoint = self.end_checkpoint.write();

        debug!(
            "Try adding certificate for network {} at height {}",
            network_id, height
        );

        // Fetch the network current point for this epoch
        match (start_checkpoint, end_checkpoint.entry(network_id)) {
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
            (None, Entry::Vacant(entry)) if height == 0 => {
                debug!("First certificate for network {}", network_id);
                // Adding the network to the end checkpoint.
                entry.insert(0);

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
                    && start_height - height <= MAX_CERTIFICATE_PER_EPOCH =>
            {
                debug!(
                    "Certificate candidate for network {} at height {} accepted",
                    network_id, height
                );
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

        self.state_store.update_certificate_header_status(
            &certificate_id,
            &agglayer_types::CertificateStatus::Candidate,
        )?;

        Ok((*self.epoch_number, certificate_index))
    }

    fn start_packing(&self) -> Result<(), Error> {
        self.in_packing
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::Acquire)
            .map_err(|_| Error::AlreadyInPackingMode)?;

        Ok(())
    }
}

impl<PendingStore, StateStore> PerEpochReader for PerEpochStore<PendingStore, StateStore>
where
    PendingStore: Send + Sync,
    StateStore: Send + Sync,
{
    fn get_start_checkpoint(&self) -> &BTreeMap<NetworkId, Height> {
        &self.start_checkpoint
    }

    fn get_end_checkpoint_height_per_network(
        &self,
        network_id: NetworkId,
    ) -> Result<Option<Height>, Error> {
        Ok(self.end_checkpoint.read().get(&network_id).copied())
    }
}
