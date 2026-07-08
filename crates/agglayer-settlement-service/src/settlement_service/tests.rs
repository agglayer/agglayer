use std::sync::{Arc, Mutex};

use agglayer_storage::tests::mocks::MockStateStore;
use agglayer_types::{
    CertificateId, ClientErrorType, ContractCallOutcome, ContractCallResult, Digest, Nonce,
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

    assert!(error.to_string().contains("exists without a running task"));
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

/// A fully-specified admin attempt: no field requires an L1 lookup.
fn mk_new_attempt(seed: u8) -> NewSettlementAttempt {
    NewSettlementAttempt {
        tx_hash: SettlementTxHash::new(Digest::from([seed.wrapping_add(1); 32])),
        sender_wallet: Some(agglayer_types::Address::from([seed; 20])),
        nonce: Some(Nonce(seed as u64)),
        submission_time: Some(std::time::SystemTime::UNIX_EPOCH),
        max_fee_per_gas: Some(30),
        max_priority_fee_per_gas: Some(3),
    }
}

/// What [`mk_new_attempt`] resolves to without touching L1.
fn mk_resolved_attempt(seed: u8) -> SettlementAttempt {
    SettlementAttempt {
        sender_wallet: agglayer_types::Address::from([seed; 20]),
        nonce: Nonce(seed as u64),
        hash: SettlementTxHash::new(Digest::from([seed.wrapping_add(1); 32])),
        submission_time: std::time::SystemTime::UNIX_EPOCH,
        max_fee_per_gas: 30,
        max_priority_fee_per_gas: 3,
    }
}

#[tokio::test]
async fn admin_insert_attempt_returns_assigned_number_and_reports_absent_task() {
    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    let job_id = mk_job_id(30);
    let expected_attempt = mk_resolved_attempt(30);

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
        .return_once(|_, _, _| Ok(4));

    let service = mk_service(Arc::new(store)).await;

    // The provider is a dead endpoint: a fully-specified attempt must
    // resolve without any L1 lookup.
    let (attempt_number, live_task) = service
        .admin_insert_settlement_attempt(job_id, mk_new_attempt(30), EditEvenIfCompleted::No)
        .await
        .expect("admin insert should succeed");

    assert_eq!(attempt_number, 4);
    assert_eq!(live_task, LiveTaskNotification::Absent);
}

#[tokio::test]
async fn admin_insert_attempt_resolves_missing_fields_from_l1() {
    let anvil = alloy::node_bindings::Anvil::new().spawn();
    let sender = anvil.addresses()[0];
    let provider = crate::utils::build_provider(&anvil);

    let tx_request = alloy::rpc::types::TransactionRequest::default()
        .to(anvil.addresses()[1])
        .value(U256::from(1))
        .max_fee_per_gas(2_000_000_000)
        .max_priority_fee_per_gas(1_000_000_000);
    let receipt = provider
        .send_transaction(tx_request)
        .await
        .expect("test transaction should be accepted")
        .get_receipt()
        .await
        .expect("test transaction should be mined");
    let tx_hash = SettlementTxHash::from(Digest::from(receipt.transaction_hash));

    let job_id = mk_job_id(34);
    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    store
        .expect_admin_insert_settlement_attempt()
        .once()
        .withf(move |requested_job_id, attempt, _| {
            requested_job_id == &job_id
                && attempt.sender_wallet == sender.into()
                && attempt.nonce == Nonce(0)
                && attempt.hash == tx_hash
                && attempt.max_fee_per_gas == 2_000_000_000
                && attempt.max_priority_fee_per_gas == 1_000_000_000
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
    .expect("settlement service should start");

    let (attempt_number, live_task) = service
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

    assert_eq!(attempt_number, 0);
    assert_eq!(live_task, LiveTaskNotification::Absent);
}

#[tokio::test]
async fn admin_insert_attempt_fails_for_unknown_tx_when_fields_missing() {
    let anvil = alloy::node_bindings::Anvil::new().spawn();
    let provider = crate::utils::build_provider(&anvil);

    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);

    let service = SettlementService::start(
        SettlementServiceConfig::default(),
        Arc::new(SettlementTransactionConfig::default()),
        Arc::new(provider),
        Arc::new(store),
        CancellationToken::new(),
    )
    .await
    .expect("settlement service should start");

    let result = service
        .admin_insert_settlement_attempt(
            mk_job_id(35),
            NewSettlementAttempt {
                tx_hash: SettlementTxHash::new(Digest::from([0x42; 32])),
                sender_wallet: None,
                nonce: None,
                submission_time: None,
                max_fee_per_gas: None,
                max_priority_fee_per_gas: None,
            },
            EditEvenIfCompleted::No,
        )
        .await;

    let error = result.expect_err("unknown transaction should be rejected");
    assert!(format!("{error:#}").contains("not known to the L1 RPC"));
}

#[tokio::test]
async fn admin_mark_attempt_definitely_failed_notifies_live_task() {
    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    let job_id = mk_job_id(31);

    store
        .expect_admin_override_settlement_attempt_result()
        .once()
        .withf(
            move |requested_job_id, attempt_number, result, edit_even_if_completed| {
                requested_job_id == &job_id
                    && *attempt_number == 2
                    && *edit_even_if_completed == EditEvenIfCompleted::No
                    && matches!(
                        result,
                        SettlementAttemptResult::ClientError(client_error)
                            if client_error.kind == ClientErrorType::AbandonedByAdmin
                                && client_error.message.contains("nonce is burned")
                    )
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
        .admin_mark_attempt_definitely_failed(job_id, 2, "nonce is burned", EditEvenIfCompleted::No)
        .await
        .expect("admin override should succeed");

    assert_eq!(live_task, LiveTaskNotification::Notified);
    assert!(matches!(
        task_control.try_recv_admin_command(),
        Some(TaskAdminCommand::ReloadAndRestart)
    ));
    assert!(task_control.try_recv_admin_command().is_none());
}

#[tokio::test]
async fn admin_remove_attempt_result_reports_unreachable_live_task() {
    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    let job_id = mk_job_id(32);

    store
        .expect_admin_remove_settlement_attempt_result()
        .once()
        .withf(
            move |requested_job_id, attempt_number, edit_even_if_completed| {
                requested_job_id == &job_id
                    && *attempt_number == 0
                    && *edit_even_if_completed == EditEvenIfCompleted::No
            },
        )
        .return_once(|_, _, _| Ok(()));

    let service = mk_service(Arc::new(store)).await;
    let (task_control_handle, task_control) = TaskControlHandle::new(&service.cancellation_token);
    // The task ended without deregistering itself yet: the command
    // channel receiver is gone.
    drop(task_control);
    service
        .task_controls
        .lock()
        .await
        .insert(job_id, task_control_handle);

    let live_task = service
        .admin_remove_attempt_result(job_id, 0, EditEvenIfCompleted::No)
        .await
        .expect("admin removal should succeed");

    assert_eq!(live_task, LiveTaskNotification::NotifyFailed);
}

#[tokio::test]
async fn admin_mutation_storage_error_propagates_without_task_notification() {
    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    let job_id = mk_job_id(33);

    store
        .expect_admin_insert_settlement_attempt()
        .once()
        .return_once(|_, _, _| {
            Err(agglayer_storage::error::Error::UnprocessedAction(
                "job already has a terminal result".to_string(),
            ))
        });

    let service = mk_service(Arc::new(store)).await;
    let (task_control_handle, mut task_control) =
        TaskControlHandle::new(&service.cancellation_token);
    service
        .task_controls
        .lock()
        .await
        .insert(job_id, task_control_handle);

    let result = service
        .admin_insert_settlement_attempt(job_id, mk_new_attempt(33), EditEvenIfCompleted::No)
        .await;

    assert!(result.is_err(), "storage refusal should propagate");
    assert!(
        task_control.try_recv_admin_command().is_none(),
        "a failed mutation must not trigger a task reload"
    );
}

#[tokio::test]
async fn admin_mutations_forward_the_force_flag_to_storage() {
    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    let job_id = mk_job_id(38);

    store
        .expect_admin_remove_settlement_attempt_result()
        .once()
        .withf(
            move |requested_job_id, attempt_number, edit_even_if_completed| {
                requested_job_id == &job_id
                    && *attempt_number == 1
                    && *edit_even_if_completed == EditEvenIfCompleted::Yes
            },
        )
        .return_once(|_, _, _| Ok(()));

    let service = mk_service(Arc::new(store)).await;

    let live_task = service
        .admin_remove_attempt_result(job_id, 1, EditEvenIfCompleted::Yes)
        .await
        .expect("forced removal should succeed");

    // A completed job has no live task; `Absent` is the expected state in
    // the forced-edit flow.
    assert_eq!(live_task, LiveTaskNotification::Absent);
}

#[tokio::test]
async fn admin_force_remove_job_result_refuses_while_task_is_live() {
    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    let job_id = mk_job_id(36);

    // No storage expectation beyond startup: the live-task guard must
    // trip before any storage write.
    let service = mk_service(Arc::new(store)).await;
    let (task_control_handle, _task_control) = TaskControlHandle::new(&service.cancellation_token);
    service
        .task_controls
        .lock()
        .await
        .insert(job_id, task_control_handle);

    let result = service
        .admin_force_remove_settlement_job_result(job_id)
        .await;

    let error = result.expect_err("a live task should block the removal");
    assert!(format!("{error:#}").contains("still live"));
}

#[tokio::test]
async fn admin_force_remove_job_result_respawns_task() {
    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    let job_id = mk_job_id(37);
    let job = mk_job(37);
    let removed_result = mk_result(37, ContractCallOutcome::Success);

    store
        .expect_admin_force_remove_settlement_job_result()
        .once()
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .return_once(|_| Ok(()));
    // The respawn reloads the job from storage, which is now pending.
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
        .expect_list_settlement_attempts()
        .once()
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .return_once(|_| Ok(Vec::new()));
    store
        .expect_list_settlement_attempt_results()
        .once()
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .return_once(|_| Ok(Vec::new()));

    let service = mk_service(Arc::new(store)).await;

    // The stale watcher of the completed job still broadcasts the removed
    // result; the removal must replace it.
    let (_stale_sender, stale_watcher) = watch::channel(Some(removed_result));
    service
        .result_watchers
        .lock()
        .await
        .insert(job_id, stale_watcher);

    service
        .admin_force_remove_settlement_job_result(job_id)
        .await
        .expect("force removal should succeed");

    assert!(
        service.task_controls.lock().await.contains_key(&job_id),
        "a fresh task should be registered for the job"
    );
    let watchers = service.result_watchers.lock().await;
    let watcher = watchers
        .get(&job_id)
        .expect("a fresh watcher should be registered for the job");
    assert_eq!(
        *watcher.borrow(),
        None,
        "the watcher must no longer broadcast the removed result"
    );
}
