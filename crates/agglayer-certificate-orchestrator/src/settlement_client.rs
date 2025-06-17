use agglayer_types::{CertificateId, CertificateIndex, EpochNumber, SettlementTxHash};
use alloy::transports::Transport;

use crate::Error;

#[cfg(any(test, feature = "testutils"))]
pub type MockTransport = alloy::transports::http::Http<reqwest::Client>;

/// Epoch Packer used to gather all the proofs generated on-the-go
/// and to submit them in a settlement tx to the L1.
#[cfg_attr(
    any(test, feature = "testutils"),
    mockall::automock(
        type PerEpochStore=MockPerEpochStore;
        type Provider=MockTransport;
    )
)]
#[async_trait::async_trait]
pub trait SettlementClient: Unpin + Send + Sync + 'static {
    type Provider: Transport + Clone;

    async fn submit_certificate_settlement(
        &self,
        certificate_id: CertificateId,
    ) -> Result<SettlementTxHash, Error>;

    /// Watch for the transaction to be mined and update the certificate
    /// accordingly
    async fn wait_for_settlement(
        &self,
        settlement_tx_hash: SettlementTxHash,
        certificate_id: CertificateId,
    ) -> Result<(EpochNumber, CertificateIndex), Error>;
}
