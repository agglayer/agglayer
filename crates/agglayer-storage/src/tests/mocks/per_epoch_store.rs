use std::collections::BTreeMap;

use agglayer_types::{Certificate, CertificateIndex, EpochNumber, Height, NetworkId, Proof};
use mockall::mock;

use crate::{
    error::Error,
    stores::{PerEpochReader, PerEpochWriter},
};

mock! {
    #[derive(Debug)]
    pub PerEpochStore {}

    impl PerEpochReader for PerEpochStore {
        fn get_epoch_number(&self) -> u64;
        fn get_certificates(&self) -> Result<Vec<(CertificateIndex, Certificate)>, Error>;
        fn get_start_checkpoint(&self) -> &BTreeMap<NetworkId, Height>;
        fn get_end_checkpoint(&self) -> BTreeMap<NetworkId, Height>;
        fn get_proof_at_index(&self, index: CertificateIndex) -> Result<Option<Proof>, Error>;
        fn get_certificate_at_index(&self, index: CertificateIndex) -> Result<Option<Certificate>, Error>;
        fn get_end_checkpoint_height_per_network(
            &self,
            network_id: NetworkId,
        ) -> Result<Option<Height>, Error>;
    }

    impl PerEpochWriter for PerEpochStore {
        fn add_certificate(&self, network_id: NetworkId, height: Height) -> Result<(EpochNumber, CertificateIndex), Error>;
        fn start_packing(&self) -> Result<(), Error>;
    }
}
