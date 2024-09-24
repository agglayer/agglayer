use agglayer_types::{Height, NetworkId};
use futures_util::future::BoxFuture;
use mockall::mock;

use crate::{Certifier, CertifierOutput, EpochPacker, Error};

mock! {
    pub Certifier {}
    impl Certifier for Certifier {
        fn certify(
            &self,
            state: agglayer_types::LocalNetworkStateData,
            network_id: NetworkId,
            height: Height,
        ) -> Result<BoxFuture<'static, Result<CertifierOutput, Error>>, Error>;
    }
}

mock! {
    pub EpochPacker {}
    impl EpochPacker for EpochPacker {
        fn pack(&self, epoch: u64) -> Result<BoxFuture<'static, Result<(), Error>>, Error>;
    }
}
