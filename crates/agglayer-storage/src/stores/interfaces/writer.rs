use agglayer_types::{Certificate, CertificateId, Height, NetworkId, Proof};

use crate::error::Error;

pub trait PerEpochWriter: Send + Sync {
    fn add_certificate(&self, network_id: NetworkId, height: Height) -> Result<(), Error>;
}

pub trait EpochStoreWriter: Send + Sync {
    type PerEpochStore;

    fn open(&self, epoch_number: u64) -> Result<Self::PerEpochStore, Error>;
}

pub trait MetadataWriter: Send + Sync {
    /// Set the latest settled epoch.
    fn set_latest_settled_epoch(&self, value: u64) -> Result<(), Error>;
}

pub trait StateWriter: Send + Sync {
    fn insert_certificate_header(&self, certificate: &Certificate) -> Result<(), Error>;
}

pub trait PendingCertificateWriter: Send + Sync {
    fn remove_pending_certificate(
        &self,
        network_id: NetworkId,
        height: Height,
    ) -> Result<(), Error>;
    fn insert_pending_certificate(
        &self,
        network_id: NetworkId,
        height: Height,
        certificate: &Certificate,
    ) -> Result<(), Error>;

    fn insert_generated_proof(
        &self,
        certificate_id: &CertificateId,
        proof: &Proof,
    ) -> Result<(), Error>;
}
