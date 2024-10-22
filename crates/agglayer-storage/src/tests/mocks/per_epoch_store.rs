use std::collections::BTreeMap;

use agglayer_types::{CertificateIndex, EpochNumber, Height, NetworkId};
use mockall::mock;

use crate::{
    error::Error,
    stores::{PerEpochReader, PerEpochWriter},
};

mock! {
    pub PerEpochStore {}

    impl PerEpochReader for PerEpochStore {
        fn get_epoch_number(&self) -> u64;

        fn get_start_checkpoint(&self) -> &BTreeMap<NetworkId, Height>;
        fn get_end_checkpoint(&self) -> BTreeMap<NetworkId, Height>;

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
