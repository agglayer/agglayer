use mockall::mock;

use super::MockPerEpochStore;
use crate::{
    error::Error,
    stores::{EpochStoreReader, EpochStoreWriter},
};

mock! {
    pub EpochsStore {}

    impl EpochStoreWriter for EpochsStore {
        type PerEpochStore = MockPerEpochStore;
        fn open(&self, epoch_number: u64) -> Result<MockPerEpochStore, Error>;
        fn open_with_start_checkpoint(
            &self,
            epoch_number: u64,
            start_checkpoint: std::collections::BTreeMap<agglayer_types::NetworkId, agglayer_types::Height>,
        ) -> Result<MockPerEpochStore, Error>;
    }

    impl EpochStoreReader for EpochsStore {}
}
