use std::sync::Arc;

use agglayer_certificate_orchestrator::EpochPacker;
use agglayer_clock::ClockRef;
use agglayer_storage::stores::{
    EpochStoreWriter, MetadataReader, PerEpochReader, PerEpochWriter, StateReader,
};
use agglayer_types::{CertificateHeader, CertificateId, CertificateStatus};
use anyhow::Result;
use futures::future::try_join_all;
use tracing::{debug, warn};

pub(crate) struct EpochSynchronizer {}

impl EpochSynchronizer {
    async fn walk_epochs<StateStore, EpochsStore, Packer>(
        state_store: Arc<StateStore>,
        epochs_store: Arc<EpochsStore>,
        mut opened_epoch: EpochsStore::PerEpochStore,
        mut current_epoch_number: u64,
        mut epoch_stream: tokio::sync::broadcast::Receiver<agglayer_clock::Event>,
        packer: Arc<Packer>,
    ) -> Result<EpochsStore::PerEpochStore>
    where
        StateStore: StateReader,
        EpochsStore: EpochStoreWriter,
        EpochsStore::PerEpochStore: PerEpochReader + PerEpochWriter,
        Packer: EpochPacker<PerEpochStore = <EpochsStore as EpochStoreWriter>::PerEpochStore>,
    {
        while opened_epoch.get_epoch_number() < current_epoch_number {
            let epoch_number = opened_epoch.get_epoch_number();
            let certificates = opened_epoch.get_certificates()?;

            let certificate_ids: Vec<CertificateId> =
                certificates.iter().map(|(_, id)| id.hash()).collect();
            let opened_epoch_arc = Arc::new(opened_epoch);

            let headers = state_store.get_certificate_headers(&certificate_ids[..])?;
            let mut futs = Vec::new();
            for (header, (index, certificate)) in headers.iter().zip(certificates) {
                match header {
                    Some(CertificateHeader {
                        epoch_number: Some(epoch_number_header),
                        certificate_index: Some(certificate_index),
                        certificate_id,
                        tx_hash: None,
                        status: CertificateStatus::Candidate,
                        ..
                    }) if *epoch_number_header == epoch_number && *certificate_index == index => {
                        debug!("Settling certificate {}", certificate_id);

                        futs.push(packer.settle_certificate(
                            opened_epoch_arc.clone(),
                            index,
                            *certificate_id,
                        )?);
                    }
                    Some(CertificateHeader {
                        certificate_id,
                        status: CertificateStatus::Settled,
                        ..
                    }) => {
                        debug!("Certificate {} is already settled", certificate_id);
                    }
                    e => {
                        warn!(
                            "Certificate {} is not candidate in epoch {} => {:?}",
                            certificate.hash(),
                            epoch_number,
                            e
                        );
                    }
                }
            }

            try_join_all(futs).await?;

            opened_epoch = Arc::try_unwrap(opened_epoch_arc)
                .map_err(|_| anyhow::Error::msg("Failed to unwrap PerEpochStore"))?;

            opened_epoch.start_packing()?;
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

    pub async fn start<StateStore, EpochsStore, Packer>(
        state_store: Arc<StateStore>,
        epochs_store: Arc<EpochsStore>,
        clock_ref: ClockRef,
        packer: Arc<Packer>,
    ) -> Result<EpochsStore::PerEpochStore>
    where
        StateStore: StateReader + MetadataReader,
        EpochsStore: EpochStoreWriter,
        EpochsStore::PerEpochStore: PerEpochReader + PerEpochWriter,
        Packer: EpochPacker<PerEpochStore = <EpochsStore as EpochStoreWriter>::PerEpochStore>,
    {
        // Get current epoch
        let current_epoch_number = clock_ref.current_epoch();
        let epoch_stream = clock_ref.subscribe()?;

        // Get the latest settled epoch
        let lse_number = state_store.get_latest_settled_epoch()?;

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
            state_store,
            epochs_store,
            opened_epoch,
            current_epoch_number,
            epoch_stream,
            packer,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, sync::atomic::AtomicU64, time::Duration};

    use agglayer_aggregator_notifier::EpochPackerClient;
    use agglayer_certificate_orchestrator::EpochPacker;
    use agglayer_config::Config;
    use agglayer_contracts::{polygon_rollup_manager::PolygonRollupManager, L1RpcClient};
    use agglayer_storage::{
        columns::epochs::end_checkpoint::EndCheckpointColumn,
        storage::epochs_db_cf_definitions,
        stores::{
            epochs::EpochsStore, pending::PendingStore, per_epoch::PerEpochStore,
            state::StateStore, MetadataWriter, PendingCertificateWriter, PerEpochWriter,
            StateWriter,
        },
        tests::{
            mocks::{MockEpochsStore, MockPerEpochStore, MockStateStore},
            TempDBDir,
        },
    };
    use agglayer_types::{Certificate, CertificateIndex, Hash, Height, NetworkId, Proof};
    use ethers::{
        providers::{Middleware, Provider},
        types::{FeeHistory, OtherFields, Transaction, TransactionReceipt, H256, U256},
        utils::Anvil,
    };
    use futures::future::BoxFuture;
    use mockall::{predicate::eq, Sequence};
    use rstest::rstest;

    use super::*;

    mockall::mock!(
        CustomPacker{}

        impl EpochPacker for CustomPacker {
            type PerEpochStore = MockPerEpochStore;
            fn settle_certificate(
                &self,
                epoch: Arc<MockPerEpochStore>,
                index: CertificateIndex,
                certificate_id: CertificateId,
            ) -> Result<BoxFuture<'static, Result<(), agglayer_certificate_orchestrator::Error>>, agglayer_certificate_orchestrator::Error>;

            fn pack(
                &self,
                closing_epoch: Arc<MockPerEpochStore>,
            ) -> Result<BoxFuture<'static, Result<(), agglayer_certificate_orchestrator::Error>>, agglayer_certificate_orchestrator::Error>;
        }
    );

    #[tokio::test]
    async fn no_lse_no_previous_start_from_genesis() {
        let mut state_store = MockStateStore::new();
        state_store
            .expect_get_latest_settled_epoch()
            .once()
            .returning(|| Ok(None));

        state_store
            .expect_get_certificate_headers()
            .returning(|_| Ok(Vec::new()));

        let mut epochs_store = MockEpochsStore::new();
        epochs_store.expect_open().with(eq(0)).returning(|epoch| {
            let mut mock = MockPerEpochStore::new();
            mock.expect_get_epoch_number().returning(move || epoch);
            mock.expect_start_packing().once().returning(|| Ok(()));
            mock.expect_get_certificates()
                .once()
                .returning(|| Ok(Vec::new()));

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
                    mock.expect_get_certificates()
                        .once()
                        .returning(|| Ok(Vec::new()));
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
                mock.expect_get_certificates()
                    .never()
                    .returning(|| Ok(Vec::new()));

                mock.expect_get_end_checkpoint()
                    .never()
                    .return_once(move || end_checkpoint.clone());

                Ok(mock)
            });
        let (sender, _receiver) = tokio::sync::broadcast::channel(1);
        let current_epoch = AtomicU64::new(10);
        let clock_ref = ClockRef::new(
            sender,
            Arc::new(current_epoch),
            Arc::new(Default::default()),
        );

        let result = EpochSynchronizer::start(
            Arc::new(state_store),
            Arc::new(epochs_store),
            clock_ref,
            Arc::new(MockCustomPacker::new()),
        )
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

        let v: Vec<CertificateId> = Vec::new();
        state_store
            .expect_get_certificate_headers()
            .with(eq(v))
            .returning(|_| Ok(Vec::new()));

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
                    mock.expect_get_certificates()
                        .once()
                        .returning(|| Ok(Vec::new()));
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
        let current_epoch = AtomicU64::new(15);
        let clock_ref = ClockRef::new(
            sender,
            Arc::new(current_epoch),
            Arc::new(Default::default()),
        );

        let result = EpochSynchronizer::start(
            Arc::new(state_store),
            Arc::new(epochs_store),
            clock_ref,
            Arc::new(MockCustomPacker::new()),
        )
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

        mockall::mock!(
            DummyPacker{}
            impl EpochPacker for DummyPacker {
                type PerEpochStore = PerEpochStore<PendingStore, StateStore>;
                fn settle_certificate(
                    &self,
                    epoch: Arc<PerEpochStore<PendingStore, StateStore>>,
                    index: CertificateIndex,
                    certificate_id: CertificateId,
                ) -> Result<BoxFuture<'static, Result<(), agglayer_certificate_orchestrator::Error>>, agglayer_certificate_orchestrator::Error>;

                fn pack(
                    &self,
                    closing_epoch: Arc<PerEpochStore<PendingStore, StateStore>>,
                ) -> Result<BoxFuture<'static, Result<(), agglayer_certificate_orchestrator::Error>>, agglayer_certificate_orchestrator::Error>;
            }
        );
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

        let (sender, _receiver) = tokio::sync::broadcast::channel(1);
        let current_epoch = AtomicU64::new(15);
        let clock_ref = ClockRef::new(
            sender,
            Arc::new(current_epoch),
            Arc::new(Default::default()),
        );

        let result = EpochSynchronizer::start(
            state_store.clone(),
            epochs_store.clone(),
            clock_ref,
            Arc::new(MockDummyPacker::new()),
        )
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

    #[rstest]
    #[test_log::test(tokio::test)]
    #[timeout(Duration::from_secs(5))]
    async fn lse_number_is_less_than_loe_real_db_with_sync() {
        let tmp = TempDBDir::new();
        let config = Arc::new(Config::new(&tmp.path));
        let state_store =
            Arc::new(StateStore::new_with_path(&config.storage.state_db_path).unwrap());
        let pending_store =
            Arc::new(PendingStore::new_with_path(&config.storage.pending_db_path).unwrap());

        mockall::mock!(
            DummyPacker{}
            impl EpochPacker for DummyPacker {
                type PerEpochStore = PerEpochStore<PendingStore, StateStore>;
                fn settle_certificate(
                    &self,
                    epoch: Arc<PerEpochStore<PendingStore, StateStore>>,
                    index: CertificateIndex,
                    certificate_id: CertificateId,
                ) -> Result<BoxFuture<'static, Result<(), agglayer_certificate_orchestrator::Error>>, agglayer_certificate_orchestrator::Error>;

                fn pack(
                    &self,
                    closing_epoch: Arc<PerEpochStore<PendingStore, StateStore>>,
                ) -> Result<BoxFuture<'static, Result<(), agglayer_certificate_orchestrator::Error>>, agglayer_certificate_orchestrator::Error>;
            }
        );
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

        let certificate = Certificate::new_for_test(network_1, 1);
        pending_store
            .insert_pending_certificate(network_1, 1, &certificate)
            .unwrap();
        let proof = Proof::new_for_test();
        pending_store
            .insert_generated_proof(&certificate.hash(), &proof)
            .unwrap();
        state_store
            .insert_certificate_header(&certificate, CertificateStatus::Proven)
            .unwrap();

        let epoch_13 = epochs_store
            .open_with_start_checkpoint(13, epoch_10.get_end_checkpoint().clone())
            .unwrap();

        epoch_13.add_certificate(network_1, 1).unwrap();

        assert_eq!(epoch_13.get_end_checkpoint().get(&network_1), Some(&1));
        drop(epoch_13);

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
        let current_epoch = AtomicU64::new(15);
        let clock_ref = ClockRef::new(
            sender,
            Arc::new(current_epoch),
            Arc::new(Default::default()),
        );
        let anvil = Anvil::new().block_time(1u64).spawn();
        let anvil_provider =
            Provider::<ethers::providers::Http>::try_from(anvil.endpoint()).unwrap();

        let (provider, mock) = Provider::mocked();
        let block_number = anvil_provider.get_block_number().await.unwrap();

        let block = anvil_provider
            .get_block(block_number)
            .await
            .unwrap()
            .unwrap();

        let tx = Transaction {
            hash: H256([1u8; 32]),
            nonce: U256::zero(),
            block_number: Some(1.into()),
            block_hash: Some([1u8; 32].into()),
            transaction_index: Some(0.into()),
            from: ethers::types::H160::zero(),
            to: None,
            value: U256::zero(),
            gas_price: None,
            gas: U256::zero(),
            input: ethers::types::Bytes::new(),
            v: 0.into(),
            r: U256::zero(),
            s: U256::zero(),
            transaction_type: None,
            access_list: None,
            max_priority_fee_per_gas: None,
            max_fee_per_gas: None,
            chain_id: None,
            other: OtherFields::default(),
        };

        let receipt = TransactionReceipt::default();
        let estimate_gas = U256::from([1u8; 32]);
        let tx_hash = H256::from([2u8; 32]);

        mock.push(receipt).unwrap();
        mock.push(tx).unwrap();
        mock.push(tx_hash).unwrap();
        mock.push(estimate_gas).unwrap();
        mock.push(FeeHistory {
            base_fee_per_gas: Vec::new(),
            gas_used_ratio: Vec::new(),
            oldest_block: U256::zero(),
            reward: Vec::new(),
        })
        .unwrap();
        mock.push(block).unwrap();

        let config = Config::default();

        let inner = PolygonRollupManager::new(
            config.l1.rollup_manager_contract,
            Arc::new(provider.clone()),
        );

        let l1_rpc = L1RpcClient::new(inner);

        let mut cfg = config.outbound.rpc.settle.clone();
        cfg.confirmations = 0;
        cfg.retry_interval = Duration::from_millis(100);

        let packer =
            EpochPackerClient::try_new(Arc::new(cfg), state_store.clone(), Arc::new(l1_rpc))
                .unwrap();

        let result = EpochSynchronizer::start(
            state_store.clone(),
            epochs_store.clone(),
            clock_ref,
            Arc::new(packer),
        )
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

        let mut expected_end_checkpoint = expected_end_checkpoint.clone();
        let epoch_13 = epochs_store.open(13).unwrap();
        assert_eq!(epoch_13.get_epoch_number(), 13);
        assert_eq!(epoch_13.get_start_checkpoint(), &expected_end_checkpoint);

        let n1 = expected_end_checkpoint.get_mut(&network_1).unwrap();
        *n1 += 1;

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

        let header = state_store
            .get_certificate_header(&certificate.hash())
            .unwrap()
            .unwrap();
        assert_eq!(header.tx_hash, Some(Hash(*tx_hash.as_fixed_bytes())));
    }
}
