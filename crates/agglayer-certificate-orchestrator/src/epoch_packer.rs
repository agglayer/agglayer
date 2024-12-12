use std::sync::Arc;

use agglayer_storage::{
    columns::latest_settled_certificate_per_network::SettledCertificate,
    stores::{PerEpochReader, PerEpochWriter},
};
use agglayer_types::{CertificateId, NetworkId};
use arc_swap::ArcSwap;
use futures_util::future::BoxFuture;

use crate::Error;

/// Epoch Packer used to gather all the proofs generated on-the-go
/// and to submit them in a settlement tx to the L1.
pub trait EpochPacker: Unpin + Send + Sync + 'static {
    type PerEpochStore: PerEpochWriter + PerEpochReader;
    /// Pack an epoch for settlement on the L1
    fn pack(
        &self,
        closing_epoch: Arc<Self::PerEpochStore>,
    ) -> Result<BoxFuture<Result<(), Error>>, Error>;

    fn settle_certificate(
        &self,
        related_epoch: Arc<ArcSwap<Self::PerEpochStore>>,
        certificate_id: CertificateId,
    ) -> Result<SettlementFuture, Error>;
}

pub type SettlementFuture<'a> = BoxFuture<'a, Result<(NetworkId, SettledCertificate), Error>>;
