use agglayer_types::{Height, LocalNetworkStateData, NetworkId};
use mockall::mock;
use pessimistic_proof::{multi_batch_header::MultiBatchHeader, LocalNetworkState};

use crate::{error::CertificationError, Certifier, CertifierOutput};

mock! {
    pub Certifier {}

#[async_trait::async_trait]
    impl Certifier for Certifier {
        async fn certify(
            &self,
            state: agglayer_types::LocalNetworkStateData,
            network_id: NetworkId,
            height: Height,
        ) -> Result<CertifierOutput, CertificationError>;

        async fn witness_generation(
            &self,
            certificate: &agglayer_types::Certificate,
            state: &mut LocalNetworkStateData,
            certificate_tx_hash: Option<agglayer_types::Digest>,
        ) -> Result<(MultiBatchHeader, LocalNetworkState, pessimistic_proof::PessimisticProofOutput), CertificationError>;
    }
}
