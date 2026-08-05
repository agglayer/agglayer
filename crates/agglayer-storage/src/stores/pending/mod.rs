use std::{path::Path, sync::Arc};

use agglayer_types::{Certificate, CertificateId, Height, NetworkId, Proof};
use rocksdb::{Direction, ReadOptions};

use super::{PendingCertificateReader, PendingCertificateWriter};
use crate::{
    columns::{
        latest_pending_certificate_per_network::{
            LatestPendingCertificatePerNetworkColumn, PendingCertificate,
        },
        latest_proven_certificate_per_network::{
            LatestProvenCertificatePerNetworkColumn, ProvenCertificate,
        },
        pending_queue::{PendingQueueColumn, PendingQueueKey, PendingQueueProtoColumn},
        proof_per_certificate::ProofPerCertificateColumn,
    },
    error::Error,
    schema::{Codec as _, ColumnDescriptor, ColumnSchema as _},
    storage::{DBError, DB},
};

pub(crate) mod cf_definitions;

#[cfg(test)]
mod tests;

/// A logical store for pending.
#[derive(Clone)]
pub struct PendingStore {
    db: Arc<DB>,
}

impl PendingStore {
    pub fn init_db(path: &Path) -> Result<DB, crate::storage::DBOpenError> {
        DB::builder(path, cf_definitions::PENDING_DB_V0)?
            .add_cfs(
                &[ColumnDescriptor::new::<PendingQueueProtoColumn>()],
                backfill_pending_certificates_proto_from_legacy_bincode,
            )?
            .finalize(cf_definitions::PENDING_DB)
    }

    pub fn new(db: Arc<DB>) -> Self {
        Self { db }
    }

    pub fn new_with_path(path: &Path) -> Result<Self, crate::storage::DBOpenError> {
        Ok(Self::new(Arc::new(Self::init_db(path)?)))
    }

    fn decode_readable_proof(certificate_id: CertificateId, bytes: &[u8]) -> Result<Proof, Error> {
        Proof::decode(bytes).map_err(|source| Error::UnreadableProof {
            id: certificate_id,
            source: DBError::from(source),
        })
    }

    fn get_readable_proof(&self, certificate_id: CertificateId) -> Result<Option<Proof>, Error> {
        let key = certificate_id.encode().map_err(DBError::from)?;
        let cf = self
            .db
            .raw_rocksdb()
            .cf_handle(ProofPerCertificateColumn::COLUMN_FAMILY_NAME)
            .ok_or(DBError::ColumnFamilyNotFound)?;

        let Some(bytes) = self
            .db
            .raw_rocksdb()
            .get_cf(&cf, key)
            .map_err(DBError::from)?
        else {
            return Ok(None);
        };

        Self::decode_readable_proof(certificate_id, &bytes).map(Some)
    }
}

/// Migration step for the certificate serialization switch from the legacy
/// pending queue CF to the proto-backed CF.
///
/// Delegates to
/// [`super::migration_helpers::copy_legacy_certificate_cf_into_proto`],
/// which streams the legacy keyspace, skips and logs rows whose bytes cannot
/// be decoded as a certificate, and copies the rest into the proto CF. The
/// legacy CF stays in place for now; runtime reads and writes only use
/// the proto CF after this backfill completes.
fn backfill_pending_certificates_proto_from_legacy_bincode(
    db: &crate::storage::DbAccess,
) -> Result<(), crate::storage::DBMigrationErrorDetails> {
    super::migration_helpers::copy_legacy_certificate_cf_into_proto::<
        PendingQueueColumn,
        PendingQueueProtoColumn,
    >(db, "pending")
}

impl PendingCertificateWriter for PendingStore {
    fn remove_pending_certificate(
        &self,
        network_id: NetworkId,
        height: Height,
    ) -> Result<(), Error> {
        let key = PendingQueueKey(network_id, height);
        Ok(self.db.delete::<PendingQueueProtoColumn>(&key)?)
    }
    fn set_latest_pending_certificate_per_network(
        &self,
        network_id: &NetworkId,
        height: &Height,
        certificate_id: &CertificateId,
    ) -> Result<(), Error> {
        Ok(self.db.put::<LatestPendingCertificatePerNetworkColumn>(
            network_id,
            &PendingCertificate(*certificate_id, *height),
        )?)
    }

    fn insert_pending_certificate(
        &self,
        network_id: NetworkId,
        height: Height,
        certificate: &Certificate,
    ) -> Result<(), Error> {
        if let Some((_id, latest_height)) =
            self.get_latest_pending_certificate_for_network(&network_id)?
        {
            if latest_height > height {
                return Err(Error::InvalidPendingHeight(
                    network_id,
                    height,
                    latest_height,
                ));
            }
        }

        // TODO: make it batch
        self.set_latest_pending_certificate_per_network(&network_id, &height, &certificate.hash())?;
        Ok(self
            .db
            .put::<PendingQueueProtoColumn>(&PendingQueueKey(network_id, height), certificate)?)
    }

    fn insert_generated_proof(
        &self,
        certificate_id: &agglayer_types::CertificateId,
        proof: &agglayer_types::Proof,
    ) -> Result<(), Error> {
        Ok(self
            .db
            .put::<ProofPerCertificateColumn>(certificate_id, proof)?)
    }

    fn remove_generated_proof(
        &self,
        certificate_id: &agglayer_types::CertificateId,
    ) -> Result<(), Error> {
        Ok(self
            .db
            .delete::<ProofPerCertificateColumn>(certificate_id)?)
    }

    fn set_latest_proven_certificate_per_network(
        &self,
        network_id: &NetworkId,
        height: &Height,
        certificate_id: &CertificateId,
    ) -> Result<(), Error> {
        Ok(self.db.put::<LatestProvenCertificatePerNetworkColumn>(
            network_id,
            &ProvenCertificate(*certificate_id, *network_id, *height),
        )?)
    }
}

impl PendingCertificateReader for PendingStore {
    fn get_latest_pending_certificate_for_network(
        &self,
        network_id: &NetworkId,
    ) -> Result<Option<(CertificateId, Height)>, Error> {
        Ok(self
            .db
            .get::<LatestPendingCertificatePerNetworkColumn>(network_id)
            .map(|v| v.map(|PendingCertificate(id, height)| (id, height)))?)
    }

    fn get_current_pending_heights(&self) -> Result<Vec<(NetworkId, PendingCertificate)>, Error> {
        Ok(self
            .db
            .iter_with_direction::<LatestPendingCertificatePerNetworkColumn>(
                ReadOptions::default(),
                Direction::Forward,
            )?
            .filter_map(|entry| entry.ok())
            .collect())
    }

    fn get_certificate(
        &self,
        network_id: NetworkId,
        height: Height,
    ) -> Result<Option<Certificate>, Error> {
        Ok(self
            .db
            .get::<PendingQueueProtoColumn>(&PendingQueueKey(network_id, height))?)
    }

    fn get_proof(&self, certificate_id: CertificateId) -> Result<Option<Proof>, Error> {
        self.get_readable_proof(certificate_id)
    }

    fn get_current_proven_height(&self) -> Result<Vec<ProvenCertificate>, Error> {
        Ok(self
            .db
            .iter_with_direction::<LatestProvenCertificatePerNetworkColumn>(
                ReadOptions::default(),
                Direction::Forward,
            )?
            .filter_map(|v| v.map(|(_, certificate)| certificate).ok())
            .collect())
    }

    fn get_current_proven_height_for_network(
        &self,
        network_id: &NetworkId,
    ) -> Result<Option<Height>, Error> {
        self.get_latest_proven_certificate_per_network(network_id)
            .map(|v| v.map(|(_network, height, _id)| height))
    }

    fn get_latest_proven_certificate_per_network(
        &self,
        network_id: &NetworkId,
    ) -> Result<Option<(NetworkId, Height, CertificateId)>, Error> {
        Ok(self
            .db
            .get::<LatestProvenCertificatePerNetworkColumn>(network_id)
            .map(|v| v.map(|ProvenCertificate(id, network, height)| (network, height, id)))?)
    }

    fn multi_get_certificate(
        &self,
        keys: &[(NetworkId, Height)],
    ) -> Result<Vec<Option<Certificate>>, Error> {
        Ok(self.db.multi_get::<PendingQueueProtoColumn>(
            keys.iter()
                .map(|(network_id, height)| PendingQueueKey(*network_id, *height)),
        )?)
    }

    fn multi_get_proof(&self, keys: &[CertificateId]) -> Result<Vec<Option<Proof>>, Error> {
        let cf = self
            .db
            .raw_rocksdb()
            .cf_handle(ProofPerCertificateColumn::COLUMN_FAMILY_NAME)
            .ok_or(Error::from(DBError::ColumnFamilyNotFound))?;

        let encoded_keys: Result<Vec<_>, _> = keys
            .iter()
            .map(|k| k.encode().map_err(DBError::from))
            .collect();

        let results = self
            .db
            .raw_rocksdb()
            .batched_multi_get_cf(cf, &encoded_keys?, false);

        results
            .into_iter()
            .zip(keys.iter())
            .map(|(result, certificate_id)| match result {
                Ok(Some(bytes)) => Self::decode_readable_proof(*certificate_id, &bytes).map(Some),
                Ok(None) => Ok(None),
                Err(error) => Err(Error::from(DBError::from(error))),
            })
            .collect()
    }
}
