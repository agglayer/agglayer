use agglayer_contracts::rollup::VerifierType;
use agglayer_types::{Address, Certificate, Digest, Height, LocalNetworkStateData, NetworkId};
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
    pub new_pp_root: Digest,
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
        certificate_tx_hash: Option<Digest>,
    ) -> Result<(MultiBatchHeader, LocalNetworkState, PessimisticProofOutput), CertificationError>;

    /// The L1 rollup-manager address settlement transactions are sent to.
    fn rollup_manager_address(&self) -> Address;

    /// The on-chain verifier type for `rollup_id`, which selects the proof
    /// encoding.
    async fn verifier_type(&self, rollup_id: u32) -> Result<VerifierType, CertificationError>;

    /// Default l1-info-tree leaf count, used when the certificate carries none.
    fn default_l1_info_tree_leaf_count(&self) -> u32;
}
