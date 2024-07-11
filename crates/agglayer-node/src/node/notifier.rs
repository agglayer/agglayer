use agglayer_certificate_orchestrator::{EpochPacker, Error};
use futures::future::BoxFuture;
use pessimistic_proof::batch_header::BatchHeader;
use tracing::debug;

#[derive(Clone)]
pub(crate) struct AggregatorNotifier {}

impl AggregatorNotifier {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl EpochPacker for AggregatorNotifier {
    type Item = BatchHeader;
    fn pack<T: IntoIterator<Item = Self::Item>>(
        &self,
        epoch: u64,
        to_pack: T,
    ) -> Result<BoxFuture<Result<(), Error>>, Error> {
        // TODO: Implement the aggregator notifier.

        let to_pack = to_pack.into_iter().collect::<Vec<_>>();

        Ok(Box::pin(async move {
            debug!(
                "Start packing epoch {} with {} headers",
                epoch,
                to_pack.len()
            );

            Ok(())
        }))
    }
}
