use std::sync::Arc;

use agglayer_types::{Certificate, CertificateHeader, CertificateId, Height, NetworkId, Proof};
use arc_swap::ArcSwap;

use crate::error::Error;

pub trait EpochStoreReader: Send + Sync {
    type PerEpochStore;

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
