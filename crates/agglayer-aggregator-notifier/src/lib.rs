#![cfg_attr(feature = "coverage", feature(coverage_attribute))]

/// ELF of the pessimistic proof program
const ELF: &[u8] =
    include_bytes!("../../pessimistic-proof-program/elf/riscv32im-succinct-zkvm-elf");

mod certifier;
mod packer;
mod proof;

pub use certifier::CertifierClient;
pub use packer::EpochPackerClient;

// /// The notifier that will be used to notify the aggregator
// /// using a prover that implements the [`AggregatorProver`] trait
// #[derive(Clone)]
// pub struct AggregatorNotifier<I, StateStore, EpochsStore, RollupManagerRpc> {
//     _phantom: std::marker::PhantomData<fn() -> I>,
//     #[allow(unused)]
//     l1_rpc: Arc<RollupManagerRpc>,
//     #[allow(unused)]
//     state_store: Arc<StateStore>,
//     #[allow(unused)]
//     epochs_store: Arc<EpochsStore>,
// }
//
// impl<I, StateStore, EpochsStore, RollupManagerRpc>
//     AggregatorNotifier<I, StateStore, EpochsStore, RollupManagerRpc>
// where
//     I: Serialize,
// {
//     /// Try to create a new notifier using the given configuration
//     pub fn try_new(
//         config: Arc<OutboundRpcSettleConfig>,
//         state_store: Arc<StateStore>,
//         epochs_store: Arc<EpochsStore>,
//         l1_rpc: Arc<RollupManagerRpc>,
//     ) -> Result<Self, Error> {
//         Ok(Self {
//             config,
//             l1_rpc,
//             _phantom: std::marker::PhantomData,
//             state_store,
//             epochs_store,
//         })
//     }
// }

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
    StateStore: StateReader + MetadataWriter + 'static,
    EpochsStore: EpochStoreReader + 'static,
    I: Clone + 'static,
{
    fn pack(&self, epoch: u64) -> Result<BoxFuture<Result<(), Error>>, Error> {
        debug!("Start the settlement of the epoch {}", epoch);

        let state_store = self.state_store.clone();
        Ok(Box::pin(async move {
            state_store.set_latest_settled_epoch(epoch)?;

            Ok(())
        }))
    }
}
