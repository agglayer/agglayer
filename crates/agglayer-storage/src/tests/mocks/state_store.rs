use agglayer_types::{
    Certificate, CertificateHeader, CertificateId, CertificateStatus, Digest, EpochNumber, Height,
    LocalNetworkStateData, NetworkId,
};
use mockall::mock;

use crate::{
    columns::latest_settled_certificate_per_network::SettledCertificate,
    error::Error,
    stores::{MetadataReader, MetadataWriter, StateReader, StateWriter},
};
mock! {
    pub StateStore {}
    impl MetadataReader for StateStore {
        fn get_latest_settled_epoch(&self) -> Result<Option<u64>, Error>;
    }

    impl MetadataWriter for StateStore {
        fn set_latest_settled_epoch(&self, value: u64) -> Result<(), Error>;
    }

    impl StateWriter for StateStore {
        fn update_settlement_tx_hash(
            &self,
            certificate_id: &CertificateId,
            tx_hash: Digest,
        ) -> Result<(), Error>;
        fn assign_certificate_to_epoch(
            &self,
            certificate_id: &CertificateId,
            epoch_number: &EpochNumber,
            certificate_index: &agglayer_types::CertificateIndex,
        ) -> Result<(), Error>;

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
            height: &Height,
            certificate_id: &CertificateId,
            epoch_number: &EpochNumber,
            certificate_index: &agglayer_types::CertificateIndex
        ) -> Result<(), Error>;

        fn write_local_network_state(
            &self,
            network_id: &NetworkId,
            new_state: &LocalNetworkStateData,
            new_leaves: &[Digest],
        ) -> Result<(), Error>;
    }

    impl StateReader for StateStore {
        fn get_active_networks(&self) -> Result<Vec<NetworkId>, Error>;

        fn get_latest_settled_certificate_per_network(
            &self,
            network_id: &NetworkId,
        ) -> Result<Option<(NetworkId, SettledCertificate)>, Error>;

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

        fn read_local_network_state(
            &self,
            network_id: NetworkId,
        ) -> Result<Option<LocalNetworkStateData>, Error>;
    }
}
