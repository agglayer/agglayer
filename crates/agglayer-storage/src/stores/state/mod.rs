use std::{path::Path, sync::Arc};

use agglayer_types::{
    Certificate, CertificateHeader, CertificateId, CertificateStatus, EpochNumber, Height,
    NetworkId,
};
use rocksdb::{Direction, ReadOptions};
use tracing::warn;

use super::{MetadataReader, MetadataWriter, StateReader, StateWriter};
use crate::{
    columns::{
        certificate_header::CertificateHeaderColumn,
        certificate_per_network::{self, CertificatePerNetworkColumn},
        latest_settled_certificate_per_network::{
            LatestSettledCertificatePerNetworkColumn, SettledCertificate,
        },
        metadata::MetadataColumn,
    },
    error::Error,
    storage::DB,
    types::{MetadataKey, MetadataValue},
};

#[cfg(test)]
mod tests;

/// A logical store for the state.
pub struct StateStore {
    db: Arc<DB>,
}

impl StateStore {
    pub fn new(db: Arc<DB>) -> Self {
        Self { db }
    }

    pub fn new_with_path(path: &Path) -> Result<Self, Error> {
        let db = Arc::new(DB::open_cf(
            path,
            crate::storage::state_db_cf_definitions(),
        )?);

        Ok(Self { db })
    }
}

impl StateWriter for StateStore {
    fn insert_certificate_header(
        &self,
        certificate: &Certificate,
        status: CertificateStatus,
    ) -> Result<(), Error> {
        // TODO: make it a batch write
        self.db.put::<CertificateHeaderColumn>(
            &certificate.hash(),
            &CertificateHeader {
                certificate_id: certificate.hash(),
                network_id: certificate.network_id,
                height: certificate.height,
                epoch_number: None,
                certificate_index: None,
                new_local_exit_root: certificate.new_local_exit_root.into(),
                status: status.clone(),
                metadata: certificate.metadata,
            },
        )?;

        if let CertificateStatus::Settled = status {
            // TODO: Check certificate conflict during insert (if conflict it's too late)
            self.db.put::<CertificatePerNetworkColumn>(
                &certificate_per_network::Key {
                    network_id: *certificate.network_id,
                    height: certificate.height,
                },
                &certificate.hash(),
            )?;
        }

        Ok(())
    }

    fn update_certificate_header_status(
        &self,
        certificate_id: &CertificateId,
        status: &CertificateStatus,
    ) -> Result<(), Error> {
        // TODO: make lockguard for certificate_id
        let certificate_header = self.db.get::<CertificateHeaderColumn>(certificate_id)?;

        if let Some(mut certificate_header) = certificate_header {
            certificate_header.status = status.clone();
            self.db
                .put::<CertificateHeaderColumn>(certificate_id, &certificate_header)?;

            if let CertificateStatus::Settled = status {
                self.db.put::<CertificatePerNetworkColumn>(
                    &certificate_per_network::Key {
                        network_id: *certificate_header.network_id,
                        height: certificate_header.height,
                    },
                    &certificate_header.certificate_id,
                )?;
            }
        }

        Ok(())
    }

    fn set_latest_settled_certificate_for_network(
        &self,
        network_id: &NetworkId,
        certificate_id: &CertificateId,
        epoch_number: &EpochNumber,
        height: &Height,
    ) -> Result<(), Error> {
        self.db.put::<LatestSettledCertificatePerNetworkColumn>(
            network_id,
            &SettledCertificate(*certificate_id, *height, *epoch_number),
        )
    }
}

impl StateReader for StateStore {
    /// Get the active networks.
    /// Meaning, the networks that have at least one submitted certificate.
    ///
    /// Performance: O(n) where n is the number of networks.
    /// This is because we need to scan all the keys in the
    /// `last_certificate_per_network` column family.
    /// This is not a problem because the number of networks is expected to be
    /// small. This function is only called once when the node starts.
    /// Benchmark: `last_certificate_bench.rs`
    fn get_active_networks(&self) -> Result<Vec<NetworkId>, Error> {
        Ok(self
            .db
            .keys::<LatestSettledCertificatePerNetworkColumn>()?
            .filter_map(|v| v.ok())
            .collect())
    }

    fn get_certificate_header(
        &self,
        certificate_id: &CertificateId,
    ) -> Result<Option<CertificateHeader>, Error> {
        tracing::info!("get_certificate_header: {}", certificate_id);
        self.db.get::<CertificateHeaderColumn>(certificate_id)
    }

    fn get_certificate_header_by_cursor(
        &self,
        network_id: NetworkId,
        height: Height,
    ) -> Result<Option<CertificateHeader>, Error> {
        self.db
            .get::<CertificatePerNetworkColumn>(&certificate_per_network::Key {
                network_id: *network_id,
                height,
            })?
            .map_or(Ok(None), |certificate_id| {
                let result = self.get_certificate_header(&certificate_id);

                if let Ok(None) = result {
                    warn!(
                        "Certificate header not found for certificate_id: {} while having a \
                         reference in the CertificatePerNetworkColumn",
                        certificate_id
                    );
                }

                result
            })
    }

    fn get_current_settled_height(
        &self,
    ) -> Result<Vec<(NetworkId, Height, CertificateId, EpochNumber)>, Error> {
        Ok(self
            .db
            .iter_with_direction::<LatestSettledCertificatePerNetworkColumn>(
                ReadOptions::default(),
                Direction::Forward,
            )?
            .filter_map(|v| {
                v.map(|(network_id, SettledCertificate(id, height, epoch))| {
                    (network_id, height, id, epoch)
                })
                .ok()
            })
            .collect())
    }
}

impl MetadataWriter for StateStore {
    fn set_latest_settled_epoch(&self, value: u64) -> Result<(), Error> {
        if let Some(current_latest_settled_epoch) = self.get_latest_settled_epoch()? {
            if current_latest_settled_epoch >= value {
                return Err(Error::UnprocessedAction(
                    "Tried to set a lower value for latest settled epoch".to_string(),
                ));
            }
        }

        self.db.put::<MetadataColumn>(
            &MetadataKey::LatestSettledEpoch,
            &MetadataValue::LatestSettledEpoch(value),
        )
    }
}

impl MetadataReader for StateStore {
    fn get_latest_settled_epoch(&self) -> Result<Option<u64>, Error> {
        self.db
            .get::<MetadataColumn>(&MetadataKey::LatestSettledEpoch)
            .and_then(|v| {
                v.map_or(Ok(None), |v| match v {
                    MetadataValue::LatestSettledEpoch(value) => Ok(Some(value)),
                    _ => Err(Error::Unexpected(
                        "Wrong value type decoded, was expecting LastSettledEpoch, decoded \
                         another type"
                            .to_string(),
                    )),
                })
            })
    }
}
