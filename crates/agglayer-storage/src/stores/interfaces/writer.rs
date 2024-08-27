use crate::error::Error;

pub trait PendingCertificateWriter: Send + Sync {}

pub trait MetadataWriter: Send + Sync {
    /// Set the latest settled epoch.
    fn set_latest_settled_epoch(&self, value: u64) -> Result<(), Error>;
}

pub trait PerEpochWriter: Send + Sync {}
