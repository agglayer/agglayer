use agglayer_types::{Height, NetworkId};
use futures_util::future::BoxFuture;
use mockall::mock;

use crate::{Certifier, CertifierOutput, EpochPacker, Error, SettlementFuture};

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
        type PerEpochStore = agglayer_storage::tests::mocks::MockPerEpochStore;
        fn pack(&self, closing_epoch: std::sync::Arc<agglayer_storage::tests::mocks::MockPerEpochStore>) -> Result<BoxFuture<'static, Result<(), Error>>, Error>;
        fn settle_certificate(
            &self,
            related_epoch: std::sync::Arc<agglayer_storage::tests::mocks::MockPerEpochStore>,
            certificate_index: agglayer_types::CertificateIndex,
            certificate_id: agglayer_types::CertificateId,
        ) -> Result<SettlementFuture<'static>, Error>;
    }
}
