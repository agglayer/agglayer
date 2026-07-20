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

        fn rollup_manager_address(&self) -> agglayer_types::Address;

        async fn verifier_type(
            &self,
            rollup_id: u32,
        ) -> Result<agglayer_contracts::rollup::VerifierType, CertificationError>;

        fn default_l1_info_tree_leaf_count(&self) -> u32;
    }
}
