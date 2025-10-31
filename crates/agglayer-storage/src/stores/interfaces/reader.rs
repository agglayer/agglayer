use std::collections::BTreeMap;

use agglayer_types::{
    primitives::alloy_primitives::BlockNumber, Certificate, CertificateHeader, CertificateId,
    CertificateIndex, EpochNumber, Height, LocalNetworkStateData, NetworkId, Proof,
};

use crate::{
    columns::{
        latest_proven_certificate_per_network::ProvenCertificate,
        latest_settled_certificate_per_network::SettledCertificate,
    },
    error::Error,
};

pub mod network_info_reader;

pub trait DebugReader: Send + Sync {
    fn get_certificate(&self, certificate_id: &CertificateId)
        -> Result<Option<Certificate>, Error>;
}

pub trait EpochStoreReader: Send + Sync {
    /// Get a certificate from a specific epoch by its index
    fn get_certificate(
        &self,
        epoch_number: EpochNumber,
        index: CertificateIndex,
    ) -> Result<Option<Certificate>, Error>;

    /// Get a proof from a specific epoch by its index
    fn get_proof(
        &self,
        epoch_number: EpochNumber,
        index: CertificateIndex,
    ) -> Result<Option<Proof>, Error>;
}

pub trait PendingCertificateReader: Send + Sync {
    fn get_latest_pending_certificate_for_network(
        &self,
        network_id: &NetworkId,
    ) -> Result<Option<(CertificateId, Height)>, Error>;

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
    fn get_current_proven_height_for_network(
        &self,
        network_id: &NetworkId,
    ) -> Result<Option<Height>, Error>;

    fn get_latest_proven_certificate_per_network(
        &self,
        network_id: &NetworkId,
    ) -> Result<Option<(NetworkId, Height, CertificateId)>, Error>;
}

pub trait MetadataReader: Send + Sync {
    /// Get the latest settled epoch.
    fn get_latest_settled_epoch(&self) -> Result<Option<EpochNumber>, Error>;
    /// Get the latest certificate settling block.
    fn get_latest_certificate_settling_block(&self) -> Result<Option<BlockNumber>, Error>;
}

pub trait StateReader: Send + Sync {
    /// Check if a network is disabled.
    fn is_network_disabled(&self, network_id: &NetworkId) -> Result<bool, Error>;

    /// Get the disabled networks.
    fn get_disabled_networks(&self) -> Result<Vec<NetworkId>, Error>;

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

    fn get_current_settled_height(&self) -> Result<Vec<(NetworkId, SettledCertificate)>, Error>;
    fn get_latest_settled_certificate_per_network(
        &self,
        network_id: &NetworkId,
    ) -> Result<Option<(NetworkId, SettledCertificate)>, Error>;

    /// Get the local network state.
    fn read_local_network_state(
        &self,
        network_id: NetworkId,
    ) -> Result<Option<LocalNetworkStateData>, Error>;
}

pub trait PerEpochReader: Send + Sync {
    /// Get the starting checkpoint of this epoch
    fn get_start_checkpoint(&self) -> &BTreeMap<NetworkId, Height>;

    /// Get the ending checkpoint of this epoch
    fn get_end_checkpoint(&self) -> BTreeMap<NetworkId, Height>;

    /// Get epoch number
    fn get_epoch_number(&self) -> EpochNumber;

    fn get_certificate_at_index(
        &self,
        index: CertificateIndex,
    ) -> Result<Option<Certificate>, Error>;
    fn get_proof_at_index(&self, index: CertificateIndex) -> Result<Option<Proof>, Error>;
    /// Get the height of a network's end checkpoint
    fn get_end_checkpoint_height_per_network(
        &self,
        network_id: NetworkId,
    ) -> Result<Option<Height>, Error>;

    fn is_epoch_packed(&self) -> bool;
}
