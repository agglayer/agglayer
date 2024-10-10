use mockall::mock;

use super::MockPerEpochStore;
use crate::error::Error;
use crate::stores::EpochStoreReader;
use crate::stores::EpochStoreWriter;

mock! {
    pub EpochsStore {}

    impl EpochStoreWriter for EpochsStore {
        type PerEpochStore = MockPerEpochStore;

        fn open(&self, epoch_number: u64) -> Result<MockPerEpochStore, Error>;
    }

    impl EpochStoreReader for EpochsStore {}
}
