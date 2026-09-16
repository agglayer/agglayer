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
use rocksdb::{ReadOptions, WriteBatch};
use tracing::{debug, error, instrument, warn};

use super::{
    interfaces::reader::PerEpochReader, MetadataWriter, PendingCertificateReader,
    PendingCertificateWriter, PerEpochWriter, StateReader, StateWriter,
};
use crate::{
    backup::BackupClient,
    columns::epochs::{
        certificates::{CertificatePerIndexColumn, CertificatePerIndexProtoColumn},
        end_checkpoint::EndCheckpointColumn,
        metadata::PerEpochMetadataColumn,
        proofs::ProofPerIndexColumn,
        start_checkpoint::StartCheckpointColumn,
    },
    error::{CertificateCandidateError, Error},
    schema::ColumnDescriptor,
    storage::DB,
    types::{PerEpochMetadataKey, PerEpochMetadataValue},
};

pub(crate) mod cf_definitions;

#[cfg(test)]
mod tests;

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
        DB::builder(path, cf_definitions::EPOCHS_DB_V0)?
            .add_cfs(
                &[ColumnDescriptor::new::<CertificatePerIndexProtoColumn>()],
                backfill_epoch_certificates_proto_from_legacy_bincode,
            )?
            .finalize(cf_definitions::EPOCHS_DB)
    }

    pub fn init_db_readonly(path: &std::path::Path) -> Result<DB, crate::storage::DBError> {
        DB::open_cf_readonly(path, cf_definitions::EPOCHS_DB)
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
    /// This is useful for operations that only need to read data from the
    /// database.
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
        // Check if the epoch is already packed, if no value is found, the epoch
        // is not packed
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
                                "Start checkpoint doesn't match the expected one; refusing to \
                                 open epoch due to inconsistent state",
                            );
                            return Err(Error::Unexpected(
                                "Start checkpoint doesn't match the expected one; inconsistent \
                                 epoch state in DB"
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
            // For read-write access, calculate the next index from existing
            // certificates. The proto-backed CF is the single
            // source of truth at runtime: legacy rows
            // are backfilled into it during `init_db`, so consulting the legacy
            // CF here would only re-read data that has already been
            // migrated.
            if let Some(Ok((index, _))) = db
                .iter_with_direction::<CertificatePerIndexProtoColumn>(
                    ReadOptions::default(),
                    rocksdb::Direction::Reverse,
                )?
                .next()
            {
                // We're starting from the next index after the last one found
                // in the database.
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
                        Err(Error::Unexpected(
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

impl<PendingStore, StateStore> PerEpochStore<PendingStore, StateStore>
where
    PendingStore: PendingCertificateReader + PendingCertificateWriter,
    StateStore: StateWriter,
{
    fn find_persisted_certificate_index(
        &self,
        certificate_id: CertificateId,
    ) -> Result<Option<CertificateIndex>, Error> {
        for entry in self.db.iter_with_direction::<CertificatePerIndexProtoColumn>(
            ReadOptions::default(),
            rocksdb::Direction::Reverse,
        )? {
            let (index, certificate) = entry?;
            if certificate.hash() == certificate_id {
                return Ok(Some(index));
            }
        }

        Ok(None)
    }

    fn validate_persisted_certificate(
        &self,
        certificate_id: CertificateId,
        network_id: NetworkId,
        height: Height,
        certificate_index: CertificateIndex,
    ) -> Result<(), Error> {
        let certificate = self
            .db
            .get::<CertificatePerIndexProtoColumn>(&certificate_index)?
            .ok_or_else(|| {
                Error::Unexpected(format!(
                    "Certificate {certificate_id} is assigned to epoch {} at index {certificate_index}, but the epoch certificate row is missing",
                    self.epoch_number
                ))
            })?;

        if certificate.hash() != certificate_id
            || certificate.network_id != network_id
            || certificate.height != height
        {
            return Err(Error::Unexpected(format!(
                "Certificate {certificate_id} does not match the epoch row at index {certificate_index}"
            )));
        }

        if self
            .db
            .get::<ProofPerIndexColumn>(&certificate_index)?
            .is_none()
        {
            return Err(Error::Unexpected(format!(
                "Certificate {certificate_id} is persisted at epoch index {certificate_index}, but its proof is missing"
            )));
        }

        Ok(())
    }

    fn finish_recovered_certificate(
        &self,
        certificate_id: CertificateId,
        network_id: NetworkId,
        height: Height,
        certificate_index: CertificateIndex,
        assign_state: bool,
    ) -> Result<(EpochNumber, CertificateIndex), Error> {
        self.validate_persisted_certificate(
            certificate_id,
            network_id,
            height,
            certificate_index,
        )?;

        if assign_state {
            self.state_store.assign_certificate_to_epoch(
                &certificate_id,
                &self.epoch_number,
                &certificate_index,
            )?;
        }

        self.cleanup_pending_certificate(certificate_id, network_id, height)?;

        Ok((*self.epoch_number, certificate_index))
    }

    fn cleanup_pending_certificate(
        &self,
        certificate_id: CertificateId,
        network_id: NetworkId,
        height: Height,
    ) -> Result<(), Error> {
        // The proof is keyed by certificate id, so deleting it is safe and
        // idempotent even if the retry happens after it was already removed.
        self.pending_store.remove_generated_proof(&certificate_id)?;

        // The pending body is keyed only by network + height. Avoid deleting a
        // different certificate that may have replaced this row while the
        // settled transition was being completed.
        if let Some(pending_certificate) =
            self.pending_store.get_certificate(network_id, height)?
        {
            if pending_certificate.hash() == certificate_id {
                self.pending_store
                    .remove_pending_certificate(network_id, height)?;
            } else {
                warn!(
                    %certificate_id,
                    %network_id,
                    %height,
                    pending_certificate_id = %pending_certificate.hash(),
                    "Pending row was replaced while completing epoch assignment; leaving it intact"
                );
            }
        }

        Ok(())
    }
}

/// Migration step for the certificate serialization switch from the legacy
/// epoch certificate CF to the proto-backed CF.
///
/// Delegates to
/// [`super::migration_helpers::copy_legacy_certificate_cf_into_proto`],
/// which streams the legacy keyspace, skips and logs rows whose bytes cannot
/// be decoded as a certificate, and copies the rest into the proto CF. The
/// original CF is left untouched so we can finish validation before
/// removing it in a later cleanup step.
fn backfill_epoch_certificates_proto_from_legacy_bincode(
    db: &crate::storage::DbAccess,
) -> Result<(), crate::storage::DBMigrationErrorDetails> {
    super::migration_helpers::copy_legacy_certificate_cf_into_proto::<
        CertificatePerIndexColumn,
        CertificatePerIndexProtoColumn,
    >(db, "epoch")
}

impl<PendingStore, StateStore> PerEpochWriter for PerEpochStore<PendingStore, StateStore>
where
    PendingStore: PendingCertificateReader + PendingCertificateWriter,
    StateStore: MetadataWriter + StateWriter + StateReader,
{
    #[instrument(skip(self), fields(epoch_number = %self.epoch_number))]
    fn add_certificate(
        &self,
        certificate_id: CertificateId,
        mode: ExecutionMode,
    ) -> Result<(EpochNumber, CertificateIndex), Error> {
        let lock = self.lock_for_adding_certificate();

        if *lock {
            Err(Error::AlreadyPacked(*self.epoch_number))?;
        }

        let certificate_header = self
            .state_store
            .get_certificate_header(&certificate_id)?
            .ok_or(Error::NoCertificateHeader)?;

        let network_id = certificate_header.network_id;
        let height = certificate_header.height;

        if mode == ExecutionMode::Default {
            match (
                certificate_header.epoch_number,
                certificate_header.certificate_index,
            ) {
                (Some(epoch_number), Some(certificate_index))
                    if epoch_number == *self.epoch_number =>
                {
                    return self.finish_recovered_certificate(
                        certificate_id,
                        network_id,
                        height,
                        certificate_index,
                        false,
                    );
                }
                (Some(epoch_number), Some(certificate_index)) => {
                    return Err(Error::UnprocessedAction(format!(
                        "Certificate {certificate_id} is already assigned to epoch {epoch_number} at index {certificate_index}"
                    )));
                }
                (Some(_), None) | (None, Some(_)) => {
                    return Err(Error::Unexpected(format!(
                        "Certificate {certificate_id} has an incomplete epoch assignment"
                    )));
                }
                (None, None) => {}
            }
        }

        // Check for network rate limiting.
        let start_checkpoint = self.start_checkpoint.get(&network_id);
        let mut end_checkpoint = self.end_checkpoint.write();

        debug!(
            "{}Try adding certificate for network {} at height {} in epoch {}",
            mode.prefix(),
            network_id,
            height,
            self.epoch_number
        );

        // The epoch-local batch is the durable first phase of the transition.
        // If it committed but state assignment or pending cleanup did not, the
        // checkpoint already points at this height. Resume from the persisted
        // certificate instead of allocating a second index.
        if mode == ExecutionMode::Default
            && end_checkpoint.get(&network_id).copied() == Some(height)
        {
            if let Some(certificate_index) =
                self.find_persisted_certificate_index(certificate_id)?
            {
                return self.finish_recovered_certificate(
                    certificate_id,
                    network_id,
                    height,
                    certificate_index,
                    true,
                );
            }
        }

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

        let end_checkpoint_entry = end_checkpoint.entry(network_id);

        let end_checkpoint_entry_assignment;

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
                end_checkpoint_entry_assignment = Some(Height::ZERO);
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
                debug!(
                    "{}Failed certificate candidate for network {}: height is {} but network is \
                     already present in the end checkpoint with height {}",
                    mode.prefix(),
                    network_id,
                    height,
                    current_height.get()
                );
                return Err(CertificateCandidateError::UnexpectedHeight(
                    network_id,
                    height,
                    *current_height.get(),
                )
                .into());
            }
            // If the network is found in the end checkpoint and the height minus one is equal to
            // the current network height. We can add the certificate.
            (_, Entry::Occupied(current_height)) if current_height.get().next() == height => {
                debug!(
                    "{}Certificate candidate for network {} at height {} accepted",
                    mode.prefix(),
                    network_id,
                    height
                );

                end_checkpoint_entry_assignment = Some(height);
            }

            (_, Entry::Occupied(current_height)) => {
                debug!(
                    "{}Failed certificate candidate for network {}: current height is {} \
                     submitted certificate height is {}",
                    mode.prefix(),
                    network_id,
                    current_height.get(),
                    height,
                );
                return Err(CertificateCandidateError::UnexpectedHeight(
                    network_id,
                    height,
                    *current_height.get(),
                )
                .into());
            }
        }

        if mode == ExecutionMode::DryRun {
            // If this is a dry run, we don't want to add the certificate to the
            // DB The certificate index is informal
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
            CertificateIndex::new(self.next_certificate_index.load(Ordering::SeqCst));
        let mut batch = WriteBatch::default();

        // Certificate, proof, and checkpoint live in the same epoch RocksDB, so
        // commit them as one durable phase. Cross-DB state/pending updates are
        // completed idempotently below and can be resumed on retry.
        self.db
            .multi_insert_batch::<CertificatePerIndexProtoColumn>(
                [(&certificate_index, &certificate)],
                &mut batch,
            )?;
        self.db.multi_insert_batch::<ProofPerIndexColumn>(
            [(&certificate_index, &proof)],
            &mut batch,
        )?;
        if let Some(height) = end_checkpoint_entry_assignment {
            self.db.multi_insert_batch::<EndCheckpointColumn>(
                [(&network_id, &height)],
                &mut batch,
            )?;
        }
        self.db.write_batch(batch)?;

        self.next_certificate_index.fetch_add(1, Ordering::SeqCst);

        if let Some(height) = end_checkpoint_entry_assignment {
            let entry = end_checkpoint_entry.or_default();
            *entry = height;
            debug!(
                "Updated end checkpoint for network {} to height {}",
                network_id, height
            );
        }

        // Keep pending data intact until the state header is durably assigned.
        // If assignment or cleanup fails, a retry detects the persisted epoch
        // row above and resumes from this same index.
        self.state_store.assign_certificate_to_epoch(
            &certificate_id,
            &self.epoch_number,
            &certificate_index,
        )?;
        debug!("Certificate assigned to epoch");

        self.cleanup_pending_certificate(certificate_id, network_id, height)?;
        debug!("Certificate and proof cleaned up from pending store");

        drop(lock);

        Ok((*self.epoch_number, certificate_index))
    }

    fn start_packing(&self) -> Result<(), Error> {
        let mut lock = self.lock_for_packing();

        if *lock {
            Err(Error::AlreadyPacked(*self.epoch_number))?;
        }

        self.db.put::<PerEpochMetadataColumn>(
            &PerEpochMetadataKey::Packed,
            &PerEpochMetadataValue::Packed(true),
        )?;

        if let Err(error) = self
            .backup_client
            .backup_epoch(self.db.clone(), *self.epoch_number)
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
        match self.db.get::<CertificatePerIndexProtoColumn>(&index) {
            // Epoch DBs created before the proto migration and only reopened
            // read-only since were never migrated: read-only opens never create
            // column families, so the proto CF is absent here. Fall back to the
            // still-present legacy CF, decoding through the same `Certificate::from`
            // conversion the migration backfill uses.
            Err(crate::storage::DBError::ColumnFamilyNotFound) => {
                warn!(
                    "Proto certificate CF missing for epoch {}: reading from the legacy CF",
                    self.epoch_number
                );
                Ok(self
                    .db
                    .get::<CertificatePerIndexColumn>(&index)?
                    .map(Certificate::from))
            }
            result => Ok(result?),
        }
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
