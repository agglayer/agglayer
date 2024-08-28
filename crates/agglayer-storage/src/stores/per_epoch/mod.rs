use std::sync::{atomic::AtomicU64, Arc};

use agglayer_types::{Height, NetworkId};

use super::PerEpochWriter;
use crate::{
    columns::{
        epochs::{certificates::CertificatePerIndexColumn, proofs::ProofPerIndexColumn},
        pending_queue::{PendingQueueColumn, PendingQueueKey},
        proof_per_certificate::ProofPerCertificateColumn,
    },
    error::Error,
    storage::{epochs_db_cf_definitions, DB},
};

/// A logical store for an Epoch.
pub struct PerEpochStore {
    db: Arc<DB>,
    pending_db: Arc<DB>,
    next_certificate_index: AtomicU64,
}

impl PerEpochStore {
    pub fn open(
        config: Arc<agglayer_config::Config>,
        epoch_number: u64,
        pending_db: Arc<DB>,
    ) -> Result<Self, Error> {
        // TODO: refactor this
        let path = config.storage.epochs_path.join(format!("{}", epoch_number));

        let db = Arc::new(DB::open_cf(&path, epochs_db_cf_definitions())?);

        Ok(Self {
            db,
            pending_db,
            next_certificate_index: AtomicU64::new(0),
        })
    }
}

impl PerEpochWriter for PerEpochStore {
    fn add_certificate(&self, network_id: NetworkId, height: Height) -> Result<(), Error> {
        let certificate = self
            .pending_db
            .get::<PendingQueueColumn>(&PendingQueueKey(network_id, height))?
            .ok_or(Error::NoCertificate)?;

        let certificate_id = certificate.hash();

        let proof = self
            .pending_db
            .get::<ProofPerCertificateColumn>(&certificate_id)?
            .ok_or(Error::NoProof)?;

        let certificate_index = self
            .next_certificate_index
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        // TODO: all of this need to be batched

        // Adding the certificate and proof to the current epoch store
        self.db
            .put::<CertificatePerIndexColumn>(&certificate_index, &certificate)?;

        self.db
            .put::<ProofPerIndexColumn>(&certificate_index, &proof)?;

        // Removing the certificate and proof from the pending store
        self.pending_db
            .delete::<ProofPerCertificateColumn>(&certificate_id)?;

        self.pending_db
            .delete::<PendingQueueColumn>(&PendingQueueKey(network_id, height))?;

        Ok(())
    }
}
