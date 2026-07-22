use std::sync::{Arc, Mutex};

use agglayer_storage::tests::mocks::MockStateStore;
use agglayer_types::{
    CertificateId, ContractCallOutcome, ContractCallResult, Digest, Nonce, RpcErrorCode,
    SettlementAttemptNumber, SettlementJob, SettlementJobId, SettlementJobResult, SettlementTxHash,
    B256, U256,
};
use alloy::{
    network::EthereumWallet,
    primitives::U64,
    providers::{mock::Asserter, ProviderBuilder},
    signers::local::PrivateKeySigner,
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

    assert!(service.task_controls.lock().await.is_empty());
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
        StoredSettlementJob::Completed(_, _) => panic!("initial load should be pending"),
    };

    let mut result_receiver = service
        .spawn_settlement_task(job_id, task, task_control_handle)
        .await;

    result_receiver
        .changed()
        .await
        .expect("reload should publish the stored terminal result");

    assert_eq!(result_receiver.borrow().as_ref(), Some(&completed_result));
    assert!(service.task_controls.lock().await.is_empty());
    assert!(service.result_watchers.lock().await.contains_key(&job_id));
}

#[tokio::test]
async fn request_new_settlement_records_certificate_link_before_job() {
    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    let certificate_id = CertificateId::new(Digest::from([7; 32]));
    let job = mk_job(7);
    // `create` resolves the gas limit via estimateGas (mock returns 200_000).
    let mut expected_job = job.clone();
    expected_job.gas_limit = 200_000;
    let recorded_job_id = Arc::new(Mutex::new(None));
    let ordering = Arc::new(Mutex::new(Vec::new()));

    store
        .expect_insert_certificate_settlement_job_id()
        .once()
        .withf(move |recorded_certificate_id, _| recorded_certificate_id == &certificate_id)
        .return_once({
            let ordering = ordering.clone();
            let recorded_job_id = recorded_job_id.clone();
            move |_, settlement_job_id| {
                ordering.lock().unwrap().push("write_link");
                *recorded_job_id.lock().unwrap() = Some(*settlement_job_id);
                Ok(())
            }
        });

    store
        .expect_insert_settlement_job()
        .once()
        .withf(move |_, recorded_job| recorded_job == &expected_job)
        .return_once({
            let ordering = ordering.clone();
            let recorded_job_id = recorded_job_id.clone();
            move |settlement_job_id, _| {
                ordering.lock().unwrap().push("write_job");
                assert_eq!(*recorded_job_id.lock().unwrap(), Some(*settlement_job_id));
                Ok(())
            }
        });

    // `create` runs `estimateGas` before persisting; answer it above the
    // ceiling so the stored limit is unchanged. Live token for estimation,
    // then cancel to stop the spawned task.
    let cancellation_token = CancellationToken::new();
    let service = SettlementService::start(
        SettlementServiceConfig::default(),
        Arc::new(SettlementTransactionConfig::default()),
        Arc::new(mk_provider_with_gas_estimate(200_000)),
        Arc::new(store),
        cancellation_token.clone(),
    )
    .await
    .expect("settlement service should start");

    let watcher = service
        .request_new_settlement(Some(certificate_id), job)
        .await
        .expect("settlement request should be accepted");
    cancellation_token.cancel();

    assert_eq!(*recorded_job_id.lock().unwrap(), Some(watcher.job_id()));
    assert_eq!(
        ordering.lock().unwrap().as_slice(),
        ["write_link", "write_job"]
    );
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
async fn admin_reload_pending_job_without_task_is_tagged_no_live_task() {
    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    let job_id = mk_job_id(25);
    let job = mk_job(25);

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
        .admin_reload_and_restart_task(job_id)
        .await
        .expect_err("reload on pending job without a task should fail");

    assert_eq!(
        error.downcast_ref::<RpcErrorCode>(),
        Some(&RpcErrorCode::NoLiveTask)
    );
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
        .await
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
async fn admin_reload_with_closed_admin_channel_is_classified_via_storage() {
    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    let job_id = mk_job_id(27);
    let job = mk_job(27);

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
    let (task_control_handle, task_control) = TaskControlHandle::new(&service.cancellation_token);
    // Drop the receiver side so the admin channel is closed rather than full,
    // simulating the task completing/dying between the lookup and the send.
    drop(task_control);
    service
        .task_controls
        .lock()
        .await
        .insert(job_id, task_control_handle);

    let error = service
        .admin_reload_and_restart_task(job_id)
        .await
        .expect_err("reload with a closed admin command channel should fail");

    assert_eq!(
        error.downcast_ref::<RpcErrorCode>(),
        Some(&RpcErrorCode::NoLiveTask)
    );
}

mod same_wallet_nonce_race;
