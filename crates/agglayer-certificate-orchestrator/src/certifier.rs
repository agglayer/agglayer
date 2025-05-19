use agglayer_types::{
    primitives::keccak::Keccak256Hasher, Certificate, Height, LocalNetworkStateData, NetworkId,
};
use pessimistic_proof::{
    multi_batch_header::MultiBatchHeader, LocalNetworkState, PessimisticProofOutput,
};

use crate::error::CertificationError;

pub trait CertificateInput: Clone {
    fn network_id(&self) -> NetworkId;
}

impl CertificateInput for Certificate {
    fn network_id(&self) -> NetworkId {
        self.network_id
    }
}

#[derive(Debug, Clone)]
pub struct CertifierOutput {
    pub certificate: Certificate,
    pub height: Height,
    pub new_state: LocalNetworkStateData,
    pub network: NetworkId,
}

pub type CertifierResult = Result<CertifierOutput, CertificationError>;

/// Apply one Certificate on top of a local state and computes one proof.
#[async_trait::async_trait]
pub trait Certifier: Unpin + Send + Sync + 'static {
    async fn certify(
        &self,
        full_state: LocalNetworkStateData,
        network_id: NetworkId,
        height: Height,
    ) -> CertifierResult;

    async fn witness_generation(
        &self,
        certificate: &Certificate,
        state: &mut LocalNetworkStateData,
    ) -> Result<
        (
            MultiBatchHeader<Keccak256Hasher>,
            LocalNetworkState,
            PessimisticProofOutput,
        ),
        CertificationError,
    >;
}
