#![cfg_attr(feature = "coverage", feature(coverage_attribute))]

use std::sync::Arc;

use agglayer_certificate_orchestrator::{EpochPacker, Error};
use agglayer_config::certificate_orchestrator::prover::ProverConfig;
use error::NotifierError;
use futures::future::BoxFuture;
use proof::Proof;
use serde::Serialize;
use sp1::SP1;
use sp1_sdk::NetworkProver;
use sp1_sdk::{LocalProver, MockProver};
use tracing::{debug, error, info};

/// ELF of the pessimistic proof program
const ELF: &[u8] =
    include_bytes!("../../pessimistic-proof-program/elf/riscv32im-succinct-zkvm-elf");

mod error;
mod proof;
mod sp1;
#[cfg(test)]
mod tests;

/// The trait that must be implemented by the prover
pub(crate) trait AggregatorProver<I>: Send + Sync {
    fn prove(&self, to_pack: Vec<I>) -> BoxFuture<'_, Result<Proof, anyhow::Error>>;
    fn verify(&self, proof: &Proof) -> Result<(), anyhow::Error>;
}

/// The notifier that will be used to notify the aggregator
/// using a prover that implements the [`AggregatorProver`] trait
#[derive(Clone)]
pub struct AggregatorNotifier<I> {
    /// The prover that will be used to generate the proof
    prover: Arc<dyn AggregatorProver<I>>,
}

impl<I> TryFrom<ProverConfig> for AggregatorNotifier<I>
where
    I: Serialize,
{
    type Error = NotifierError;

    #[cfg_attr(feature = "coverage", coverage(off))]
    fn try_from(config: ProverConfig) -> Result<Self, Self::Error> {
        match config {
            ProverConfig::SP1Network {} => Ok(Self {
                prover: Arc::new(SP1::new(NetworkProver::new(), ELF)),
            }),
            ProverConfig::SP1Local {} => Ok(Self {
                prover: Arc::new(SP1::new(LocalProver::new(), ELF)),
            }),
            ProverConfig::SP1Mock {} => Ok(Self {
                prover: Arc::new(SP1::new(MockProver::new(), ELF)),
            }),
        }
    }
}

impl<I> EpochPacker for AggregatorNotifier<I>
where
    I: Clone + 'static,
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
