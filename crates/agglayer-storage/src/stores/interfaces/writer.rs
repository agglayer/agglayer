use std::collections::BTreeMap;

use agglayer_types::{
    primitives::Digest, Certificate, CertificateId, CertificateIndex, CertificateStatus,
    EpochNumber, ExecutionMode, Height, LocalNetworkStateData, NetworkId, Proof, SettledClaim,
    SettlementTxHash,
};

use crate::{
    error::Error,
    stores::{PerEpochReader, StateReader},
};

pub mod settlement_writer;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateEvenIfAlreadyPresent {
    Yes,
    No,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateStatusToCandidate {
    Yes,
    No,
}

pub trait StateWriter: Send + Sync {
    /// Persist a normal submission through the pending and state stores.
    /// StateStore serializes this sequence through both database writes.
    fn submit_pending_certificate<P: PendingCertificateWriter>(
        &self,
        pending: &P,
        certificate: &Certificate,
    ) -> eyre::Result<()>
    where
        Self: StateReader + Sized,
    {
        crate::stores::certificate_updater::submit(
            pending,
            self,
            certificate,
            &CertificateStatus::Pending,
        )
    }

    /// Publish a submission's header and pending pointer in one state batch.
    /// The legacy pending pointer and body must already be persisted. Requests
    /// a best-effort backup after a successful batch when the status requires
    /// it.
    fn publish_pending_certificate(
        &self,
        certificate: &Certificate,
        status: &CertificateStatus,
    ) -> Result<(), Error>;

    /// Publish only the cached pending pointer, preserving the header status.
    fn set_latest_pending_certificate_pointer(
        &self,
        network_id: NetworkId,
        height: Height,
        certificate_id: CertificateId,
    ) -> Result<(), Error>;

    /// Publish the proven cache pointer without changing header status.
    /// Production callers use `certificate_updater::set_proven` to publish
    /// both pointers.
    fn set_latest_proven_certificate_pointer(
        &self,
        network_id: NetworkId,
        certificate_id: CertificateId,
    ) -> Result<(), Error>;

    fn disable_network(
        &self,
        network_id: &NetworkId,
        disabled_by: agglayer_types::network_info::DisabledBy,
    ) -> Result<(), Error>;

    fn enable_network(&self, network_id: &NetworkId) -> Result<(), Error>;

    fn update_settlement_tx_hash(
        &self,
        certificate_id: &CertificateId,
        tx_hash: SettlementTxHash,
        force: UpdateEvenIfAlreadyPresent,
        set_status: UpdateStatusToCandidate,
    ) -> Result<(), Error>;

    fn remove_settlement_tx_hash(&self, certificate_id: &CertificateId) -> Result<(), Error>;

    /// Updates the stored status of `certificate_id`.
    ///
    /// Moving a certificate to [`CertificateStatus::Proven`] also requests a
    /// backup: settlement is submitted from a spawned task shortly after, so
    /// this is the last status write still ordered ahead of the certificate
    /// reaching L1.
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

    /// Record the network's settled cursor together with everything
    /// `getNetworkInfo` serves about the settlement: the settled pointer, the
    /// local exit tree leaf count, and the claim the certificate carried.
    ///
    /// All of it lands in one batch, so a request reads one settlement rather
    /// than fields from two, and nothing has to be re-derived on the request
    /// path.
    fn set_latest_settled_certificate_for_network(
        &self,
        network_id: &NetworkId,
        height: &Height,
        certificate_id: &CertificateId,
        epoch_number: &EpochNumber,
        certificate_index: &CertificateIndex,
        settled_claim: Option<SettledClaim>,
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

    /// Persist the legacy pending pointer and certificate body. Production
    /// submission calls this through `certificate_updater::submit`.
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
}
