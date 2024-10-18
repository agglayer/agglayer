use std::sync::Arc;

use arc_swap::ArcSwap;
use mockall::mock;

use super::MockPerEpochStore;
use crate::stores::EpochStoreReader;

mock! {
    pub EpochsStore {}

    impl EpochStoreReader for EpochsStore {
        type PerEpochStore = MockPerEpochStore;

        fn get_current_epoch(&self) -> Arc<ArcSwap<MockPerEpochStore>>;
    }
}
