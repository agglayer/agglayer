use agglayer_types::{Hash, LocalNetworkStateData, NetworkId};
use mockall::mock;

use crate::{
    error::Error,
    stores::{LocalNetworkStateReader, LocalNetworkStateWriter},
};

mock! {
    pub LocalNetworkStateStore {}
    impl LocalNetworkStateReader for LocalNetworkStateStore {
        fn read_local_network_state(
            &self,
            network_id: NetworkId,
        ) -> Result<Option<LocalNetworkStateData>, Error>;
    }

    impl LocalNetworkStateWriter for LocalNetworkStateStore {
        fn write_local_network_state(
            &self,
            network_id: &NetworkId,
            new_state: &LocalNetworkStateData,
            new_leaves: &[Hash],
        ) -> Result<(), Error>;
    }
}
