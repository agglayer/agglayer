#![cfg_attr(feature = "coverage", feature(coverage_attribute))]

use agglayer_certificate_orchestrator::{EpochPacker, Error};
use agglayer_config::certificate_orchestrator::prover::ProverConfig;
use futures::future::BoxFuture;
use serde::Serialize;
use tracing::debug;

/// ELF of the pessimistic proof program
const ELF: &[u8] =
    include_bytes!("../../pessimistic-proof-program/elf/riscv32im-succinct-zkvm-elf");

mod certifier;
mod packer;
mod proof;

pub use certifier::CertifierClient;
pub use packer::EpochPackerClient;

/// The notifier that will be used to notify the aggregator
/// using a prover that implements the [`AggregatorProver`] trait
#[derive(Clone)]
pub struct AggregatorNotifier<I> {
    _phantom: std::marker::PhantomData<fn() -> I>,
}

impl<I> TryFrom<ProverConfig> for AggregatorNotifier<I>
where
    I: Serialize,
{
    type Error = anyhow::Error;

    #[cfg_attr(feature = "coverage", coverage(off))]
    fn try_from(_config: ProverConfig) -> Result<Self, Self::Error> {
        Ok(Self {
            _phantom: std::marker::PhantomData,
        })
    }
}

impl<I> EpochPacker for AggregatorNotifier<I>
where
    I: Send + Sync + Unpin + Clone + 'static,
{
    type Item = I;
    fn pack<T: IntoIterator<Item = Self::Item>>(
        &self,
        epoch: u64,
        to_pack: T,
    ) -> Result<BoxFuture<Result<(), Error>>, Error> {
        let to_pack = to_pack.into_iter().collect::<Vec<_>>();

        debug!(
            "Start the settlement for the epoch {} with {} p-proofs",
            epoch,
            to_pack.len()
        );

        Ok(Box::pin(async move {
            // TODO: Submit the settlement tx for each proof
            // No aggregation for now, we settle each PP individually
            Ok(())
        }))
    }
}
