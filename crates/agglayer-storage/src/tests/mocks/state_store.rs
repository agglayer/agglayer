use agglayer_types::{
    Certificate, CertificateHeader, CertificateId, CertificateStatus, Height, NetworkId,
};
use mockall::mock;

use crate::{
    error::Error,
    stores::{StateReader, StateWriter},
};
mock! {
    pub StateStore {}
    impl StateWriter for StateStore {
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
    }

    impl StateReader for StateStore {
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
        fn get_current_settled_height(&self) -> Result<Vec<(NetworkId, Height, CertificateId)>, Error>;
    }
}
