#![cfg_attr(feature = "coverage", feature(coverage_attribute))]

use std::sync::Arc;

use agglayer_certificate_orchestrator::{EpochPacker, Error};
use agglayer_config::certificate_orchestrator::prover::ProverConfig;
use agglayer_storage::stores::{
    EpochStoreReader, EpochStoreWriter, PerEpochReader, PerEpochWriter, StateReader,
};
use agglayer_types::{CertificateId, CertificateIndex, EpochNumber};
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
pub struct AggregatorNotifier<I, StateStore, EpochsStore> {
    _phantom: std::marker::PhantomData<fn() -> I>,
    #[allow(unused)]
    state_store: Arc<StateStore>,
    #[allow(unused)]
    epochs_store: Arc<EpochsStore>,
}

impl<I, StateStore, EpochsStore> AggregatorNotifier<I, StateStore, EpochsStore>
where
    I: Serialize,
{
    /// Try to create a new notifier using the given configuration
    pub fn try_new(
        _config: &ProverConfig,
        state_store: Arc<StateStore>,
        epochs_store: Arc<EpochsStore>,
    ) -> Result<Self, Error> {
        Ok(Self {
            _phantom: std::marker::PhantomData,
            state_store,
            epochs_store,
        })
    }
}

impl<I, StateStore, EpochsStore> EpochPacker for AggregatorNotifier<I, StateStore, EpochsStore>
where
    StateStore: StateReader + 'static,
    EpochsStore: EpochStoreWriter + EpochStoreReader + 'static,
    I: Clone + 'static,
{
    type PerEpochStore = <EpochsStore as EpochStoreWriter>::PerEpochStore;

    fn settle_certificate(
        &self,
        epoch_number: EpochNumber,
        certificate_index: CertificateIndex,
        certificate_id: CertificateId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn pack(
        &self,
        closing_epoch: Arc<Self::PerEpochStore>,
    ) -> Result<BoxFuture<Result<(), Error>>, Error> {
        let epoch_number = closing_epoch.get_epoch_number();
        debug!("Start the settlement of the epoch {}", epoch_number);

        Ok(Box::pin(async move {
            // No aggregation for now, we settle each PP individually
            let _result: Result<(), Error> = tokio::task::spawn_blocking(move || {
                closing_epoch.start_packing()?;

                Ok(())
            })
            .await
            // TODO: Handle error in a better way
            .map_err(|_| Error::InternalError)?;

            Ok(())
        }))
    }
}
