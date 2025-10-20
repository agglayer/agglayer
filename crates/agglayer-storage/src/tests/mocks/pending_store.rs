use agglayer_types::{Certificate, CertificateId, Height, NetworkId, Proof, SettlementTxHash};
use mockall::mock;

use crate::{
    columns::latest_proven_certificate_per_network::ProvenCertificate,
    error::Error,
    stores::{PendingCertificateReader, PendingCertificateWriter},
    types::SettlementTxHashRecord,
};

mock! {
    pub PendingStore {}
    impl PendingCertificateReader for PendingStore {
        fn get_certificate(
            &self,
            network_id: NetworkId,
            height: Height,
        ) -> Result<Option<Certificate>, Error>;

        fn get_latest_proven_certificate_per_network(
            &self,
            network_id: &NetworkId,
        ) -> Result<Option<(NetworkId, Height, CertificateId)>, Error>;

        fn get_latest_pending_certificate_for_network(
            &self,
            network_id: &NetworkId,
        ) -> Result<Option<(CertificateId, Height)>, Error>;

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

        fn get_settlement_tx_hashes_for_certificate(
            &self,
            certificate_id: CertificateId,
        ) -> Result<Vec<SettlementTxHash>, Error>;
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
}
