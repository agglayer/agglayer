use std::{collections::BTreeMap, sync::Arc};

use agglayer_types::{
    Certificate, CertificateHeader, CertificateId, EpochNumber, Height, NetworkId, Proof,
};
use arc_swap::ArcSwap;

use crate::{
    columns::latest_proven_certificate_per_network::ProvenCertificate, error::Error,
    stores::PerEpochWriter,
};

pub trait EpochStoreReader: Send + Sync {
    type PerEpochStore: PerEpochReader + PerEpochWriter;

    fn get_current_epoch(&self) -> Arc<ArcSwap<Self::PerEpochStore>>;
}

pub trait PendingCertificateReader: Send + Sync {
    fn get_certificate(
        &self,
        network_id: NetworkId,
        height: Height,
    ) -> Result<Option<Certificate>, Error>;

    fn get_proof(&self, certificate_id: CertificateId) -> Result<Option<Proof>, Error>;

    fn multi_get_certificate(
        &self,
        keys: &[(NetworkId, Height)],
    ) -> Result<Vec<Option<Certificate>>, Error>;

    fn multi_get_proof(&self, keys: &[CertificateId]) -> Result<Vec<Option<Proof>>, Error>;
    fn get_current_proven_height(&self) -> Result<Vec<ProvenCertificate>, Error>;
}

pub trait MetadataReader: Send + Sync {
    /// Get the latest settled epoch.
    fn get_latest_settled_epoch(&self) -> Result<Option<u64>, Error>;
}

pub trait StateReader: Send + Sync {
    /// Get the active networks.
    fn get_active_networks(&self) -> Result<Vec<NetworkId>, Error>;

    fn get_certificate_header(
        &self,
        certificate_id: &CertificateId,
    ) -> Result<Option<CertificateHeader>, Error>;

    fn get_certificate_header_by_cursor(
        &self,
        network_id: NetworkId,
        height: Height,
    ) -> Result<Option<CertificateHeader>, Error>;
    fn get_current_settled_height(&self) -> Result<Vec<(NetworkId, Height, CertificateId)>, Error>;
}

pub trait PerEpochReader: Send + Sync {
    /// Get the epoch number
    fn epoch_number(&self) -> EpochNumber;

    /// Get the starting checkpoint of this epoch
    fn get_start_checkpoint(&self) -> &BTreeMap<NetworkId, Height>;

    /// Get the height of a network's end checkpoint
    fn get_end_checkpoint_height_per_network(
        &self,
        network_id: NetworkId,
    ) -> Result<Option<Height>, Error>;
}
