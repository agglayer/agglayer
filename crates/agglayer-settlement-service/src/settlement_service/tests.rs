use std::{
    future::pending,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    task::{Context, Poll},
    time::Duration,
};

use agglayer_config::Multiplier;
use agglayer_storage::{
    error::Error as StorageError, stores::EditEvenIfCompleted, tests::mocks::MockStateStore,
};
use agglayer_types::{
    CertificateId, ClientErrorType, ContractCallOutcome, ContractCallResult, Digest, Nonce,
    RpcErrorCode, SettlementAttempt, SettlementAttemptNumber, SettlementAttemptResult,
    SettlementJob, SettlementJobId, SettlementJobResult, SettlementTxHash, B256, U256,
};
use alloy::{
    network::EthereumWallet,
    primitives::U64,
    providers::{mock::Asserter, ProviderBuilder},
    rpc::{client::RpcClient, json_rpc::RequestPacket},
    signers::local::PrivateKeySigner,
    transports::{TransportError, TransportFut},
};

use super::*;
use crate::settlement_task::{
    SettlementTask, StoredSettlementJob, TaskAdminCommand, TaskControlHandle,
};

fn mk_provider() -> impl Provider + WalletProvider + 'static {
    ProviderBuilder::new()
        .wallet(EthereumWallet::from(
            PrivateKeySigner::from_slice(&[0x11; 32]).expect("valid test signing key"),
        ))
        .connect_http(
            "http://127.0.0.1:0"
                .parse()
                .expect("test provider URL should parse"),
        )
}

fn mk_provider_with_gas_estimate(gas_estimate: u64) -> impl Provider + WalletProvider + 'static {
    let asserter = Asserter::new();
    asserter.push_success(&U64::from(gas_estimate));
    ProviderBuilder::new()
        .wallet(EthereumWallet::from(
            PrivateKeySigner::from_slice(&[0x11; 32]).expect("valid test signing key"),
        ))
        .connect_mocked_client(asserter)
}

/// Mock transport that records the first request and keeps it pending when
/// its asserter has no queued response. This gives service tests a live task
/// parked in an L1 call, rather than a task that panics against a dead HTTP
/// endpoint.
#[derive(Clone, Debug)]
struct ParkingAsserterTransport {
    asserter: Asserter,
    request_count: Arc<AtomicUsize>,
}

impl tower::Service<RequestPacket> for ParkingAsserterTransport {
    type Response = alloy::rpc::json_rpc::ResponsePacket;
    type Error = TransportError;
    type Future = TransportFut<'static>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _request: RequestPacket) -> Self::Future {
        assert!(
            self.asserter.read_q().is_empty(),
            "parking transport must not have a queued response"
        );
        self.request_count.fetch_add(1, Ordering::SeqCst);
        Box::pin(pending())
    }
}

fn mk_parked_provider() -> (impl Provider + WalletProvider + 'static, Arc<AtomicUsize>) {
    let request_count = Arc::new(AtomicUsize::new(0));
    let client = RpcClient::new(
        ParkingAsserterTransport {
            asserter: Asserter::new(),
            request_count: request_count.clone(),
        },
        true,
    );
    let provider = ProviderBuilder::new()
        .wallet(EthereumWallet::from(
            PrivateKeySigner::from_slice(&[0x11; 32]).expect("valid test signing key"),
        ))
        .connect_client(client);
    (provider, request_count)
}

async fn wait_until(mut condition: impl FnMut() -> bool) {
    tokio::time::timeout(Duration::from_secs(10), async {
        while !condition() {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("condition should become true");
}

async fn wait_until_l1_request_is_parked(request_count: &AtomicUsize) {
    wait_until(|| request_count.load(Ordering::SeqCst) > 0).await;
}

fn expect_empty_startup_recovery(store: &mut MockStateStore) {
    store
        .expect_list_settlement_job_ids()
        .once()
        .return_once(|| Ok(Vec::new()));
}

async fn mk_service(
    store: Arc<MockStateStore>,
) -> SettlementService<impl Provider + WalletProvider + 'static, MockStateStore> {
    mk_service_with_token(store, CancellationToken::new()).await
}

async fn mk_service_with_token(
    store: Arc<MockStateStore>,
    cancellation_token: CancellationToken,
) -> SettlementService<impl Provider + WalletProvider + 'static, MockStateStore> {
    SettlementService::start(
        SettlementServiceConfig::default(),
        Arc::new(SettlementTransactionConfig::default()),
        Arc::new(mk_provider()),
        store,
        cancellation_token,
    )
    .await
    .expect("settlement service should start")
    .0
}

fn mk_job_id(seed: u128) -> SettlementJobId {
    SettlementJobId::from(ulid::Ulid::from(seed))
}

fn mk_job(seed: u8) -> SettlementJob {
    SettlementJob {
        contract_address: agglayer_types::Address::from([seed; 20]),
        calldata: vec![seed, seed.wrapping_add(1)].into(),
        eth_value: U256::from(seed),
        gas_limit: seed as u128 + 100_000,
    }
}

fn expect_pending_job_load(store: &mut MockStateStore, job_id: SettlementJobId, seed: u8) {
    let job = mk_job(seed);
    let attempt = mk_resolved_attempt(
        seed,
        SettlementTxHash::new(Digest::from([seed.wrapping_add(1); 32])),
    );
    store
        .expect_get_settlement_job()
        .once()
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .return_once(move |_| Ok(Some(job)));
    store
        .expect_get_settlement_job_result()
        .once()
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .return_once(|_| Ok(None));
    store
        .expect_list_settlement_attempt_results()
        .once()
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .return_once(|_| Ok(Vec::new()));
    store
        .expect_list_settlement_attempts()
        .once()
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .return_once(move |_| Ok(vec![(0, attempt)]));
}

fn mk_result(seed: u8, outcome: ContractCallOutcome) -> SettlementJobResult {
    SettlementJobResult {
        wallet: agglayer_types::Address::from([seed.wrapping_add(3); 20]),
        nonce: Nonce(seed as u64 + 200),
        attempt_number: SettlementAttemptNumber(seed as u64 + 300),
        contract_call_result: ContractCallResult {
            outcome,
            metadata: vec![seed, seed.wrapping_add(1)].into(),
            block_hash: B256::from([seed; 32]),
            block_number: seed as u64,
            tx_hash: SettlementTxHash::new(Digest::from([seed.wrapping_add(2); 32])),
        },
    }
}

#[tokio::test]
async fn start_scans_jobs_and_skips_completed_ones() {
    let mut store = MockStateStore::new();
    let job_id = mk_job_id(9);
    let job = mk_job(9);
    let result = mk_result(9, ContractCallOutcome::Success);

    store
        .expect_list_settlement_job_ids()
        .once()
        .return_once(move || Ok(vec![job_id]));
    store
        .expect_get_settlement_job()
        .once()
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .return_once(move |_| Ok(Some(job)));
    store
        .expect_get_settlement_job_result()
        .once()
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .return_once(move |_| Ok(Some(result)));
    store.expect_list_settlement_attempts().never();
    store.expect_list_settlement_attempt_results().never();

    let service = mk_service(Arc::new(store)).await;

    assert!(service
        .task_controls
        .lock()
        .expect("settlement task_controls lock poisoned")
        .is_empty());
    assert!(service.result_watchers.lock().await.is_empty());
}

#[tokio::test]
async fn start_skips_unloadable_jobs_and_keeps_scanning() {
    let mut store = MockStateStore::new();
    let unloadable_job_id = mk_job_id(8);
    let completed_job_id = mk_job_id(9);
    let completed_job = mk_job(9);
    let completed_result = mk_result(9, ContractCallOutcome::Success);

    // The unloadable job comes first: startup must skip it and still
    // process the following one.
    store
        .expect_list_settlement_job_ids()
        .once()
        .return_once(move || Ok(vec![unloadable_job_id, completed_job_id]));
    store
        .expect_get_settlement_job()
        .once()
        .withf(move |requested_job_id| requested_job_id == &unloadable_job_id)
        .return_once(|_| {
            Err(agglayer_storage::error::Error::UnprocessedAction(
                "corrupt settlement job row".into(),
            ))
        });
    store
        .expect_get_settlement_job()
        .once()
        .withf(move |requested_job_id| requested_job_id == &completed_job_id)
        .return_once(move |_| Ok(Some(completed_job)));
    store
        .expect_get_settlement_job_result()
        .once()
        .withf(move |requested_job_id| requested_job_id == &completed_job_id)
        .return_once(move |_| Ok(Some(completed_result)));

    let (service, recovery_skipped_jobs) = SettlementService::start(
        SettlementServiceConfig::default(),
        Arc::new(SettlementTransactionConfig::default()),
        Arc::new(mk_provider()),
        Arc::new(store),
        CancellationToken::new(),
    )
    .await
    .expect("settlement service should start");

    assert_eq!(recovery_skipped_jobs, 1);
    assert!(service
        .task_controls
        .lock()
        .expect("settlement task_controls lock poisoned")
        .is_empty());
    assert!(service.result_watchers.lock().await.is_empty());
}

#[tokio::test]
async fn retrieve_uses_in_memory_watcher_before_storage() {
    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    let service = mk_service(Arc::new(store)).await;
    let job_id = mk_job_id(1);
    let in_memory_result = mk_result(2, ContractCallOutcome::Revert);

    let (_sender, watcher) = watch::channel(Some(in_memory_result.clone()));
    service.result_watchers.lock().await.insert(job_id, watcher);

    let retrieved = service
        .retrieve_settlement_result(job_id)
        .await
        .expect("retrieval should succeed");

    match retrieved {
        RetrievedSettlementResult::Completed(result) => assert_eq!(result, in_memory_result),
        RetrievedSettlementResult::Pending(_) => panic!("expected completed result"),
    }
}

#[tokio::test]
async fn retrieve_uses_stored_terminal_result_without_watcher() {
    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    let job_id = mk_job_id(2);
    let stored_result = mk_result(3, ContractCallOutcome::Success);
    let stored_result_for_store = stored_result.clone();

    store
        .expect_get_settlement_job_result()
        .once()
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .return_once(move |_| Ok(Some(stored_result_for_store)));

    let service = mk_service(Arc::new(store)).await;

    let retrieved = service
        .retrieve_settlement_result(job_id)
        .await
        .expect("retrieval should succeed");

    match retrieved {
        RetrievedSettlementResult::Completed(result) => assert_eq!(result, stored_result),
        RetrievedSettlementResult::Pending(_) => panic!("expected completed result"),
    }
}

#[tokio::test]
async fn retrieve_fails_for_unknown_job_id() {
    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    let job_id = mk_job_id(4);

    store
        .expect_get_settlement_job_result()
        .once()
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .return_once(|_| Ok(None));
    store
        .expect_get_settlement_job()
        .once()
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .return_once(|_| Ok(None));

    let service = mk_service(Arc::new(store)).await;

    let result = service.retrieve_settlement_result(job_id).await;
    assert!(result.is_err(), "unknown job should fail");
    let error = result.err().expect("result should be an error");

    assert!(error.to_string().contains("No settlement job found for id"));
}

#[tokio::test]
async fn retrieve_fails_when_pending_job_has_no_running_task() {
    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    let job_id = mk_job_id(5);
    let job = mk_job(5);

    store
        .expect_get_settlement_job_result()
        .once()
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .return_once(|_| Ok(None));
    store
        .expect_get_settlement_job()
        .once()
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .return_once(move |_| Ok(Some(job)));

    let service = mk_service(Arc::new(store)).await;

    let result = service.retrieve_settlement_result(job_id).await;
    assert!(
        result.is_err(),
        "pending job without a watcher should fail as an invariant break"
    );
    let error = result.err().expect("result should be an error");

    // The `RpcErrorCode` tag is the outermost context layer, so `Display`
    // (`to_string()`) now renders just the tag; the original message is still
    // in the chain, which the default `Debug` output includes.
    assert!(format!("{error:?}").contains("exists without a running task"));
    assert_eq!(
        error.downcast_ref::<RpcErrorCode>(),
        Some(&RpcErrorCode::NoLiveTask)
    );
}

#[tokio::test]
async fn reload_and_restart_preserves_watcher_when_reload_finds_completed_job() {
    let mut store = MockStateStore::new();
    let job_id = mk_job_id(6);
    let job = mk_job(6);
    let completed_result = mk_result(6, ContractCallOutcome::Success);
    let completed_result_for_store = completed_result.clone();
    let result_reads = Arc::new(Mutex::new(0usize));

    store
        .expect_list_settlement_job_ids()
        .once()
        .return_once(|| Ok(Vec::new()));
    store
        .expect_get_settlement_job()
        .times(2)
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .returning({
            let job = job.clone();
            move |_| Ok(Some(job.clone()))
        });
    store
        .expect_get_settlement_job_result()
        .times(2)
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .returning(move |_| {
            let mut result_reads = result_reads.lock().unwrap();
            *result_reads += 1;
            if *result_reads == 1 {
                Ok(None)
            } else {
                Ok(Some(completed_result_for_store.clone()))
            }
        });
    store
        .expect_list_settlement_attempt_results()
        .once()
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .return_once(|_| Ok(Vec::new()));
    store
        .expect_list_settlement_attempts()
        .once()
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .return_once(|_| Ok(Vec::new()));

    let store = Arc::new(store);
    let service = mk_service(store).await;
    let (task_control_handle, task_control) = TaskControlHandle::new(&service.cancellation_token);
    task_control_handle
        .try_send(TaskAdminCommand::ReloadAndRestart)
        .expect("reload command should fit in admin channel");
    let task = match SettlementTask::load(
        job_id,
        service.tx_config.clone(),
        service.provider.clone(),
        service.store.clone(),
        service.wallet_nonce_locks.clone(),
        task_control,
    )
    .await
    .expect("settlement task should load")
    {
        StoredSettlementJob::Pending(task) => task,
        StoredSettlementJob::Completed(_) => panic!("initial load should be pending"),
    };

    let mut result_receiver = service
        .spawn_settlement_task(job_id, task, task_control_handle)
        .await;

    result_receiver
        .changed()
        .await
        .expect("reload should publish the stored terminal result");

    assert_eq!(result_receiver.borrow().as_ref(), Some(&completed_result));
    assert!(service
        .task_controls
        .lock()
        .expect("settlement task_controls lock poisoned")
        .is_empty());
    assert!(service.result_watchers.lock().await.contains_key(&job_id));
}

#[tokio::test]
async fn request_new_settlement_persists_job_with_certificate_link() {
    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    let certificate_id = CertificateId::new(Digest::from([7; 32]));
    let job = mk_job(7);
    // `create` multiplies the 200_000 estimate by 2.0 and caps it at 300_000.
    let mut expected_job = job.clone();
    expected_job.gas_limit = 300_000;
    let recorded_job_id = Arc::new(Mutex::new(None));
    let recorded_job_id_for_store = recorded_job_id.clone();

    // The job and both certificate links are persisted in one atomic call.
    store
        .expect_insert_settlement_job_with_certificate()
        .once()
        .withf(move |_, recorded_job, recorded_certificate_id| {
            recorded_job == &expected_job && recorded_certificate_id == &certificate_id
        })
        .return_once(move |settlement_job_id, _, _| {
            *recorded_job_id_for_store.lock().unwrap() = Some(*settlement_job_id);
            Ok(())
        });
    store.expect_insert_settlement_job().never();

    // `create` runs `estimateGas` before persisting. Configure the ceiling
    // strictly between the raw and multiplied estimates so both knobs bind.
    // Live token for estimation, then cancel to stop the spawned task.
    let cancellation_token = CancellationToken::new();
    let tx_config = SettlementTransactionConfig {
        gas_limit_multiplier_factor: Multiplier::from_u64_per_1000(2000),
        gas_limit_ceiling: U256::from(300_000),
        ..SettlementTransactionConfig::default()
    };
    let service = SettlementService::start(
        SettlementServiceConfig::default(),
        Arc::new(tx_config),
        Arc::new(mk_provider_with_gas_estimate(200_000)),
        Arc::new(store),
        cancellation_token.clone(),
    )
    .await
    .expect("settlement service should start")
    .0;

    let watcher = service
        .request_new_settlement(Some(certificate_id), job)
        .await
        .expect("settlement request should be accepted");
    cancellation_token.cancel();

    assert_eq!(*recorded_job_id.lock().unwrap(), Some(watcher.job_id()));
}

#[tokio::test]
async fn admin_abort_unknown_job_is_tagged_not_found() {
    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    let job_id = mk_job_id(20);

    store
        .expect_get_settlement_job()
        .once()
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .return_once(|_| Ok(None));

    let service = mk_service(Arc::new(store)).await;

    let error = service
        .admin_abort_task(job_id)
        .await
        .expect_err("abort on unknown job should fail");

    assert_eq!(
        error.downcast_ref::<RpcErrorCode>(),
        Some(&RpcErrorCode::NotFound)
    );
}

#[tokio::test]
async fn admin_abort_completed_job_is_tagged_already_completed() {
    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    let job_id = mk_job_id(21);
    let job = mk_job(21);
    let result = mk_result(21, ContractCallOutcome::Success);

    store
        .expect_get_settlement_job()
        .once()
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .return_once(move |_| Ok(Some(job)));
    store
        .expect_get_settlement_job_result()
        .once()
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .return_once(move |_| Ok(Some(result)));

    let service = mk_service(Arc::new(store)).await;

    let error = service
        .admin_abort_task(job_id)
        .await
        .expect_err("abort on completed job should fail");

    assert_eq!(
        error.downcast_ref::<RpcErrorCode>(),
        Some(&RpcErrorCode::AlreadyCompleted)
    );
}

#[tokio::test]
async fn admin_abort_pending_job_without_task_is_tagged_no_live_task() {
    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    let job_id = mk_job_id(22);
    let job = mk_job(22);

    store
        .expect_get_settlement_job()
        .once()
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .return_once(move |_| Ok(Some(job)));
    store
        .expect_get_settlement_job_result()
        .once()
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .return_once(|_| Ok(None));

    let service = mk_service(Arc::new(store)).await;

    let error = service
        .admin_abort_task(job_id)
        .await
        .expect_err("abort on pending job without a task should fail");

    assert_eq!(
        error.downcast_ref::<RpcErrorCode>(),
        Some(&RpcErrorCode::NoLiveTask)
    );
}

#[tokio::test]
async fn panicked_task_deregisters_control_and_abort_reports_no_live_task() {
    let mut store = MockStateStore::new();
    let job_id = mk_job_id(28);
    let job = mk_job(28);
    let attempt = mk_resolved_attempt(28, SettlementTxHash::new(Digest::from([29; 32])));

    store
        .expect_list_settlement_job_ids()
        .once()
        .return_once(move || Ok(vec![job_id]));
    store
        .expect_get_settlement_job()
        .times(2)
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .returning(move |_| Ok(Some(job.clone())));
    store
        .expect_get_settlement_job_result()
        .times(2)
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .returning(|_| Ok(None));
    store
        .expect_list_settlement_attempt_results()
        .once()
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .return_once(|_| Ok(Vec::new()));
    store
        .expect_list_settlement_attempts()
        .once()
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .return_once(move |_| Ok(vec![(0, attempt)]));

    // `mk_service` uses the dead endpoint at http://127.0.0.1:0. The task's
    // first L1 query fails non-recoverably and deliberately panics.
    let service = mk_service(Arc::new(store)).await;
    wait_until(|| !service.has_live_task(job_id)).await;

    let error = service
        .admin_abort_task(job_id)
        .await
        .expect_err("abort after task panic should report no live task");

    assert_eq!(
        error.downcast_ref::<RpcErrorCode>(),
        Some(&RpcErrorCode::NoLiveTask)
    );
}

#[tokio::test]
async fn admin_reload_unknown_job_is_tagged_not_found() {
    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    let job_id = mk_job_id(23);

    store
        .expect_get_settlement_job()
        .once()
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .return_once(|_| Ok(None));

    let service = mk_service(Arc::new(store)).await;

    let error = service
        .admin_reload_and_restart_task(job_id)
        .await
        .expect_err("reload on unknown job should fail");

    assert_eq!(
        error.downcast_ref::<RpcErrorCode>(),
        Some(&RpcErrorCode::NotFound)
    );
}

#[tokio::test]
async fn admin_reload_completed_job_is_tagged_already_completed() {
    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    let job_id = mk_job_id(24);
    let job = mk_job(24);
    let result = mk_result(24, ContractCallOutcome::Success);

    store
        .expect_get_settlement_job()
        .once()
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .return_once(move |_| Ok(Some(job)));
    store
        .expect_get_settlement_job_result()
        .once()
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .return_once(move |_| Ok(Some(result)));

    let service = mk_service(Arc::new(store)).await;

    let error = service
        .admin_reload_and_restart_task(job_id)
        .await
        .expect_err("reload on completed job should fail");

    assert_eq!(
        error.downcast_ref::<RpcErrorCode>(),
        Some(&RpcErrorCode::AlreadyCompleted)
    );
}

#[tokio::test]
async fn abort_then_reload_respawns_pending_job() {
    let mut store = MockStateStore::new();
    let job_id = mk_job_id(25);
    let job = mk_job(25);
    let attempt = mk_resolved_attempt(25, SettlementTxHash::new(Digest::from([26; 32])));

    store
        .expect_list_settlement_job_ids()
        .once()
        .return_once(move || Ok(vec![job_id]));
    store
        .expect_get_settlement_job()
        .times(3)
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .returning(move |_| Ok(Some(job.clone())));
    store
        .expect_get_settlement_job_result()
        .times(3)
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .returning(|_| Ok(None));
    store
        .expect_list_settlement_attempt_results()
        .times(2)
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .returning(|_| Ok(Vec::new()));
    store
        .expect_list_settlement_attempts()
        .times(2)
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .returning(move |_| Ok(vec![(0, attempt.clone())]));

    let cancellation_token = CancellationToken::new();
    let (provider, request_count) = mk_parked_provider();
    let service = SettlementService::start(
        SettlementServiceConfig::default(),
        Arc::new(SettlementTransactionConfig::default()),
        Arc::new(provider),
        Arc::new(store),
        cancellation_token.clone(),
    )
    .await
    .expect("settlement service should start")
    .0;
    wait_until_l1_request_is_parked(&request_count).await;

    assert!(service.has_live_task(job_id));
    let old_watcher = service
        .result_watchers
        .lock()
        .await
        .get(&job_id)
        .cloned()
        .expect("the parked task must have a watcher");

    service
        .admin_abort_task(job_id)
        .await
        .expect("the parked task should accept an abort");
    wait_until(|| !service.has_live_task(job_id)).await;
    assert!(!service.result_watchers.lock().await.contains_key(&job_id));

    service
        .admin_reload_and_restart_task(job_id)
        .await
        .expect("reload should respawn the pending job");
    wait_until(|| request_count.load(Ordering::SeqCst) >= 2).await;

    assert!(service.has_live_task(job_id));
    let fresh_watcher = service
        .result_watchers
        .lock()
        .await
        .get(&job_id)
        .cloned()
        .expect("the respawned task must have a watcher");
    assert!(!fresh_watcher.same_channel(&old_watcher));
    assert!(fresh_watcher.borrow().is_none());

    match service
        .retrieve_settlement_result(job_id)
        .await
        .expect("the respawned job should be retrievable")
    {
        RetrievedSettlementResult::Pending(mut watcher) => assert!(
            watcher.watcher().has_changed().is_ok(),
            "the fresh watcher must still have a live sender"
        ),
        RetrievedSettlementResult::Completed(_) => panic!("the respawned job must be pending"),
    }

    cancellation_token.cancel();
}

#[tokio::test]
async fn reload_of_live_task_queues_command() {
    let mut store = MockStateStore::new();
    let job_id = mk_job_id(29);
    store
        .expect_list_settlement_job_ids()
        .once()
        .return_once(move || Ok(vec![job_id]));
    // These strict one-load expectations prove reload queues to the existing
    // control rather than loading and spawning a second task.
    expect_pending_job_load(&mut store, job_id, 29);

    let cancellation_token = CancellationToken::new();
    let (provider, request_count) = mk_parked_provider();
    let service = SettlementService::start(
        SettlementServiceConfig::default(),
        Arc::new(SettlementTransactionConfig::default()),
        Arc::new(provider),
        Arc::new(store),
        cancellation_token.clone(),
    )
    .await
    .expect("settlement service should start")
    .0;
    wait_until_l1_request_is_parked(&request_count).await;

    service
        .admin_reload_and_restart_task(job_id)
        .await
        .expect("reload should queue to the live task");

    assert!(service.has_live_task(job_id));
    assert_eq!(
        service
            .task_controls
            .lock()
            .expect("settlement task_controls lock poisoned")
            .len(),
        1
    );
    assert_eq!(request_count.load(Ordering::SeqCst), 1);
    cancellation_token.cancel();
}

#[tokio::test]
async fn admin_reload_with_full_admin_channel_is_tagged_unavailable() {
    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    let job_id = mk_job_id(26);

    let service = mk_service(Arc::new(store)).await;
    let (task_control_handle, _task_control) = TaskControlHandle::new(&service.cancellation_token);
    while task_control_handle
        .try_send(TaskAdminCommand::ReloadAndRestart)
        .is_ok()
    {}
    service
        .task_controls
        .lock()
        .expect("settlement task_controls lock poisoned")
        .insert(job_id, task_control_handle);

    let error = service
        .admin_reload_and_restart_task(job_id)
        .await
        .expect_err("reload with a full admin command channel should fail");

    assert_eq!(
        error.downcast_ref::<RpcErrorCode>(),
        Some(&RpcErrorCode::Unavailable)
    );
}

#[tokio::test]
async fn reload_retries_after_closed_handle_teardown() {
    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    let job_id = mk_job_id(27);
    let job = mk_job(27);
    let attempt = mk_resolved_attempt(27, SettlementTxHash::new(Digest::from([28; 32])));

    store
        .expect_get_settlement_job()
        .times(2)
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .returning(move |_| Ok(Some(job.clone())));
    store
        .expect_get_settlement_job_result()
        .times(2)
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .returning(|_| Ok(None));
    store
        .expect_list_settlement_attempt_results()
        .once()
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .return_once(|_| Ok(Vec::new()));
    store
        .expect_list_settlement_attempts()
        .once()
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .return_once(move |_| Ok(vec![(0, attempt)]));

    let cancellation_token = CancellationToken::new();
    let (provider, request_count) = mk_parked_provider();
    let service = SettlementService::start(
        SettlementServiceConfig::default(),
        Arc::new(SettlementTransactionConfig::default()),
        Arc::new(provider),
        Arc::new(store),
        cancellation_token.clone(),
    )
    .await
    .expect("settlement service should start")
    .0;
    let (task_control_handle, task_control) = TaskControlHandle::new(&service.cancellation_token);
    // Drop the receiver side so the admin channel is closed rather than full,
    // simulating the window before the task's teardown guard deregisters it.
    drop(task_control);
    service
        .task_controls
        .lock()
        .expect("settlement task_controls lock poisoned")
        .insert(job_id, task_control_handle);
    let (stale_sender, stale_watcher) = watch::channel(None);
    drop(stale_sender);
    service
        .result_watchers
        .lock()
        .await
        .insert(job_id, stale_watcher.clone());

    let error = service
        .admin_reload_and_restart_task(job_id)
        .await
        .expect_err("reload should wait for closed-handle teardown");
    assert_eq!(
        error.downcast_ref::<RpcErrorCode>(),
        Some(&RpcErrorCode::Unavailable)
    );
    assert!(service
        .task_controls
        .lock()
        .expect("settlement task_controls lock poisoned")
        .contains_key(&job_id));

    // Simulate the teardown guard completing. A retry now sees no task and
    // safely follows the normal respawn path.
    service
        .task_controls
        .lock()
        .expect("settlement task_controls lock poisoned")
        .remove(&job_id);
    service
        .admin_reload_and_restart_task(job_id)
        .await
        .expect("reload should respawn after teardown completes");
    wait_until_l1_request_is_parked(&request_count).await;

    assert!(service.has_live_task(job_id));
    let fresh_watcher = service
        .result_watchers
        .lock()
        .await
        .get(&job_id)
        .cloned()
        .expect("respawn should register a fresh watcher");
    assert!(!fresh_watcher.same_channel(&stale_watcher));
    assert!(fresh_watcher.borrow().is_none());
    cancellation_token.cancel();
}

#[tokio::test]
async fn reload_load_failure_leaves_no_registrations() {
    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    let job_id = mk_job_id(30);
    let job = mk_job(30);
    let job_reads = Arc::new(AtomicUsize::new(0));

    store
        .expect_get_settlement_job()
        .times(2)
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .returning({
            let job_reads = job_reads.clone();
            move |_| {
                if job_reads.fetch_add(1, Ordering::SeqCst) == 0 {
                    Ok(Some(job.clone()))
                } else {
                    Err(StorageError::UnprocessedAction(
                        "settlement job row became unreadable".to_owned(),
                    ))
                }
            }
        });
    store
        .expect_get_settlement_job_result()
        .once()
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .return_once(|_| Ok(None));

    let service = mk_service(Arc::new(store)).await;
    let (stale_sender, stale_watcher) = watch::channel(None);
    drop(stale_sender);
    service
        .result_watchers
        .lock()
        .await
        .insert(job_id, stale_watcher);

    let error = service
        .admin_reload_and_restart_task(job_id)
        .await
        .expect_err("an unreadable pending job should fail to reload");

    assert_eq!(
        error.downcast_ref::<RpcErrorCode>(),
        Some(&RpcErrorCode::Unavailable)
    );
    assert!(!service.has_live_task(job_id));
    assert!(!service.result_watchers.lock().await.contains_key(&job_id));
}

#[tokio::test]
async fn reload_completed_during_load_is_tagged_already_completed() {
    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    let job_id = mk_job_id(31);
    let job = mk_job(31);
    let result = mk_result(31, ContractCallOutcome::Success);
    let result_reads = Arc::new(AtomicUsize::new(0));

    store
        .expect_get_settlement_job()
        .times(2)
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .returning(move |_| Ok(Some(job.clone())));
    store
        .expect_get_settlement_job_result()
        .times(2)
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .returning({
            let result_reads = result_reads.clone();
            move |_| {
                if result_reads.fetch_add(1, Ordering::SeqCst) == 0 {
                    Ok(None)
                } else {
                    Ok(Some(result.clone()))
                }
            }
        });

    let service = mk_service(Arc::new(store)).await;
    let error = service
        .admin_reload_and_restart_task(job_id)
        .await
        .expect_err("a job that completed during reload should be refused");

    assert_eq!(
        error.downcast_ref::<RpcErrorCode>(),
        Some(&RpcErrorCode::AlreadyCompleted)
    );
    assert!(!service.has_live_task(job_id));
    assert!(!service.result_watchers.lock().await.contains_key(&job_id));
}

#[tokio::test]
async fn admin_force_remove_job_result_refuses_while_task_is_live() {
    let mut store = MockStateStore::new();
    let job_id = mk_job_id(70);
    store
        .expect_list_settlement_job_ids()
        .once()
        .return_once(move || Ok(vec![job_id]));
    expect_pending_job_load(&mut store, job_id, 70);
    store
        .expect_admin_force_remove_settlement_job_result()
        .never();

    let cancellation_token = CancellationToken::new();
    let (provider, request_count) = mk_parked_provider();
    let service = SettlementService::start(
        SettlementServiceConfig::default(),
        Arc::new(SettlementTransactionConfig::default()),
        Arc::new(provider),
        Arc::new(store),
        cancellation_token.clone(),
    )
    .await
    .expect("settlement service should start")
    .0;
    wait_until_l1_request_is_parked(&request_count).await;

    let live_control = service
        .task_controls
        .lock()
        .expect("settlement task_controls lock poisoned")
        .get(&job_id)
        .cloned()
        .expect("the parked task must have a registered control");
    live_control
        .try_send(TaskAdminCommand::ReloadAndRestart)
        .expect("the parked task's control channel must still be open");

    let error = service
        .admin_force_remove_settlement_job_result(job_id)
        .await
        .expect_err("force-remove must refuse a job with a live task");

    assert_eq!(
        error.downcast_ref::<RpcErrorCode>(),
        Some(&RpcErrorCode::TaskStillLive)
    );
    cancellation_token.cancel();
}

#[tokio::test]
async fn admin_force_remove_job_result_respawns_task() {
    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    let job_id = mk_job_id(71);
    store
        .expect_admin_force_remove_settlement_job_result()
        .once()
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .return_once(|_| Ok(()));
    expect_pending_job_load(&mut store, job_id, 71);

    let cancellation_token = CancellationToken::new();
    let (provider, request_count) = mk_parked_provider();
    let service = SettlementService::start(
        SettlementServiceConfig::default(),
        Arc::new(SettlementTransactionConfig::default()),
        Arc::new(provider),
        Arc::new(store),
        cancellation_token.clone(),
    )
    .await
    .expect("settlement service should start")
    .0;

    let (_old_sender, old_watcher) =
        watch::channel(Some(mk_result(71, ContractCallOutcome::Success)));
    service
        .result_watchers
        .lock()
        .await
        .insert(job_id, old_watcher.clone());

    service
        .admin_force_remove_settlement_job_result(job_id)
        .await
        .expect("force-remove should respawn the pending job");
    wait_until_l1_request_is_parked(&request_count).await;

    assert!(service
        .task_controls
        .lock()
        .expect("settlement task_controls lock poisoned")
        .contains_key(&job_id));
    let result_watchers = service.result_watchers.lock().await;
    let fresh_watcher = result_watchers
        .get(&job_id)
        .expect("the respawned task must have a watcher");
    assert!(!fresh_watcher.same_channel(&old_watcher));
    assert!(fresh_watcher.borrow().is_none());
    drop(result_watchers);
    cancellation_token.cancel();
}

#[tokio::test]
async fn concurrent_force_removes_are_serialized() {
    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    let job_id = mk_job_id(72);
    let delete_count = Arc::new(AtomicUsize::new(0));
    store
        .expect_admin_force_remove_settlement_job_result()
        .once()
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .return_once({
            let delete_count = delete_count.clone();
            move |_| {
                delete_count.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        });
    expect_pending_job_load(&mut store, job_id, 72);

    let cancellation_token = CancellationToken::new();
    let (provider, request_count) = mk_parked_provider();
    let service = SettlementService::start(
        SettlementServiceConfig::default(),
        Arc::new(SettlementTransactionConfig::default()),
        Arc::new(provider),
        Arc::new(store),
        cancellation_token.clone(),
    )
    .await
    .expect("settlement service should start")
    .0;

    let (first, second) = tokio::join!(
        service.admin_force_remove_settlement_job_result(job_id),
        service.admin_force_remove_settlement_job_result(job_id),
    );
    let loser = match (first, second) {
        (Ok(()), Err(error)) | (Err(error), Ok(())) => error,
        (Ok(()), Ok(())) => panic!("the admin lock must prevent a second successful respawn"),
        (Err(first), Err(second)) => {
            panic!("one force-remove should succeed, got {first:?} and {second:?}")
        }
    };

    assert!(matches!(
        loser.downcast_ref::<RpcErrorCode>(),
        Some(RpcErrorCode::TaskStillLive | RpcErrorCode::NotCompleted)
    ));
    assert_eq!(delete_count.load(Ordering::SeqCst), 1);
    wait_until_l1_request_is_parked(&request_count).await;
    cancellation_token.cancel();
}

#[tokio::test]
async fn force_remove_load_failure_leaves_clean_aborted_state() {
    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    let job_id = mk_job_id(73);
    store
        .expect_admin_force_remove_settlement_job_result()
        .once()
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .return_once(|_| Ok(()));
    store
        .expect_get_settlement_job()
        .once()
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .return_once(|_| {
            Err(StorageError::UnprocessedAction(
                "settlement job row became unreadable".to_owned(),
            ))
        });

    let service = mk_service(Arc::new(store)).await;
    let (_old_sender, old_watcher) =
        watch::channel(Some(mk_result(73, ContractCallOutcome::Success)));
    service
        .result_watchers
        .lock()
        .await
        .insert(job_id, old_watcher);

    let error = service
        .admin_force_remove_settlement_job_result(job_id)
        .await
        .expect_err("an unreadable pending job should fail to reload");

    assert_eq!(
        error.downcast_ref::<RpcErrorCode>(),
        Some(&RpcErrorCode::Unavailable)
    );
    assert!(!service
        .task_controls
        .lock()
        .expect("settlement task_controls lock poisoned")
        .contains_key(&job_id));
    assert!(!service.result_watchers.lock().await.contains_key(&job_id));
}

/// A fully specified admin attempt.
fn mk_new_attempt(seed: u8, tx_hash: SettlementTxHash) -> NewSettlementAttempt {
    NewSettlementAttempt {
        tx_hash,
        sender_wallet: Some(agglayer_types::Address::from([seed; 20])),
        nonce: Some(Nonce(seed as u64)),
        submission_time: Some(std::time::SystemTime::UNIX_EPOCH),
        max_fee_per_gas: Some(30),
        max_priority_fee_per_gas: Some(3),
    }
}

/// What [`mk_new_attempt`] resolves to when its identity matches L1.
fn mk_resolved_attempt(seed: u8, tx_hash: SettlementTxHash) -> SettlementAttempt {
    SettlementAttempt {
        sender_wallet: agglayer_types::Address::from([seed; 20]),
        nonce: Nonce(seed as u64),
        hash: tx_hash,
        submission_time: std::time::SystemTime::UNIX_EPOCH,
        max_fee_per_gas: 30,
        max_priority_fee_per_gas: 3,
    }
}

fn mk_l1_transaction(
    sender: agglayer_types::Address,
    nonce: u64,
    max_fee_per_gas: u128,
    max_priority_fee_per_gas: u128,
) -> alloy::rpc::types::Transaction {
    let transaction =
        alloy::consensus::TxEnvelope::Eip1559(alloy::consensus::Signed::new_unhashed(
            alloy::consensus::TxEip1559 {
                chain_id: 1,
                nonce,
                gas_limit: 21_000,
                max_fee_per_gas,
                max_priority_fee_per_gas,
                ..Default::default()
            },
            alloy::primitives::Signature::new(U256::from(1), U256::from(2), false),
        ));

    alloy::rpc::types::Transaction {
        inner: alloy::consensus::transaction::Recovered::new_unchecked(transaction, sender.into()),
        block_hash: None,
        block_number: None,
        transaction_index: None,
        effective_gas_price: Some(max_fee_per_gas),
    }
}

fn mk_provider_with_tx_response(
    transaction: Option<alloy::rpc::types::Transaction>,
) -> impl Provider + WalletProvider + 'static {
    let asserter = Asserter::new();
    asserter.push_success(&transaction);
    ProviderBuilder::new()
        .wallet(EthereumWallet::from(
            PrivateKeySigner::from_slice(&[0x11; 32]).expect("valid test signing key"),
        ))
        .connect_mocked_client(asserter)
}

const UNKNOWN_TX_WARNING: &str = "Settlement transaction is not known to the L1 RPC; trusting \
                                  explicitly provided sender wallet and nonce";

struct ExpectedWarnCountingSubscriber {
    warn_events: Arc<AtomicUsize>,
}

impl tracing::Subscriber for ExpectedWarnCountingSubscriber {
    fn enabled(&self, _metadata: &tracing::Metadata<'_>) -> bool {
        true
    }

    fn new_span(&self, _span: &tracing::span::Attributes<'_>) -> tracing::span::Id {
        tracing::span::Id::from_u64(1)
    }

    fn record(&self, _span: &tracing::span::Id, _values: &tracing::span::Record<'_>) {}

    fn record_follows_from(&self, _span: &tracing::span::Id, _follows: &tracing::span::Id) {}

    fn event(&self, event: &tracing::Event<'_>) {
        if *event.metadata().level() != tracing::Level::WARN
            || event.metadata().target() != "agglayer_settlement_service::settlement_service"
        {
            return;
        }

        struct ExpectedMessageVisitor(bool);

        impl tracing::field::Visit for ExpectedMessageVisitor {
            fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
                if field.name() == "message" && format!("{value:?}") == UNKNOWN_TX_WARNING {
                    self.0 = true;
                }
            }

            fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
                if field.name() == "message" && value == UNKNOWN_TX_WARNING {
                    self.0 = true;
                }
            }
        }

        let mut visitor = ExpectedMessageVisitor(false);
        event.record(&mut visitor);
        if visitor.0 {
            self.warn_events.fetch_add(1, Ordering::SeqCst);
        }
    }

    fn enter(&self, _span: &tracing::span::Id) {}

    fn exit(&self, _span: &tracing::span::Id) {}
}

#[tokio::test]
async fn admin_insert_attempt_returns_assigned_number_and_reports_absent_task() {
    let transaction = mk_l1_transaction(agglayer_types::Address::from([30; 20]), 30, 30, 3);
    let tx_hash =
        SettlementTxHash::from(alloy::network::TransactionResponse::tx_hash(&transaction));
    let provider = mk_provider_with_tx_response(Some(transaction));

    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    let job_id = mk_job_id(30);
    let expected_attempt = mk_resolved_attempt(30, tx_hash);

    store
        .expect_admin_insert_settlement_attempt()
        .once()
        .withf(
            move |requested_job_id, requested_attempt, edit_even_if_completed| {
                requested_job_id == &job_id
                    && requested_attempt == &expected_attempt
                    && *edit_even_if_completed == EditEvenIfCompleted::No
            },
        )
        .return_once(|_, _, _| Ok(3));

    let service = SettlementService::start(
        SettlementServiceConfig::default(),
        Arc::new(SettlementTransactionConfig::default()),
        Arc::new(provider),
        Arc::new(store),
        CancellationToken::new(),
    )
    .await
    .expect("settlement service should start")
    .0;

    let result = service
        .admin_insert_settlement_attempt(
            job_id,
            mk_new_attempt(30, tx_hash),
            EditEvenIfCompleted::No,
        )
        .await
        .expect("admin insert should succeed");

    assert_eq!(result, (3, LiveTaskNotification::Absent));
}

#[tokio::test]
async fn admin_insert_attempt_trusts_explicit_identity_for_unknown_tx_and_warns() {
    let tx_hash = SettlementTxHash::new(Digest::from([0x40; 32]));
    let expected_attempt = mk_resolved_attempt(40, tx_hash);
    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    let job_id = mk_job_id(40);
    store
        .expect_admin_insert_settlement_attempt()
        .once()
        .withf(move |requested_job_id, attempt, edit_even_if_completed| {
            requested_job_id == &job_id
                && attempt == &expected_attempt
                && *edit_even_if_completed == EditEvenIfCompleted::No
        })
        .return_once(|_, _, _| Ok(4));

    let service = SettlementService::start(
        SettlementServiceConfig::default(),
        Arc::new(SettlementTransactionConfig::default()),
        Arc::new(mk_provider_with_tx_response(None)),
        Arc::new(store),
        CancellationToken::new(),
    )
    .await
    .expect("settlement service should start")
    .0;

    let warn_events = Arc::new(AtomicUsize::new(0));
    // Thread-local default: `#[tokio::test]` uses a current-thread runtime, so
    // the warning from this call is isolated from concurrently running tests.
    let _guard = tracing::subscriber::set_default(ExpectedWarnCountingSubscriber {
        warn_events: warn_events.clone(),
    });
    let result = service
        .admin_insert_settlement_attempt(
            job_id,
            mk_new_attempt(40, tx_hash),
            EditEvenIfCompleted::No,
        )
        .await
        .expect("an unknown transaction with explicit identity should be accepted");

    assert_eq!(result, (4, LiveTaskNotification::Absent));
    assert_eq!(warn_events.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn admin_insert_attempt_resolves_missing_fields_from_l1() {
    let sender = agglayer_types::Address::from([0x41; 20]);
    let nonce = 7;
    let max_fee_per_gas = 2_000_000_000;
    let max_priority_fee_per_gas = 1_000_000_000;
    let transaction = mk_l1_transaction(sender, nonce, max_fee_per_gas, max_priority_fee_per_gas);
    let tx_hash =
        SettlementTxHash::from(alloy::network::TransactionResponse::tx_hash(&transaction));
    let provider = mk_provider_with_tx_response(Some(transaction));

    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    let job_id = mk_job_id(31);
    store
        .expect_admin_insert_settlement_attempt()
        .once()
        .withf(move |requested_job_id, attempt, edit_even_if_completed| {
            requested_job_id == &job_id
                && attempt.sender_wallet == sender
                && attempt.nonce == Nonce(nonce)
                && attempt.hash == tx_hash
                && attempt.max_fee_per_gas == max_fee_per_gas
                && attempt.max_priority_fee_per_gas == max_priority_fee_per_gas
                && *edit_even_if_completed == EditEvenIfCompleted::No
        })
        .return_once(|_, _, _| Ok(0));

    let service = SettlementService::start(
        SettlementServiceConfig::default(),
        Arc::new(SettlementTransactionConfig::default()),
        Arc::new(provider),
        Arc::new(store),
        CancellationToken::new(),
    )
    .await
    .expect("settlement service should start")
    .0;

    let result = service
        .admin_insert_settlement_attempt(
            job_id,
            NewSettlementAttempt {
                tx_hash,
                sender_wallet: None,
                nonce: None,
                submission_time: None,
                max_fee_per_gas: None,
                max_priority_fee_per_gas: None,
            },
            EditEvenIfCompleted::No,
        )
        .await
        .expect("admin insert should resolve the attempt from L1");

    assert_eq!(result, (0, LiveTaskNotification::Absent));
}

#[tokio::test]
async fn admin_insert_attempt_rejects_explicit_sender_mismatch_with_l1() {
    let l1_sender_wallet = agglayer_types::Address::from([0x51; 20]);
    let provided_sender_wallet = agglayer_types::Address::from([0x52; 20]);
    let nonce = 9;
    let transaction = mk_l1_transaction(l1_sender_wallet, nonce, 30, 3);
    let tx_hash =
        SettlementTxHash::from(alloy::network::TransactionResponse::tx_hash(&transaction));

    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    store.expect_admin_insert_settlement_attempt().never();
    let service = SettlementService::start(
        SettlementServiceConfig::default(),
        Arc::new(SettlementTransactionConfig::default()),
        Arc::new(mk_provider_with_tx_response(Some(transaction))),
        Arc::new(store),
        CancellationToken::new(),
    )
    .await
    .expect("settlement service should start")
    .0;

    let error = service
        .admin_insert_settlement_attempt(
            mk_job_id(41),
            NewSettlementAttempt {
                tx_hash,
                sender_wallet: Some(provided_sender_wallet),
                nonce: Some(Nonce(nonce)),
                submission_time: None,
                max_fee_per_gas: None,
                max_priority_fee_per_gas: None,
            },
            EditEvenIfCompleted::No,
        )
        .await
        .expect_err("an explicit sender that disagrees with L1 should be rejected");

    assert_eq!(
        error.downcast_ref::<RpcErrorCode>(),
        Some(&RpcErrorCode::InvalidParams)
    );
    assert_eq!(
        error.chain().map(ToString::to_string).collect::<Vec<_>>(),
        vec![
            "invalid params".to_owned(),
            format!(
                "Explicit sender wallet {provided_sender_wallet} does not match L1 sender wallet \
                 {l1_sender_wallet} for settlement transaction {tx_hash}"
            ),
        ]
    );
}

#[tokio::test]
async fn admin_insert_attempt_rejects_explicit_nonce_mismatch_with_l1() {
    let sender_wallet = agglayer_types::Address::from([0x53; 20]);
    let l1_nonce = Nonce(10);
    let provided_nonce = Nonce(11);
    let transaction = mk_l1_transaction(sender_wallet, l1_nonce.0, 30, 3);
    let tx_hash =
        SettlementTxHash::from(alloy::network::TransactionResponse::tx_hash(&transaction));

    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    store.expect_admin_insert_settlement_attempt().never();
    let service = SettlementService::start(
        SettlementServiceConfig::default(),
        Arc::new(SettlementTransactionConfig::default()),
        Arc::new(mk_provider_with_tx_response(Some(transaction))),
        Arc::new(store),
        CancellationToken::new(),
    )
    .await
    .expect("settlement service should start")
    .0;

    let error = service
        .admin_insert_settlement_attempt(
            mk_job_id(42),
            NewSettlementAttempt {
                tx_hash,
                sender_wallet: Some(sender_wallet),
                nonce: Some(provided_nonce),
                submission_time: None,
                max_fee_per_gas: None,
                max_priority_fee_per_gas: None,
            },
            EditEvenIfCompleted::No,
        )
        .await
        .expect_err("an explicit nonce that disagrees with L1 should be rejected");

    assert_eq!(
        error.downcast_ref::<RpcErrorCode>(),
        Some(&RpcErrorCode::InvalidParams)
    );
    assert_eq!(
        error.chain().map(ToString::to_string).collect::<Vec<_>>(),
        vec![
            "invalid params".to_owned(),
            format!(
                "Explicit nonce {provided_nonce} does not match L1 nonce {l1_nonce} for \
                 settlement transaction {tx_hash}"
            ),
        ]
    );
}

#[tokio::test]
async fn admin_insert_attempt_fails_for_unknown_tx_when_fields_missing() {
    for (sender_wallet, nonce) in [
        (None, None),
        (Some(agglayer_types::Address::from([0x42; 20])), None),
        (None, Some(Nonce(42))),
    ] {
        let mut store = MockStateStore::new();
        expect_empty_startup_recovery(&mut store);
        store.expect_admin_insert_settlement_attempt().never();

        let service = SettlementService::start(
            SettlementServiceConfig::default(),
            Arc::new(SettlementTransactionConfig::default()),
            Arc::new(mk_provider_with_tx_response(None)),
            Arc::new(store),
            CancellationToken::new(),
        )
        .await
        .expect("settlement service should start")
        .0;

        let error = service
            .admin_insert_settlement_attempt(
                mk_job_id(32),
                NewSettlementAttempt {
                    tx_hash: SettlementTxHash::new(Digest::from([0x42; 32])),
                    sender_wallet,
                    nonce,
                    submission_time: None,
                    max_fee_per_gas: None,
                    max_priority_fee_per_gas: None,
                },
                EditEvenIfCompleted::No,
            )
            .await
            .expect_err("unknown transaction with an incomplete identity should be rejected");

        assert_eq!(
            error.downcast_ref::<RpcErrorCode>(),
            Some(&RpcErrorCode::NotFound)
        );
        assert!(format!("{error:#}").contains("not known to the L1 RPC"));
    }
}

#[tokio::test]
async fn admin_insert_attempt_completed_job_error_is_tagged_without_notification() {
    let transaction = mk_l1_transaction(agglayer_types::Address::from([33; 20]), 33, 30, 3);
    let tx_hash =
        SettlementTxHash::from(alloy::network::TransactionResponse::tx_hash(&transaction));
    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    let job_id = mk_job_id(33);

    store
        .expect_admin_insert_settlement_attempt()
        .once()
        .withf(move |requested_job_id, _, edit_even_if_completed| {
            requested_job_id == &job_id && *edit_even_if_completed == EditEvenIfCompleted::No
        })
        .return_once(move |_, _, _| Err(StorageError::SettlementJobAlreadyCompleted(job_id)));

    let service = SettlementService::start(
        SettlementServiceConfig::default(),
        Arc::new(SettlementTransactionConfig::default()),
        Arc::new(mk_provider_with_tx_response(Some(transaction))),
        Arc::new(store),
        CancellationToken::new(),
    )
    .await
    .expect("settlement service should start")
    .0;
    let (task_control_handle, mut task_control) =
        TaskControlHandle::new(&service.cancellation_token);
    service
        .task_controls
        .lock()
        .expect("settlement task_controls lock poisoned")
        .insert(job_id, task_control_handle);

    let error = service
        .admin_insert_settlement_attempt(
            job_id,
            mk_new_attempt(33, tx_hash),
            EditEvenIfCompleted::No,
        )
        .await
        .expect_err("a completed job should reject an unforced insert");

    assert_eq!(
        error.downcast_ref::<RpcErrorCode>(),
        Some(&RpcErrorCode::AlreadyCompleted)
    );
    assert!(
        task_control.try_recv_admin_command().is_none(),
        "a failed storage insert must not trigger a task reload"
    );
}

#[tokio::test]
async fn admin_mutations_forward_the_force_flag_to_storage() {
    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    let mark_without_force_job = mk_job_id(30);
    let mark_with_force_job = mk_job_id(31);
    let remove_without_force_job = mk_job_id(32);
    let remove_with_force_job = mk_job_id(33);

    store
        .expect_admin_override_settlement_attempt_result()
        .once()
        .withf(
            move |job_id, attempt_number, result, edit_even_if_completed| {
                job_id == &mark_without_force_job
                    && *attempt_number == 0
                    && matches!(
                        result,
                        SettlementAttemptResult::ClientError(client_error)
                            if client_error.kind == ClientErrorType::AbandonedByAdmin
                                && client_error.message.contains("not final")
                    )
                    && *edit_even_if_completed == EditEvenIfCompleted::No
            },
        )
        .return_once(|_, _, _, _| Ok(()));
    store
        .expect_admin_override_settlement_attempt_result()
        .once()
        .withf(
            move |job_id, attempt_number, result, edit_even_if_completed| {
                job_id == &mark_with_force_job
                    && *attempt_number == 1
                    && matches!(
                        result,
                        SettlementAttemptResult::ClientError(client_error)
                            if client_error.kind == ClientErrorType::AbandonedByAdmin
                                && client_error.message.contains("definitely final")
                    )
                    && *edit_even_if_completed == EditEvenIfCompleted::Yes
            },
        )
        .return_once(|_, _, _, _| Ok(()));
    store
        .expect_admin_remove_settlement_attempt_result()
        .once()
        .withf(move |job_id, attempt_number, edit_even_if_completed| {
            job_id == &remove_without_force_job
                && *attempt_number == 2
                && *edit_even_if_completed == EditEvenIfCompleted::No
        })
        .return_once(|_, _, _| Ok(()));
    store
        .expect_admin_remove_settlement_attempt_result()
        .once()
        .withf(move |job_id, attempt_number, edit_even_if_completed| {
            job_id == &remove_with_force_job
                && *attempt_number == 3
                && *edit_even_if_completed == EditEvenIfCompleted::Yes
        })
        .return_once(|_, _, _| Ok(()));

    let service = mk_service(Arc::new(store)).await;

    assert_eq!(
        service
            .admin_mark_attempt_definitely_failed(
                mark_without_force_job,
                0,
                "not final",
                EditEvenIfCompleted::No,
            )
            .await
            .expect("unforced mark should succeed"),
        LiveTaskNotification::Absent
    );
    assert_eq!(
        service
            .admin_mark_attempt_definitely_failed(
                mark_with_force_job,
                1,
                "definitely final",
                EditEvenIfCompleted::Yes,
            )
            .await
            .expect("forced mark should succeed"),
        LiveTaskNotification::Absent
    );
    assert_eq!(
        service
            .admin_remove_attempt_result(remove_without_force_job, 2, EditEvenIfCompleted::No,)
            .await
            .expect("unforced removal should succeed"),
        LiveTaskNotification::Absent
    );
    assert_eq!(
        service
            .admin_remove_attempt_result(remove_with_force_job, 3, EditEvenIfCompleted::Yes)
            .await
            .expect("forced removal should succeed"),
        LiveTaskNotification::Absent
    );
}

#[tokio::test]
async fn admin_mutation_storage_error_propagates_without_task_notification() {
    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    let job_id = mk_job_id(34);

    store
        .expect_admin_override_settlement_attempt_result()
        .once()
        .withf(
            move |requested_job_id, attempt_number, _, edit_even_if_completed| {
                requested_job_id == &job_id
                    && *attempt_number == 0
                    && *edit_even_if_completed == EditEvenIfCompleted::No
            },
        )
        .return_once(move |_, _, _, _| Err(StorageError::SettlementJobAlreadyCompleted(job_id)));

    let service = mk_service(Arc::new(store)).await;
    let (task_control_handle, mut task_control) =
        TaskControlHandle::new(&service.cancellation_token);
    service
        .task_controls
        .lock()
        .expect("settlement task_controls lock poisoned")
        .insert(job_id, task_control_handle);

    let error = service
        .admin_mark_attempt_definitely_failed(job_id, 0, "stuck", EditEvenIfCompleted::No)
        .await
        .expect_err("a storage refusal should propagate");

    assert_eq!(
        error.downcast_ref::<RpcErrorCode>(),
        Some(&RpcErrorCode::AlreadyCompleted)
    );
    assert!(format!("{error:#}").contains("already has a terminal result"));
    assert!(
        task_control.try_recv_admin_command().is_none(),
        "a failed storage edit must not trigger a task reload"
    );
}

#[tokio::test]
async fn admin_mutation_reports_absent_live_task() {
    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    let job_id = mk_job_id(35);

    store
        .expect_admin_remove_settlement_attempt_result()
        .once()
        .withf(
            move |requested_job_id, attempt_number, edit_even_if_completed| {
                requested_job_id == &job_id
                    && *attempt_number == 4
                    && *edit_even_if_completed == EditEvenIfCompleted::No
            },
        )
        .return_once(|_, _, _| Ok(()));

    let service = mk_service(Arc::new(store)).await;
    let live_task = service
        .admin_remove_attempt_result(job_id, 4, EditEvenIfCompleted::No)
        .await
        .expect("admin removal should succeed");

    assert_eq!(live_task, LiveTaskNotification::Absent);
}

#[tokio::test]
async fn admin_mutation_queues_reload_for_parked_live_task() {
    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    let job_id = mk_job_id(36);

    store
        .expect_admin_override_settlement_attempt_result()
        .once()
        .withf(
            move |requested_job_id, attempt_number, _, edit_even_if_completed| {
                requested_job_id == &job_id
                    && *attempt_number == 5
                    && *edit_even_if_completed == EditEvenIfCompleted::No
            },
        )
        .return_once(|_, _, _, _| Ok(()));

    let service = mk_service(Arc::new(store)).await;
    let (task_control_handle, mut task_control) =
        TaskControlHandle::new(&service.cancellation_token);
    service
        .task_controls
        .lock()
        .expect("settlement task_controls lock poisoned")
        .insert(job_id, task_control_handle);

    let live_task = service
        .admin_mark_attempt_definitely_failed(job_id, 5, "nonce burned", EditEvenIfCompleted::No)
        .await
        .expect("admin mark should succeed");

    assert_eq!(live_task, LiveTaskNotification::Queued);
    assert_eq!(
        serde_json::to_value(live_task).expect("live-task notification should serialize"),
        serde_json::json!("queued")
    );
    assert!(matches!(
        task_control.try_recv_admin_command(),
        Some(TaskAdminCommand::ReloadAndRestart)
    ));
    assert!(task_control.try_recv_admin_command().is_none());
}

#[tokio::test]
async fn admin_mutation_reports_closed_and_full_notification_channels() {
    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    let closed_job_id = mk_job_id(37);
    let full_job_id = mk_job_id(38);

    store
        .expect_admin_remove_settlement_attempt_result()
        .once()
        .withf(
            move |requested_job_id, attempt_number, edit_even_if_completed| {
                requested_job_id == &closed_job_id
                    && *attempt_number == 6
                    && *edit_even_if_completed == EditEvenIfCompleted::No
            },
        )
        .return_once(|_, _, _| Ok(()));
    store
        .expect_admin_remove_settlement_attempt_result()
        .once()
        .withf(
            move |requested_job_id, attempt_number, edit_even_if_completed| {
                requested_job_id == &full_job_id
                    && *attempt_number == 7
                    && *edit_even_if_completed == EditEvenIfCompleted::No
            },
        )
        .return_once(|_, _, _| Ok(()));

    let service = mk_service(Arc::new(store)).await;

    let (closed_handle, closed_control) = TaskControlHandle::new(&service.cancellation_token);
    drop(closed_control);
    service
        .task_controls
        .lock()
        .expect("settlement task_controls lock poisoned")
        .insert(closed_job_id, closed_handle);

    let (full_handle, _full_control) = TaskControlHandle::new(&service.cancellation_token);
    while full_handle
        .try_send(TaskAdminCommand::ReloadAndRestart)
        .is_ok()
    {}
    service
        .task_controls
        .lock()
        .expect("settlement task_controls lock poisoned")
        .insert(full_job_id, full_handle);

    assert_eq!(
        service
            .admin_remove_attempt_result(closed_job_id, 6, EditEvenIfCompleted::No)
            .await
            .expect("the persisted edit should survive a closed notification channel"),
        LiveTaskNotification::NotifyFailed
    );
    assert_eq!(
        service
            .admin_remove_attempt_result(full_job_id, 7, EditEvenIfCompleted::No)
            .await
            .expect("the persisted edit should survive a full notification channel"),
        LiveTaskNotification::NotifyFailed
    );
}

mod same_wallet_nonce_race;
