use agglayer_types::{
    Certificate, CertificateHeader, CertificateId, CertificateIndex, CertificateStatus,
    EpochNumber, Hash, Height, NetworkId,
};
use mockall::mock;

use crate::{
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
        fn assign_certificate_to_epoch(
            &self,
            certificate_id: &CertificateId,
            epoch_number: &EpochNumber,
            certificate_index: &CertificateIndex,
        ) -> Result<(), Error>;
        fn add_tx_hash_to_certificate_header(
            &self,
            certificate_id: &CertificateId,
            tx_hash: Hash,
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
            certificate_id: &CertificateId,
            epoch_number: &EpochNumber,
            height: &Height,
        ) -> Result<(), Error>;
    }

    impl StateReader for StateStore {
        fn get_active_networks(&self) -> Result<Vec<NetworkId>, Error>;

        fn get_certificate_headers(
            &self,
            certificate_ids: &[CertificateId]
        ) -> Result<Vec<Option<CertificateHeader>>, Error>;

        fn get_certificate_header(
            &self,
            certificate_id: &CertificateId,
        ) -> Result<Option<CertificateHeader>, Error>;

        fn get_certificate_header_by_cursor(
            &self,
            network_id: NetworkId,
            height: Height,
        ) -> Result<Option<CertificateHeader>, Error>;
        fn get_current_settled_height(&self) -> Result<Vec<(NetworkId, Height, CertificateId, EpochNumber)>, Error>;
    }
}
