use std::sync::Arc;

use agglayer_clock::ClockRef;
use agglayer_storage::stores::{
    EpochStoreWriter, MetadataReader, MetadataWriter, PerEpochReader, PerEpochWriter, StateReader,
};
use anyhow::{bail, Result};

pub(crate) struct EpochSynchronizer {}

impl EpochSynchronizer {
    fn walk_epochs<StateStore, EpochsStore>(
        state_store: Arc<StateStore>,
        epochs_store: Arc<EpochsStore>,
        mut loe: EpochsStore::PerEpochStore,
        mut current_epoch_number: u64,
        mut epoch_stream: tokio::sync::broadcast::Receiver<agglayer_clock::Event>,
    ) -> Result<EpochsStore::PerEpochStore>
    where
        StateStore: StateReader + MetadataReader + MetadataWriter,
        EpochsStore: EpochStoreWriter,
        EpochsStore::PerEpochStore: PerEpochReader + PerEpochWriter,
    {
        let mut lse;
        while loe.get_epoch_number() < current_epoch_number {
            loe.start_packing()?;
            lse = loe;
            loe = epochs_store
                .open_with_start_checkpoint(lse.get_epoch_number() + 1, lse.get_end_checkpoint())?;
            state_store.set_latest_opened_epoch(loe.get_epoch_number())?;

            if let Ok(agglayer_clock::Event::EpochEnded(n)) = epoch_stream.try_recv() {
                current_epoch_number = n;
            }
        }

        Ok(loe)
    }

    pub async fn start<StateStore, EpochsStore>(
        state_store: Arc<StateStore>,
        epochs_store: Arc<EpochsStore>,
        clock_ref: ClockRef,
    ) -> Result<EpochsStore::PerEpochStore>
    where
        StateStore: StateReader + MetadataReader + MetadataWriter,
        EpochsStore: EpochStoreWriter,
        EpochsStore::PerEpochStore: PerEpochReader + PerEpochWriter,
    {
        // Get current epoch
        let current_epoch_number = clock_ref.current_epoch();
        let epoch_stream = clock_ref.subscribe()?;

        // Get the latest settled epoch
        let lse_number = state_store.get_latest_settled_epoch()?;

        // Get the latest opened epoch
        let loe_number = state_store.get_latest_opened_epoch()?;

        match (lse_number, loe_number) {
            // No LSE and no LOE, we return the current_epoch
            (None, None) => Ok(epochs_store.open(current_epoch_number)?),

            // No LSE and LOE, we start from the latest opened epoch
            (None, Some(loe)) => {
                let loe = epochs_store.open(loe)?;
                Self::walk_epochs(
                    state_store,
                    epochs_store,
                    loe,
                    current_epoch_number,
                    epoch_stream,
                )
            }

            (Some(lse_number), maybe_loe)
                if maybe_loe.is_none() || matches!(maybe_loe, Some(loe) if lse_number <= loe) =>
            {
                let lse = epochs_store.open(lse_number)?;
                let loe = epochs_store.open_with_start_checkpoint(
                    lse.get_epoch_number() + 1,
                    lse.get_end_checkpoint(),
                )?;

                Self::walk_epochs(
                    state_store,
                    epochs_store,
                    loe,
                    current_epoch_number,
                    epoch_stream,
                )
            }

            // LSE is greater than LOE, it is a consistency error
            (_, _) => {
                bail!("LSE is greater than LOE, this is a consistency error")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicU64;

    use agglayer_storage::tests::mocks::{MockEpochsStore, MockPerEpochStore, MockStateStore};
    use mockall::predicate::eq;

    use super::*;

    #[tokio::test]
    async fn no_lse_no_previous_start_from_now() {
        let mut state_store = MockStateStore::new();
        state_store
            .expect_get_latest_settled_epoch()
            .once()
            .returning(|| Ok(None));
        state_store
            .expect_get_latest_opened_epoch()
            .once()
            .returning(|| Ok(None));

        let mut epochs_store = MockEpochsStore::new();
        epochs_store.expect_open().with(eq(10)).returning(|epoch| {
            let mut mock = MockPerEpochStore::new();
            mock.expect_get_epoch_number().returning(move || epoch);
            Ok(mock)
        });

        let (sender, _receiver) = tokio::sync::broadcast::channel(1);
        let current_epoch = AtomicU64::new(10);
        let clock_ref = ClockRef::new(
            sender,
            Arc::new(current_epoch),
            Arc::new(Default::default()),
        );

        let result =
            EpochSynchronizer::start(Arc::new(state_store), Arc::new(epochs_store), clock_ref)
                .await;

        assert!(result.is_ok());

        let epoch = result.unwrap();

        assert_eq!(epoch.get_epoch_number(), 10);
    }

    #[tokio::test]
    async fn no_lse_with_previous_start_from_now() {
        let mut state_store = MockStateStore::new();
        state_store
            .expect_get_latest_settled_epoch()
            .once()
            .returning(|| Ok(None));
        state_store
            .expect_get_latest_opened_epoch()
            .once()
            .returning(|| Ok(Some(6)));

        let mut epochs_store = MockEpochsStore::new();
        epochs_store.expect_open().with(eq(6)).returning(|epoch| {
            let mut mock = MockPerEpochStore::new();
            mock.expect_get_epoch_number().returning(move || epoch);
            Ok(mock)
        });

        let (sender, _receiver) = tokio::sync::broadcast::channel(1);
        let current_epoch = AtomicU64::new(6);
        let clock_ref = ClockRef::new(
            sender,
            Arc::new(current_epoch),
            Arc::new(Default::default()),
        );

        let result =
            EpochSynchronizer::start(Arc::new(state_store), Arc::new(epochs_store), clock_ref)
                .await;

        assert!(result.is_ok());

        let epoch = result.unwrap();

        assert_eq!(epoch.get_epoch_number(), 6);
    }
}
