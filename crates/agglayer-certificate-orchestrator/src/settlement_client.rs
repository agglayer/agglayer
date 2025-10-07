use agglayer_types::{CertificateId, CertificateIndex, EpochNumber, NetworkId, SettlementTxHash};

use crate::Error;

#[cfg(any(test, feature = "testutils"))]
#[allow(unused)]
pub type MockProvider = alloy::providers::RootProvider<alloy::network::Ethereum>;

/// Settlement client used to gather all the proofs generated on-the-go
/// and to submit them in a settlement tx to the L1.
#[cfg_attr(
    any(test, feature = "testutils"),
    mockall::automock(
        type PerEpochStore=MockPerEpochStore;
        type Provider=alloy::providers::RootProvider<alloy::network::Ethereum>;
    )
)]
#[async_trait::async_trait]
pub trait SettlementClient: Unpin + Send + Sync + 'static {
    type Provider: alloy::providers::Provider<alloy::network::Ethereum>;

    async fn submit_certificate_settlement(
        &self,
        certificate_id: CertificateId,
    ) -> Result<SettlementTxHash, Error>;

    /// Watch for the transaction to be mined and update the certificate
    /// accordingly.
    async fn wait_for_settlement(
        &self,
        settlement_tx_hash: SettlementTxHash,
        certificate_id: CertificateId,
    ) -> Result<(EpochNumber, CertificateIndex), Error>;

    /// Returns a reference to the provider for direct L1 queries
    fn get_provider(&self) -> &Self::Provider;

    /// Returns the latest PP settlement root from the settlement logs
    /// if any.
    /// It queries the `VerifyPessimisticStateTransition` events from the l1.
    async fn get_last_settled_pp_root(
        &self,
        network_id: NetworkId,
    ) -> Result<(Option<[u8; 32]>, Option<SettlementTxHash>), Error>;
}
