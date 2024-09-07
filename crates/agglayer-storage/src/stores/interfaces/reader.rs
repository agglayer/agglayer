use crate::{error::Error, types::NetworkId};

pub trait PendingCertificateReader: Send + Sync {}

pub trait MetadataReader: Send + Sync {
    /// Get the latest settled epoch.
    fn get_latest_settled_epoch(&self) -> Result<Option<u64>, Error>;
}

pub trait StateReader: Send + Sync {
    /// Get the active networks.
    fn get_active_networks(&self) -> Result<Vec<NetworkId>, Error>;
}
