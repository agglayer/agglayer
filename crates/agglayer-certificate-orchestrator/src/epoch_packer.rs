use std::sync::Arc;

#[cfg(any(test, feature = "testutils"))]
use agglayer_storage::tests::mocks::MockPerEpochStore;
use agglayer_storage::{
    columns::latest_settled_certificate_per_network::SettledCertificate,
    stores::{PerEpochReader, PerEpochWriter},
};
use agglayer_types::{CertificateId, Height, NetworkId};
use ethers::{
    providers::{JsonRpcClient, PendingTransaction},
    types::H256,
};

use crate::Error;

/// Epoch Packer used to gather all the proofs generated on-the-go
/// and to submit them in a settlement tx to the L1.
#[cfg_attr(
    any(test, feature = "testutils"),
    mockall::automock(
        type PerEpochStore=MockPerEpochStore;
        type Provider=ethers::providers::MockProvider;
    )
)]
#[async_trait::async_trait]
pub trait EpochPacker: Unpin + Send + Sync + 'static {
    type PerEpochStore: PerEpochWriter + PerEpochReader;
    type Provider: JsonRpcClient;

    /// Pack an epoch for settlement on the L1
    async fn pack(&self, closing_epoch: Arc<Self::PerEpochStore>) -> Result<(), Error>;

    async fn transaction_exists(&self, tx_hash: H256) -> Result<bool, Error>;

    #[cfg_attr(any(test, feature = "testutils"), mockall::concretize)]
    async fn settle_certificate(
        &self,
        certificate_id: CertificateId,
    ) -> Result<(NetworkId, SettledCertificate), Error>;

    async fn recover_settlement(
        &self,
        tx_hash: H256,
        certificate_id: CertificateId,
        network_id: NetworkId,
        height: Height,
    ) -> Result<(NetworkId, SettledCertificate), Error>;

    /// Watch for the transaction to be mined and update the certificate
    /// accordingly
    #[cfg_attr(any(test, feature = "testutils"), mockall::concretize)]
    async fn watch_and_update(
        &self,
        pending_tx: PendingTransaction<'_, Self::Provider>,
        certificate_id: CertificateId,
        network_id: NetworkId,
        height: Height,
    ) -> Result<(NetworkId, SettledCertificate), Error>;
}
