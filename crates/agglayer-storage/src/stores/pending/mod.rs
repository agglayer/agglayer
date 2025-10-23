use std::{path::Path, sync::Arc};

use agglayer_types::{Certificate, CertificateId, Height, NetworkId, Proof};
use rocksdb::{Direction, ReadOptions};

use super::{PendingCertificateReader, PendingCertificateWriter};
use crate::{
    columns::{
        network_info::NetworkInfoColumn, pending_queue::{PendingQueueColumn, PendingQueueKey}, proof_per_certificate::ProofPerCertificateColumn
    }, error::Error, storage::DB, stores::{state::StateStore, NetworkInfoReader as _}, types::{network_info::{self, v0, BasicProvenCertificateInfo}, BasicPendingCertificateInfo}
};

/// A logical store for pending.
#[derive(Clone)]
pub struct PendingStore {
    db: Arc<DB>,
    state: Arc<StateStore>,
}

impl PendingStore {
    pub fn new(db: Arc<DB>, state: Arc<StateStore>) -> Self {
        Self { db, state }
    }
    pub fn new_with_path(path: &Path, state: Arc<StateStore>) -> Result<Self, Error> {
        let db = Arc::new(DB::open_cf(
            path,
            crate::storage::pending_db_cf_definitions(),
        )?);

        Ok(Self::new(db, state))
    }
}

impl PendingCertificateWriter for PendingStore {
    fn remove_pending_certificate(
        &self,
        network_id: NetworkId,
        height: Height,
    ) -> Result<(), Error> {
        Ok(self
            .db
            .delete::<PendingQueueColumn>(&PendingQueueKey(network_id, height))?)
    }

    fn set_latest_pending_certificate_info(
        &self,
        network_id: NetworkId,
        info: &BasicPendingCertificateInfo,
    ) -> Result<(), Error> {
        self.state.db.put::<NetworkInfoColumn>(
            &network_info::Key {
                network_id: network_id.into(),
                kind: v0::network_info_value::ValueDiscriminants::LatestPendingCertificateInfo
            },
            &network_info::Value {
                value: Some(v0::network_info_value::Value::LatestPendingCertificateInfo(v0::PendingCertificateInfo {
                    id: Some(info.certificate_id.into()),
                    height: Some(info.height.into()),
                })),
            },
        )?;
        Ok(())
    }

    fn insert_pending_certificate(
        &self,
        network_id: NetworkId,
        height: Height,
        certificate: &Certificate,
    ) -> Result<(), Error> {
        if let Some(i) =
            self.state.get_latest_pending_certificate_info(network_id)?
        {
            if i.height > height {
                // TODO: This is technically not Candidate error,
                return Err(Error::CertificateCandidateError(
                    crate::error::CertificateCandidateError::UnexpectedHeight(
                        network_id,
                        height,
                        i.height,
                    ),
                ));
            }
        }

        // TODO: make it batch
        self.set_latest_pending_certificate_info(network_id, &BasicPendingCertificateInfo { certificate_id: certificate.hash(), height })?;
        Ok(self
            .db
            .put::<PendingQueueColumn>(&PendingQueueKey(network_id, height), certificate)?)
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
        _network_id: &NetworkId,
        _height: &Height,
        _certificate_id: &CertificateId,
    ) -> Result<(), Error> {
        todo!() // TODO
    }
}

impl PendingCertificateReader for PendingStore {
    fn get_certificate(
        &self,
        network_id: NetworkId,
        height: Height,
    ) -> Result<Option<Certificate>, Error> {
        Ok(self
            .db
            .get::<PendingQueueColumn>(&PendingQueueKey(network_id, height))?)
    }

    fn get_proof(&self, certificate_id: CertificateId) -> Result<Option<Proof>, Error> {
        Ok(self.db.get::<ProofPerCertificateColumn>(&certificate_id)?)
    }

    fn get_current_proven_height(&self) -> Result<Vec<(NetworkId, BasicProvenCertificateInfo)>, Error> {
        let _: (Direction, ReadOptions);
        todo!() // TODO
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
        _network_id: &NetworkId,
    ) -> Result<Option<(NetworkId, Height, CertificateId)>, Error> {
        todo!() // TODO
    }

    fn multi_get_certificate(
        &self,
        keys: &[(NetworkId, Height)],
    ) -> Result<Vec<Option<Certificate>>, Error> {
        Ok(self
            .db
            .multi_get::<PendingQueueColumn>(keys.iter().map(|(n, h)| PendingQueueKey(*n, *h)))?)
    }

    fn multi_get_proof(&self, keys: &[CertificateId]) -> Result<Vec<Option<Proof>>, Error> {
        Ok(self
            .db
            .multi_get::<ProofPerCertificateColumn>(keys.iter().copied())?)
    }
}
