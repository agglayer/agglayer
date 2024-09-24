use agglayer_types::{Certificate, CertificateId, Height, NetworkId, Proof};
use mockall::mock;

use crate::{
    columns::latest_proven_certificate_per_network::ProvenCertificate,
    error::Error,
    stores::{PendingCertificateReader, PendingCertificateWriter},
};

mock! {
    pub PendingStore {}
    impl PendingCertificateReader for PendingStore {
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
    }

    impl PendingCertificateWriter for PendingStore {
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
}
