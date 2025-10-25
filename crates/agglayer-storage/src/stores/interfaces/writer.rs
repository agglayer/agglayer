use std::collections::BTreeMap;

use agglayer_types::{
    primitives::Digest, Certificate, CertificateId, CertificateIndex, CertificateStatus,
    EpochNumber, ExecutionMode, Height, LocalNetworkStateData, NetworkId, Proof, SettlementTxHash,
};

use crate::{error::Error, stores::PerEpochReader, types::SettlementTxHashRecord};

pub trait DebugWriter: Send + Sync {
    fn add_certificate(&self, certificate: &Certificate) -> Result<(), Error>;
}

pub trait PerEpochWriter: Send + Sync {
    fn add_certificate(
        &self,
        certificate_id: CertificateId,
        mode: ExecutionMode,
    ) -> Result<(EpochNumber, CertificateIndex), Error>;
    fn start_packing(&self) -> Result<(), Error>;
}

pub trait EpochStoreWriter: Send + Sync {
    type PerEpochStore: PerEpochWriter + PerEpochReader;

    fn open(&self, epoch_number: EpochNumber) -> Result<Self::PerEpochStore, Error>;
    fn open_with_start_checkpoint(
        &self,
        epoch_number: EpochNumber,
        start_checkpoint: BTreeMap<NetworkId, Height>,
    ) -> Result<Self::PerEpochStore, Error>;
}

pub trait MetadataWriter: Send + Sync {
    /// Set the latest settled epoch.
    fn set_latest_settled_epoch(&self, value: EpochNumber) -> Result<(), Error>;
}

pub trait StateWriter: Send + Sync {
    fn update_settlement_tx_hash(
        &self,
        certificate_id: &CertificateId,
        tx_hash: SettlementTxHash,
        force: bool,
    ) -> Result<(), Error>;

    fn remove_settlement_tx_hash(&self, certificate_id: &CertificateId) -> Result<(), Error>;

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

    fn assign_certificate_to_epoch(
        &self,
        certificate_id: &CertificateId,
        epoch_number: &EpochNumber,
        certificate_index: &CertificateIndex,
    ) -> Result<(), Error>;

    fn set_latest_settled_certificate_for_network(
        &self,
        network_id: &NetworkId,
        height: &Height,
        certificate_id: &CertificateId,
        epoch_number: &EpochNumber,
        certificate_index: &CertificateIndex,
    ) -> Result<(), Error>;

    fn write_local_network_state(
        &self,
        network_id: &NetworkId,
        new_state: &LocalNetworkStateData,
        new_leaves: &[Digest],
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

    fn set_latest_pending_certificate_per_network(
        &self,
        network_id: &NetworkId,
        height: &Height,
        certificate_id: &CertificateId,
    ) -> Result<(), Error>;

    fn insert_settlement_tx_hash_for_certificate(
        &self,
        certificate_id: &CertificateId,
        tx_hash: SettlementTxHash,
    ) -> Result<(), Error>;

    fn update_settlement_tx_hashes_for_certificate<'a, F>(
        &'a self,
        certificate_id: &CertificateId,
        f: F,
    ) -> Result<(), Error>
    where
        F: FnOnce(SettlementTxHashRecord) -> Result<SettlementTxHashRecord, String> + 'a;
}
