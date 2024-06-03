use crate::{error::Error, types::NetworkId};

pub mod metadata;
pub mod pending;
pub mod per_epoch;
pub mod state;

pub trait PendingCertificateWriter: Send + Sync {}

pub trait PendingCertificateReader: Send + Sync {}

pub trait MetadataReader: Send + Sync {
    /// Get the latest settled epoch.
    fn get_latest_settled_epoch(&self) -> Result<Option<u64>, Error>;
}

pub trait MetadataWriter: Send + Sync {
    /// Set the latest settled epoch.
    fn set_latest_settled_epoch(&self, value: u64) -> Result<(), Error>;
}

pub trait StateReader: Send + Sync {
    /// Get the active networks.
    fn get_active_networks(&self) -> Result<Vec<NetworkId>, Error>;
}

pub trait PerEpochWriter: Send + Sync {}
