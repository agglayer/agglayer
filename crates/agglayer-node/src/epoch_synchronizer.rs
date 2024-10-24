use std::sync::Arc;

use agglayer_clock::ClockRef;
use agglayer_storage::stores::{
    EpochStoreWriter, MetadataReader, MetadataWriter, PerEpochReader, PerEpochWriter, StateReader,
};
use anyhow::{bail, Result};
use tracing::debug;

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

            match state_store.set_latest_opened_epoch(loe.get_epoch_number()) {
                Err(agglayer_storage::error::Error::UnprocessedAction(_)) => {
                    debug!("LOE is behind the current one in storage");
                }
                Err(e) => return Err(e.into()),
                Ok(_) => (),
            }

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
                debug!("No LSE, starting from the latest opened epoch");
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
                debug!("LSE is less than or equal to LOE, starting from LSE + 1");
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
    use std::{collections::BTreeMap, sync::atomic::AtomicU64};

    use agglayer_config::Config;
    use agglayer_storage::{
        columns::epochs::end_checkpoint::EndCheckpointColumn,
        storage::epochs_db_cf_definitions,
        stores::{
            epochs::EpochsStore, pending::PendingStore, state::StateStore, PendingCertificateWriter,
        },
        tests::{
            mocks::{MockEpochsStore, MockPerEpochStore, MockStateStore},
            TempDBDir,
        },
    };
    use agglayer_types::{Certificate, Proof};
    use mockall::predicate::{eq, in_iter};

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

    #[test_log::test(tokio::test)]
    async fn lse_number_is_less_than_loe() {
        let mut state_store = MockStateStore::new();
        state_store
            .expect_get_latest_settled_epoch()
            .once()
            .returning(|| Ok(Some(10)));
        state_store
            .expect_get_latest_opened_epoch()
            .once()
            .returning(|| Ok(Some(15)));

        state_store
            .expect_set_latest_opened_epoch()
            .with(in_iter(vec![11, 12, 13, 14, 15]))
            .returning(|_| {
                Err(agglayer_storage::error::Error::UnprocessedAction(
                    "".to_string(),
                ))
            });

        let mut start_checkpoint = BTreeMap::new();
        start_checkpoint.insert(0.into(), 0);

        let mut epochs_store = MockEpochsStore::new();
        let end_checkpoint = start_checkpoint.clone();
        epochs_store
            .expect_open()
            .once()
            .with(eq(10))
            .return_once(move |epoch| {
                let mut mock = MockPerEpochStore::new();
                mock.expect_get_epoch_number().returning(move || epoch);
                mock.expect_get_end_checkpoint()
                    .once()
                    .return_once(move || end_checkpoint);

                Ok(mock)
            });

        epochs_store
            .expect_open_with_start_checkpoint()
            .with(in_iter(vec![11, 12, 13, 14, 15]), eq(start_checkpoint))
            .returning(|epoch, start_checkpoint| {
                let mut mock = MockPerEpochStore::new();
                mock.expect_get_epoch_number().returning(move || epoch);
                if epoch == 15 {
                    mock.expect_start_packing().never().returning(|| Ok(()));
                    mock.expect_get_end_checkpoint()
                        .never()
                        .return_once(move || start_checkpoint.clone());
                } else {
                    mock.expect_start_packing().once().returning(|| Ok(()));
                    mock.expect_get_end_checkpoint()
                        .once()
                        .return_once(move || start_checkpoint.clone());
                }

                Ok(mock)
            });

        let (sender, _receiver) = tokio::sync::broadcast::channel(1);
        let current_epoch = AtomicU64::new(15);
        let clock_ref = ClockRef::new(
            sender,
            Arc::new(current_epoch),
            Arc::new(Default::default()),
        );

        let result =
            EpochSynchronizer::start(Arc::new(state_store), Arc::new(epochs_store), clock_ref)
                .await;

        println!("{:?}", result);
        assert!(result.is_ok());

        let epoch = result.unwrap();

        assert_eq!(epoch.get_epoch_number(), 15);
    }

    #[test_log::test(tokio::test)]
    async fn lse_number_is_less_than_loe_real_db() {
        let tmp = TempDBDir::new();
        let config = Arc::new(Config::new(&tmp.path));
        let state_store =
            Arc::new(StateStore::new_with_path(&config.storage.state_db_path).unwrap());
        let pending_store =
            Arc::new(PendingStore::new_with_path(&config.storage.pending_db_path).unwrap());

        let epochs_store = Arc::new(
            EpochsStore::new(
                config.clone(),
                15,
                pending_store.clone(),
                state_store.clone(),
            )
            .unwrap(),
        );

        let start_checkpoint = BTreeMap::new();

        let network_1 = 1.into();
        let network_2 = 2.into();

        let epoch_10 = epochs_store
            .open_with_start_checkpoint(10, start_checkpoint.clone())
            .unwrap();
        let certificate_1 = Certificate::new_for_test(network_1, 0);
        let certificate_2 = Certificate::new_for_test(network_2, 0);
        pending_store
            .insert_pending_certificate(network_1, 0, &certificate_1)
            .unwrap();
        pending_store
            .insert_generated_proof(&certificate_1.hash(), &Proof::new_for_test())
            .unwrap();

        pending_store
            .insert_pending_certificate(network_2, 0, &certificate_2)
            .unwrap();
        pending_store
            .insert_generated_proof(&certificate_2.hash(), &Proof::new_for_test())
            .unwrap();

        epoch_10.add_certificate(network_1, 0).unwrap();
        epoch_10.add_certificate(network_2, 0).unwrap();

        let mut expected_end_checkpoint = BTreeMap::new();

        expected_end_checkpoint.insert(network_1, 0);
        expected_end_checkpoint.insert(network_2, 0);

        let path_15 = config.storage.epochs_db_path.join("15");
        let epoch_15 =
            agglayer_storage::storage::DB::open_cf(&path_15, epochs_db_cf_definitions()).unwrap();

        let mut expected_end_checkpoint_15 = expected_end_checkpoint.clone();

        expected_end_checkpoint_15
            .entry(network_1)
            .and_modify(|e| *e += 1);

        epoch_15
            .multi_insert::<EndCheckpointColumn>(&expected_end_checkpoint_15)
            .unwrap();

        drop(epoch_15);
        drop(epoch_10);
        state_store.set_latest_settled_epoch(10).unwrap();
        state_store.set_latest_opened_epoch(15).unwrap();

        let (sender, _receiver) = tokio::sync::broadcast::channel(1);
        let current_epoch = AtomicU64::new(15);
        let clock_ref = ClockRef::new(
            sender,
            Arc::new(current_epoch),
            Arc::new(Default::default()),
        );

        let result = EpochSynchronizer::start(state_store.clone(), epochs_store.clone(), clock_ref)
            .await
            .unwrap();

        assert_eq!(result.get_epoch_number(), 15);

        drop(result);
        let epoch_10 = epochs_store.open(10).unwrap();
        assert_eq!(epoch_10.get_epoch_number(), 10);
        assert_eq!(epoch_10.get_start_checkpoint(), &start_checkpoint);
        assert_eq!(epoch_10.get_end_checkpoint(), expected_end_checkpoint);

        let epoch_11 = epochs_store.open(11).unwrap();
        assert_eq!(epoch_11.get_epoch_number(), 11);
        assert_eq!(epoch_11.get_start_checkpoint(), &expected_end_checkpoint);
        assert_eq!(epoch_11.get_end_checkpoint(), expected_end_checkpoint);

        let epoch_12 = epochs_store.open(12).unwrap();
        assert_eq!(epoch_12.get_epoch_number(), 12);
        assert_eq!(epoch_12.get_start_checkpoint(), &expected_end_checkpoint);
        assert_eq!(epoch_12.get_end_checkpoint(), expected_end_checkpoint);

        let epoch_13 = epochs_store.open(13).unwrap();
        assert_eq!(epoch_13.get_epoch_number(), 13);
        assert_eq!(epoch_13.get_start_checkpoint(), &expected_end_checkpoint);
        assert_eq!(epoch_13.get_end_checkpoint(), expected_end_checkpoint);

        let epoch_14 = epochs_store.open(14).unwrap();
        assert_eq!(epoch_14.get_epoch_number(), 14);
        assert_eq!(epoch_14.get_start_checkpoint(), &expected_end_checkpoint);
        assert_eq!(epoch_14.get_end_checkpoint(), expected_end_checkpoint);

        let epoch_15 = epochs_store.open(15).unwrap();
        assert_eq!(epoch_15.get_epoch_number(), 15);
        assert_eq!(epoch_15.get_start_checkpoint(), &expected_end_checkpoint);
        assert_eq!(epoch_15.get_end_checkpoint(), expected_end_checkpoint_15);

        assert_eq!(state_store.get_latest_opened_epoch().unwrap(), Some(15));
        assert_eq!(state_store.get_latest_settled_epoch().unwrap(), Some(14));
    }
}
