use std::sync::Arc;

use agglayer_types::{Certificate, CertificateId, Height, NetworkId, Proof};

use super::{PendingCertificateReader, PendingCertificateWriter};
use crate::{
    columns::{
        pending_queue::{PendingQueueColumn, PendingQueueKey},
        proof_per_certificate::ProofPerCertificateColumn,
    },
    error::Error,
    storage::DB,
};

/// A logical store for pending.
#[derive(Debug, Clone)]
pub struct PendingStore {
    db: Arc<DB>,
}

impl PendingStore {
    pub fn new(db: Arc<DB>) -> Self {
        Self { db }
    }
}

impl PendingCertificateWriter for PendingStore {
    fn remove_pending_certificate(
        &self,
        network_id: NetworkId,
        height: Height,
    ) -> Result<(), Error> {
        self.db
            .delete::<PendingQueueColumn>(&PendingQueueKey(network_id, height))
    }

    fn insert_pending_certificate(
        &self,
        network_id: NetworkId,
        height: Height,
        certificate: &Certificate,
    ) -> Result<(), Error> {
        self.db
            .put::<PendingQueueColumn>(&PendingQueueKey(network_id, height), certificate)
    }

    fn insert_generated_proof(
        &self,
        certificate_id: &agglayer_types::CertificateId,
        proof: &agglayer_types::Proof,
    ) -> Result<(), Error> {
        self.db
            .put::<ProofPerCertificateColumn>(certificate_id, proof)
    }
}

impl PendingCertificateReader for PendingStore {
    fn get_certificate(
        &self,
        network_id: NetworkId,
        height: Height,
    ) -> Result<Option<Certificate>, Error> {
        self.db
            .get::<PendingQueueColumn>(&PendingQueueKey(network_id, height))
    }

    fn get_proof(&self, certificate_id: CertificateId) -> Result<Option<Proof>, Error> {
        self.db.get::<ProofPerCertificateColumn>(&certificate_id)
    }

    fn multi_get_certificate(
        &self,
        keys: &[(NetworkId, Height)],
    ) -> Result<Vec<Option<Certificate>>, Error> {
        self.db
            .multi_get::<PendingQueueColumn>(keys.iter().map(|(n, h)| PendingQueueKey(*n, *h)))
    }

    fn multi_get_proof(&self, keys: &[CertificateId]) -> Result<Vec<Option<Proof>>, Error> {
        self.db
            .multi_get::<ProofPerCertificateColumn>(keys.iter().copied())
    }
}
