use std::sync::Arc;

use agglayer_types::{Certificate, CertificateHeader, CertificateId, Height, NetworkId};
use rocksdb::{Direction, ReadOptions};

use super::{StateReader, StateWriter};
use crate::{
    columns::{
        certificate_per_network::{self, CertificatePerNetworkColumn},
        latest_certificate_per_network::{
            LatestSettledCertificatePerNetworkColumn, SettledCertificate,
        },
    },
    error::Error,
    storage::DB,
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
}

impl StateWriter for StateStore {
    fn insert_certificate_header(&self, certificate: &Certificate) -> Result<(), Error> {
        self.db.put::<CertificatePerNetworkColumn>(
            &certificate_per_network::Key {
                network_id: *certificate.network_id,
                height: certificate.height,
            },
            &CertificateHeader {
                certificate_id: certificate.hash(),
                network_id: certificate.network_id,
                height: certificate.height,
                epoch_number: None,
                certificate_index: None,
                new_local_exit_root: certificate.new_local_exit_root,
            },
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
    fn get_certificate_header_by_cursor(
        &self,
        network_id: NetworkId,
        height: Height,
    ) -> Result<Option<CertificateHeader>, Error> {
        self.db
            .get::<CertificatePerNetworkColumn>(&certificate_per_network::Key {
                network_id: *network_id,
                height,
            })
    }

    fn get_current_settled_height(&self) -> Result<Vec<(NetworkId, Height, CertificateId)>, Error> {
        Ok(self
            .db
            .iter_with_direction::<LatestSettledCertificatePerNetworkColumn>(
                ReadOptions::default(),
                Direction::Forward,
            )?
            .filter_map(|v| {
                v.map(|(network_id, SettledCertificate(id, height, _epoch))| {
                    (network_id, height, id)
                })
                .ok()
            })
            .collect())
    }
}
