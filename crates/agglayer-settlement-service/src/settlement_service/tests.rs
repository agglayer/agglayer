use std::sync::{Arc, Mutex};

use agglayer_storage::{
    error::Error as StorageError, stores::EditEvenIfCompleted, tests::mocks::MockStateStore,
};
use agglayer_types::{
    CertificateId, ClientErrorType, ContractCallOutcome, ContractCallResult, Digest, Nonce,
    RpcErrorCode, SettlementAttemptNumber, SettlementAttemptResult, SettlementJob, SettlementJobId,
    SettlementJobResult, SettlementTxHash, B256, U256,
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
    .expect("settlement service should start")
    .0;

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
        .await
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
async fn admin_mutation_notifies_parked_live_task() {
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
        .await
        .insert(job_id, task_control_handle);

    let live_task = service
        .admin_mark_attempt_definitely_failed(job_id, 5, "nonce burned", EditEvenIfCompleted::No)
        .await
        .expect("admin mark should succeed");

    assert_eq!(live_task, LiveTaskNotification::Notified);
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
        .await
        .insert(closed_job_id, closed_handle);

    let (full_handle, _full_control) = TaskControlHandle::new(&service.cancellation_token);
    while full_handle
        .try_send(TaskAdminCommand::ReloadAndRestart)
        .is_ok()
    {}
    service
        .task_controls
        .lock()
        .await
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
