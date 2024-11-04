use std::collections::BTreeMap;

use agglayer_types::{
    Certificate, CertificateId, CertificateIndex, CertificateStatus, EpochNumber, Height,
    NetworkId, Proof,
};

use crate::{error::Error, stores::PerEpochReader};

pub trait PerEpochWriter: Send + Sync {
    fn add_certificate(
        &self,
        network_id: NetworkId,
        height: Height,
    ) -> Result<(EpochNumber, CertificateIndex), Error>;
    fn start_packing(&self) -> Result<(), Error>;
}

pub trait EpochStoreWriter: Send + Sync {
    type PerEpochStore: PerEpochWriter + PerEpochReader;

    fn open(&self, epoch_number: u64) -> Result<Self::PerEpochStore, Error>;
    fn open_with_start_checkpoint(
        &self,
        epoch_number: u64,
        start_checkpoint: BTreeMap<NetworkId, Height>,
    ) -> Result<Self::PerEpochStore, Error>;
}

pub trait MetadataWriter: Send + Sync {
    /// Set the latest settled epoch.
    fn set_latest_settled_epoch(&self, value: u64) -> Result<(), Error>;
}

pub trait StateWriter: Send + Sync {
    fn insert_certificate_header(
        &self,
        certificate: &Certificate,
        status: CertificateStatus,
    ) -> Result<(), Error>;

    fn update_certificate_header_status(
        &self,
        certificate_id: &CertificateId,
        status: &CertificateStatus,
    ) -> Result<(), Error>;

    fn set_latest_settled_certificate_for_network(
        &self,
        network_id: &NetworkId,
        certificate_id: &CertificateId,
        epoch_number: &EpochNumber,
        height: &Height,
    ) -> Result<(), Error>;
}

pub trait PendingCertificateWriter: Send + Sync {
    fn remove_pending_certificate(
        &self,
        network_id: NetworkId,
        height: Height,
    ) -> Result<(), Error>;

    fn remove_generated_proof(&self, certificate_id: &CertificateId) -> Result<(), Error>;

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

    fn set_latest_proven_certificate_per_network(
        &self,
        network_id: &NetworkId,
        height: &Height,
        certificate_id: &CertificateId,
    ) -> Result<(), Error>;
}
