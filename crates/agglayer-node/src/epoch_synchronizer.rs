use std::sync::Arc;

use agglayer_clock::ClockRef;
use agglayer_storage::{
    error::Error as StorageError,
    stores::{
        EpochStoreWriter, MetadataReader, MetadataWriter, PerEpochReader, PerEpochWriter,
        StateReader,
    },
};
use anyhow::Result;
use tracing::{debug, error, info};

pub(crate) struct EpochSynchronizer {}

impl EpochSynchronizer {
    fn walk_epochs<EpochsStore>(
        epochs_store: Arc<EpochsStore>,
        mut opened_epoch: EpochsStore::PerEpochStore,
        mut current_epoch_number: u64,
        mut epoch_stream: tokio::sync::broadcast::Receiver<agglayer_clock::Event>,
    ) -> Result<EpochsStore::PerEpochStore>
    where
        EpochsStore: EpochStoreWriter,
        EpochsStore::PerEpochStore: PerEpochReader + PerEpochWriter,
    {
        while opened_epoch.get_epoch_number() < current_epoch_number {
            match opened_epoch.start_packing() {
                Err(StorageError::AlreadyPacked(_)) => {
                    info!(
                        "Epoch {} already packed, continue",
                        opened_epoch.get_epoch_number()
                    );
                }
                Err(error) => {
                    error!(
                        "Error starting packing for epoch {}: {:?}",
                        opened_epoch.get_epoch_number(),
                        error
                    );

                    return Err(error.into());
                }
                Ok(_) => {}
            }
            opened_epoch = epochs_store.open_with_start_checkpoint(
                opened_epoch.get_epoch_number() + 1,
                opened_epoch.get_end_checkpoint(),
            )?;

            if let Ok(agglayer_clock::Event::EpochEnded(n)) = epoch_stream.try_recv() {
                current_epoch_number = n;
            }
        }

        Ok(opened_epoch)
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

        debug!("synchronizer: Current epoch: {}", current_epoch_number);
        let opened_epoch = match lse_number {
            // No LSE, we start from epoch 0
            None => epochs_store.open(0)?,

            Some(lse_number) => {
                let lse = epochs_store.open(lse_number)?;
                epochs_store.open_with_start_checkpoint(
                    lse.get_epoch_number() + 1,
                    lse.get_end_checkpoint(),
                )?
            }
        };

        Self::walk_epochs(
            epochs_store,
            opened_epoch,
            current_epoch_number,
            epoch_stream,
        )
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, num::NonZeroU64, sync::atomic::AtomicU64};

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
    use agglayer_types::{Certificate, ExecutionMode, Height, NetworkId, Proof};
    use mockall::{predicate::eq, Sequence};

    use super::*;

    #[tokio::test]
    async fn no_lse_no_previous_start_from_genesis() {
        let mut state_store = MockStateStore::new();
        state_store
            .expect_get_latest_settled_epoch()
            .once()
            .returning(|| Ok(None));

        let mut epochs_store = MockEpochsStore::new();
        epochs_store.expect_open().with(eq(0)).returning(|epoch| {
            let mut mock = MockPerEpochStore::new();
            mock.expect_get_epoch_number().returning(move || epoch);
            mock.expect_start_packing().once().returning(|| Ok(()));
            mock.expect_get_end_checkpoint()
                .once()
                .returning(BTreeMap::new);

            Ok(mock)
        });

        let mut seq = Sequence::new();

        // We expect to open epochs 1 to 9, settling each one
        for i in 1..=9 {
            epochs_store
                .expect_open_with_start_checkpoint()
                .once()
                .in_sequence(&mut seq)
                .with(eq(i), eq(BTreeMap::new()))
                .returning(|epoch, end_checkpoint: BTreeMap<NetworkId, Height>| {
                    let mut mock = MockPerEpochStore::new();
                    mock.expect_get_epoch_number().returning(move || epoch);
                    mock.expect_start_packing().once().returning(|| Ok(()));
                    mock.expect_get_end_checkpoint()
                        .once()
                        .return_once(move || end_checkpoint.clone());

                    Ok(mock)
                });
        }
        epochs_store
            .expect_open_with_start_checkpoint()
            .once()
            .in_sequence(&mut seq)
            .with(eq(10), eq(BTreeMap::new()))
            .returning(|epoch, end_checkpoint: BTreeMap<NetworkId, Height>| {
                let mut mock = MockPerEpochStore::new();
                mock.expect_get_epoch_number().returning(move || epoch);
                mock.expect_start_packing().never().returning(|| Ok(()));
                mock.expect_get_end_checkpoint()
                    .never()
                    .return_once(move || end_checkpoint.clone());

                Ok(mock)
            });
        let (sender, _receiver) = tokio::sync::broadcast::channel(1);
        let current_block = AtomicU64::new(10);
        let clock_ref = ClockRef::new(
            sender,
            Arc::new(current_block),
            Arc::new(NonZeroU64::new(1).unwrap()),
        );

        let result =
            EpochSynchronizer::start(Arc::new(state_store), Arc::new(epochs_store), clock_ref)
                .await;

        assert!(result.is_ok());

        let epoch = result.unwrap();

        assert_eq!(epoch.get_epoch_number(), 10);
    }

    #[test_log::test(tokio::test)]
    async fn lse_number_is_defined() {
        let mut state_store = MockStateStore::new();
        state_store
            .expect_get_latest_settled_epoch()
            .once()
            .returning(|| Ok(Some(10)));

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

        let mut seq = Sequence::new();
        for i in 11..=14 {
            let end_checkpoint = start_checkpoint.clone();
            epochs_store
                .expect_open_with_start_checkpoint()
                .once()
                .in_sequence(&mut seq)
                .with(eq(i), eq(start_checkpoint.clone()))
                .return_once(move |epoch, _start_checkpoint| {
                    let mut mock = MockPerEpochStore::new();
                    mock.expect_get_epoch_number().returning(move || epoch);
                    mock.expect_start_packing().once().returning(|| Ok(()));
                    mock.expect_get_end_checkpoint()
                        .once()
                        .return_once(move || end_checkpoint.clone());
                    Ok(mock)
                });
        }
        epochs_store
            .expect_open_with_start_checkpoint()
            .once()
            .in_sequence(&mut seq)
            .with(eq(15), eq(start_checkpoint))
            .returning(|epoch, start_checkpoint| {
                let mut mock = MockPerEpochStore::new();
                mock.expect_get_epoch_number().returning(move || epoch);
                mock.expect_start_packing().never().returning(|| Ok(()));
                mock.expect_get_end_checkpoint()
                    .never()
                    .return_once(move || start_checkpoint.clone());

                Ok(mock)
            });

        let (sender, _receiver) = tokio::sync::broadcast::channel(1);
        let current_block = AtomicU64::new(15);
        let clock_ref = ClockRef::new(
            sender,
            Arc::new(current_block),
            Arc::new(NonZeroU64::new(1).unwrap()),
        );

        let result =
            EpochSynchronizer::start(Arc::new(state_store), Arc::new(epochs_store), clock_ref)
                .await;

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
            .insert_generated_proof(&certificate_1.hash(), &Proof::dummy())
            .unwrap();

        pending_store
            .insert_pending_certificate(network_2, 0, &certificate_2)
            .unwrap();
        pending_store
            .insert_generated_proof(&certificate_2.hash(), &Proof::dummy())
            .unwrap();

        epoch_10
            .add_certificate(network_1, 0, ExecutionMode::Default)
            .unwrap();
        epoch_10
            .add_certificate(network_2, 0, ExecutionMode::Default)
            .unwrap();

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

        let (sender, _receiver) = tokio::sync::broadcast::channel(1);
        let current_block = AtomicU64::new(15);
        let clock_ref = ClockRef::new(
            sender,
            Arc::new(current_block),
            Arc::new(NonZeroU64::new(1).unwrap()),
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

        assert_eq!(state_store.get_latest_settled_epoch().unwrap(), Some(14));
    }

    #[test_log::test(tokio::test)]
    async fn current_epoch_should_be_open() {
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
            .insert_generated_proof(&certificate_1.hash(), &Proof::dummy())
            .unwrap();

        pending_store
            .insert_pending_certificate(network_2, 0, &certificate_2)
            .unwrap();
        pending_store
            .insert_generated_proof(&certificate_2.hash(), &Proof::dummy())
            .unwrap();

        epoch_10
            .add_certificate(network_1, 0, ExecutionMode::Default)
            .unwrap();
        epoch_10
            .add_certificate(network_2, 0, ExecutionMode::Default)
            .unwrap();

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

        let (sender, _receiver) = tokio::sync::broadcast::channel(1);
        let current_block = AtomicU64::new(15);
        let clock_ref = ClockRef::new(
            sender,
            Arc::new(current_block),
            Arc::new(NonZeroU64::new(1).unwrap()),
        );

        let result = EpochSynchronizer::start(state_store.clone(), epochs_store.clone(), clock_ref)
            .await
            .unwrap();

        assert_eq!(result.get_epoch_number(), 15);

        drop(result);
        let epoch_15 = epochs_store.open(15).unwrap();
        assert_eq!(epoch_15.get_epoch_number(), 15);
        assert_eq!(epoch_15.get_start_checkpoint(), &expected_end_checkpoint);
        assert_eq!(epoch_15.get_end_checkpoint(), expected_end_checkpoint_15);
        assert!(!epoch_15.is_epoch_packed());

        let epoch_14 = epochs_store.open(14).unwrap();
        assert!(epoch_14.is_epoch_packed());

        assert_eq!(state_store.get_latest_settled_epoch().unwrap(), Some(14));
    }
}
