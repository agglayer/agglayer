use super::*;

#[tokio::test]
async fn admin_abort_task_unknown_job_returns_job_not_found() {
    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    let job_id = mk_job_id(20);
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

    let error = service
        .admin_abort_task(job_id)
        .await
        .expect_err("abort of an unknown job must fail");
    assert!(matches!(
        error,
        crate::SettlementAdminError::JobNotFound(id) if id == job_id
    ));
}

#[tokio::test]
async fn admin_abort_task_completed_job_returns_job_completed() {
    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    let job_id = mk_job_id(21);
    let result = mk_result(21, ContractCallOutcome::Success);
    store
        .expect_get_settlement_job_result()
        .once()
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .return_once(move |_| Ok(Some(result)));

    let service = mk_service(Arc::new(store)).await;

    let error = service
        .admin_abort_task(job_id)
        .await
        .expect_err("abort of a completed job must fail");
    assert!(matches!(
        error,
        crate::SettlementAdminError::JobCompleted(id) if id == job_id
    ));
}

#[tokio::test]
async fn admin_abort_task_pending_job_without_task_returns_no_live_task() {
    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    let job_id = mk_job_id(22);
    let job = mk_job(22);
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

    let error = service
        .admin_abort_task(job_id)
        .await
        .expect_err("abort without a live task must fail");
    assert!(matches!(
        error,
        crate::SettlementAdminError::NoLiveTask(id) if id == job_id
    ));
}

#[tokio::test]
async fn admin_abort_task_storage_error_on_result_read_returns_storage_error() {
    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    let job_id = mk_job_id(24);
    store
        .expect_get_settlement_job_result()
        .once()
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .return_once(|_| {
            Err(agglayer_storage::error::Error::Unexpected(
                "boom".to_string(),
            ))
        });

    let service = mk_service(Arc::new(store)).await;

    let error = service
        .admin_abort_task(job_id)
        .await
        .expect_err("abort must surface the storage error");
    assert!(matches!(error, crate::SettlementAdminError::Storage { .. }));
}

#[tokio::test]
async fn admin_abort_task_storage_error_on_job_read_returns_storage_error() {
    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    let job_id = mk_job_id(25);
    store
        .expect_get_settlement_job_result()
        .once()
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .return_once(|_| Ok(None));
    store
        .expect_get_settlement_job()
        .once()
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .return_once(|_| {
            Err(agglayer_storage::error::Error::Unexpected(
                "boom".to_string(),
            ))
        });

    let service = mk_service(Arc::new(store)).await;

    let error = service
        .admin_abort_task(job_id)
        .await
        .expect_err("abort must surface the storage error");
    assert!(matches!(error, crate::SettlementAdminError::Storage { .. }));
}

#[tokio::test]
async fn admin_reload_and_restart_task_unknown_job_returns_job_not_found() {
    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    let job_id = mk_job_id(26);
    // Tolerant mocks: how often the reload path consults storage before
    // classifying the miss is an implementation detail.
    store
        .expect_get_settlement_job_result()
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .returning(|_| Ok(None));
    store
        .expect_get_settlement_job()
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .returning(|_| Ok(None));

    let service = mk_service(Arc::new(store)).await;

    let error = service
        .admin_reload_and_restart_task(job_id)
        .await
        .expect_err("reload of an unknown job must fail");
    assert!(matches!(
        error,
        crate::SettlementAdminError::JobNotFound(id) if id == job_id
    ));
}

/// Loads a pending job through `SettlementTask::load` and spawns its
/// task, mirroring the reload test above. The caller provides the
/// storage expectations for the load (job, no result, no attempts).
async fn load_and_spawn_pending_task(
    service: &SettlementService<impl Provider + WalletProvider + 'static, MockStateStore>,
    job_id: SettlementJobId,
) {
    let (task_control_handle, task_control) = TaskControlHandle::new(&service.cancellation_token);
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
        StoredSettlementJob::Completed(_, _) => {
            panic!("load should find a pending job")
        }
    };
    service
        .spawn_settlement_task(job_id, task, task_control_handle)
        .await;
}

/// Storage expectations for loading one pending job, tolerant of the
/// extra reads the spawned task performs before it gets cancelled.
fn expect_pending_job_reads(store: &mut MockStateStore, seed: u8) {
    let job = mk_job(seed);
    store
        .expect_get_settlement_job()
        .returning(move |_| Ok(Some(job.clone())));
    store
        .expect_get_settlement_job_result()
        .returning(|_| Ok(None));
    store
        .expect_list_settlement_attempts()
        .returning(|_| Ok(Vec::new()));
    store
        .expect_list_settlement_attempt_results()
        .returning(|_| Ok(Vec::new()));
    store
        .expect_max_settlement_nonce_for_wallet()
        .returning(|_| Ok(None));
}

#[tokio::test]
async fn admin_abort_task_cancels_live_task() {
    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    let job_id = mk_job_id(23);
    expect_pending_job_reads(&mut store, 23);

    let service = mk_service(Arc::new(store)).await;
    // Determinism assumption: on the current_thread test runtime the
    // spawned task is first polled after `cancel()` below, so it exits at
    // the loop-top control check without touching the dead provider
    // endpoint.
    load_and_spawn_pending_task(&service, job_id).await;
    assert!(service.has_live_task(job_id).await);

    service
        .admin_abort_task(job_id)
        .await
        .expect("abort of a live task must succeed");

    // The task observes the cancellation asynchronously.
    tokio::time::timeout(std::time::Duration::from_secs(10), async {
        while service.has_live_task(job_id).await {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("aborted task should deregister its control handle");
}

#[tokio::test]
async fn admin_reload_and_restart_respawns_dead_task() {
    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    let job_id = mk_job_id(24);
    expect_pending_job_reads(&mut store, 24);

    let cancellation_token = CancellationToken::new();
    let service = mk_service_with_token(Arc::new(store), cancellation_token.clone()).await;
    assert!(!service.has_live_task(job_id).await);

    // Determinism assumption: on the current_thread test runtime the
    // spawned task is first polled after `cancel()` below, so it exits at
    // the loop-top control check without touching the dead provider
    // endpoint.
    service
        .admin_reload_and_restart_task(job_id)
        .await
        .expect("reload of a pending job without a task must respawn it");

    assert!(service.has_live_task(job_id).await);

    // A retrieval after the respawn gets a functioning watcher.
    let retrieved = service
        .retrieve_settlement_result(job_id)
        .await
        .expect("retrieval after respawn should succeed");
    assert!(matches!(retrieved, RetrievedSettlementResult::Pending(_)));

    cancellation_token.cancel();
}

/// Regression for bot r3589631500 (PR #1681, line 366): a new-settlement
/// request must not interleave its create+spawn with an admin respawn.
///
/// `request_new_settlement` holds `admin_operation_lock` across create and
/// spawn, the same lock `admin_reload_and_restart_task` takes. This test
/// holds that lock externally and asserts `request_new_settlement` cannot
/// make progress (it blocks before `SettlementTask::create` touches storage),
/// proving the two are mutually exclusive. Without the fix the request runs
/// to completion while the lock is held.
#[tokio::test]
async fn request_new_settlement_serializes_against_admin_lock() {
    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    // `create` writes the job row once, but only after the request acquires
    // the admin lock. An observable counter lets the test assert the write
    // has NOT happened while the lock is held.
    let job_writes = Arc::new(Mutex::new(0usize));
    let job_writes_for_store = job_writes.clone();
    store.expect_insert_settlement_job().returning(move |_, _| {
        *job_writes_for_store.lock().unwrap() += 1;
        Ok(())
    });

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

    // Simulate an in-flight admin respawn by holding the admin operation
    // lock, then fire a new-settlement request.
    let admin_guard = service.admin_operation_lock.clone().lock_owned().await;

    let request_service = service.clone();
    let request = tokio::spawn(async move {
        request_service
            .request_new_settlement(None, mk_job(40))
            .await
    });

    // Give the spawned request ample time to run if it ignored the lock.
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    assert!(
        !request.is_finished(),
        "request_new_settlement must block on the admin operation lock"
    );
    assert_eq!(
        *job_writes.lock().unwrap(),
        0,
        "request_new_settlement must not persist the job while the admin lock is held"
    );

    // Releasing the lock lets the request proceed.
    drop(admin_guard);
    let watcher = tokio::time::timeout(std::time::Duration::from_secs(10), request)
        .await
        .expect("request should finish once the admin lock is free")
        .expect("request task should not panic")
        .expect("settlement request should be accepted");
    let _ = watcher;

    cancellation_token.cancel();
}

#[tokio::test]
async fn admin_reload_and_restart_completed_job_returns_job_completed() {
    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    let job_id = mk_job_id(25);
    let job = mk_job(25);
    let result = mk_result(25, ContractCallOutcome::Success);
    store
        .expect_get_settlement_job()
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .returning(move |_| Ok(Some(job.clone())));
    store
        .expect_get_settlement_job_result()
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .returning(move |_| Ok(Some(result.clone())));

    let service = mk_service(Arc::new(store)).await;

    let error = service
        .admin_reload_and_restart_task(job_id)
        .await
        .expect_err("reload of a completed job must fail");
    assert!(matches!(
        error,
        crate::SettlementAdminError::JobCompleted(id) if id == job_id
    ));
}

#[tokio::test]
async fn admin_reload_and_restart_live_task_sends_command() {
    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    let job_id = mk_job_id(27);
    // Strict counts: the initial explicit load is the only permitted
    // read, so a regression that respawned instead of sending the
    // command would trip these expectations with a second load.
    let job = mk_job(27);
    store
        .expect_get_settlement_job()
        .times(1)
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .returning(move |_| Ok(Some(job.clone())));
    store
        .expect_get_settlement_job_result()
        .times(1)
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .returning(|_| Ok(None));
    store
        .expect_list_settlement_attempts()
        .times(1)
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .returning(|_| Ok(Vec::new()));
    store
        .expect_list_settlement_attempt_results()
        .times(1)
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .returning(|_| Ok(Vec::new()));
    store
        .expect_max_settlement_nonce_for_wallet()
        .returning(|_| Ok(None));

    let cancellation_token = CancellationToken::new();
    let service = mk_service_with_token(Arc::new(store), cancellation_token.clone()).await;
    // Determinism assumption: on the current_thread test runtime the
    // spawned task is first polled after `cancel()` below, so it exits at
    // the loop-top control check without touching the dead provider
    // endpoint.
    load_and_spawn_pending_task(&service, job_id).await;

    service
        .admin_reload_and_restart_task(job_id)
        .await
        .expect("reload of a live task must be accepted");
    // No second load occurred: the command was queued to the live
    // task, not respawned. The task stays registered.
    assert!(service.has_live_task(job_id).await);

    cancellation_token.cancel();
}

#[tokio::test]
async fn admin_reload_and_restart_load_failure_returns_reload_failed() {
    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    let job_id = mk_job_id(28);
    let job = mk_job(28);
    // The job row exists (both for the load and for the failure
    // classification re-read), but hydrating the attempts fails, so
    // the load fails on a job that is still present.
    store
        .expect_get_settlement_job()
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .returning(move |_| Ok(Some(job.clone())));
    store
        .expect_get_settlement_job_result()
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .returning(|_| Ok(None));
    store
        .expect_list_settlement_attempt_results()
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .returning(|_| Ok(Vec::new()));
    store
        .expect_list_settlement_attempts()
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .returning(|_| {
            Err(agglayer_storage::error::Error::Unexpected(
                "boom".to_string(),
            ))
        });

    let service = mk_service(Arc::new(store)).await;

    let error = service
        .admin_reload_and_restart_task(job_id)
        .await
        .expect_err("reload must surface the load failure");
    assert!(matches!(
        error,
        crate::SettlementAdminError::ReloadFailed { .. }
    ));
}

#[tokio::test]
async fn admin_reload_and_restart_respawns_after_task_panic() {
    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    let job_id = mk_job_id(29);
    expect_pending_job_reads(&mut store, 29);

    let cancellation_token = CancellationToken::new();
    let service = mk_service_with_token(Arc::new(store), cancellation_token.clone()).await;

    // Simulate a panicked task: its control handle stays registered
    // while the receiver side is gone, along with a watcher whose
    // sender died with the task.
    let (task_control_handle, task_control) = TaskControlHandle::new(&service.cancellation_token);
    drop(task_control);
    service
        .task_controls
        .lock()
        .await
        .insert(job_id, task_control_handle);
    service
        .result_watchers
        .lock()
        .await
        .insert(job_id, watch::channel(None).1);
    assert!(service.has_live_task(job_id).await);

    service
        .admin_reload_and_restart_task(job_id)
        .await
        .expect("reload must respawn over the stale entry of a panicked task");

    // The respawned task registered a fresh, working handle.
    assert!(service.has_live_task(job_id).await);
    let control = service
        .task_controls
        .lock()
        .await
        .get(&job_id)
        .cloned()
        .expect("control handle must be registered");
    assert!(!control.is_closed());

    // The stale watcher was replaced by one backed by the respawned
    // task's live sender: retrieval reports the job as pending instead
    // of failing on a dead channel.
    let retrieved = service
        .retrieve_settlement_result(job_id)
        .await
        .expect("retrieval after respawn should succeed");
    match retrieved {
        RetrievedSettlementResult::Pending(mut watcher) => assert!(
            watcher.watcher().has_changed().is_ok(),
            "respawn must replace the stale watcher with a functioning one"
        ),
        RetrievedSettlementResult::Completed(_) => panic!("job must still be pending"),
    }

    cancellation_token.cancel();
}

#[tokio::test]
async fn admin_reload_over_panicked_completed_job_clears_stale_pending_watcher() {
    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    let job_id = mk_job_id(31);
    let stored_result = mk_result(31, ContractCallOutcome::Success);
    {
        let job = mk_job(31);
        store
            .expect_get_settlement_job()
            .withf(move |id| id == &job_id)
            .returning(move |_| Ok(Some(job.clone())));
        let result = stored_result.clone();
        store
            .expect_get_settlement_job_result()
            .withf(move |id| id == &job_id)
            .returning(move |_| Ok(Some(result.clone())));
    }

    let cancellation_token = CancellationToken::new();
    let service = mk_service_with_token(Arc::new(store), cancellation_token.clone()).await;

    // Panic-mid-completion state: closed control handle + a watcher
    // still holding `None`, both registered.
    let (handle, control) = TaskControlHandle::new(&service.cancellation_token);
    drop(control);
    service.task_controls.lock().await.insert(job_id, handle);
    let (_dead_sender, dead_receiver) = watch::channel(None);
    service
        .result_watchers
        .lock()
        .await
        .insert(job_id, dead_receiver);

    let error = service
        .admin_reload_and_restart_task(job_id)
        .await
        .expect_err("completed job must report JobCompleted");
    assert!(matches!(error, crate::SettlementAdminError::JobCompleted(id) if id == job_id));

    let retrieved = service
        .retrieve_settlement_result(job_id)
        .await
        .expect("retrieve must succeed");
    assert!(
        matches!(retrieved, RetrievedSettlementResult::Completed(result) if result == stored_result),
        "retrieve must read the completed result from storage, not the stale pending watcher",
    );

    cancellation_token.cancel();
}

/// A failed in-task reload must never leave a closed control handle
/// registered: closed-while-registered is reserved for panicked tasks,
/// and a concurrent admin reload observing it mid-teardown would spawn
/// a duplicate task that the dying run loop then strips from the maps.
#[tokio::test]
async fn failed_in_task_reload_never_leaves_a_closed_handle_registered() {
    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    let job_id = mk_job_id(30);
    let job = mk_job(30);
    store
        .expect_get_settlement_job()
        .returning(move |_| Ok(Some(job.clone())));
    store
        .expect_get_settlement_job_result()
        .returning(|_| Ok(None));
    store
        .expect_list_settlement_attempt_results()
        .returning(|_| Ok(Vec::new()));
    // The initial explicit load succeeds; the reload-triggered load
    // fails, tearing the task down through the run loop's error arm.
    let attempts_reads = Arc::new(Mutex::new(0usize));
    store.expect_list_settlement_attempts().returning(move |_| {
        let mut attempts_reads = attempts_reads.lock().unwrap();
        *attempts_reads += 1;
        if *attempts_reads == 1 {
            Ok(Vec::new())
        } else {
            Err(agglayer_storage::error::Error::Unexpected(
                "boom".to_string(),
            ))
        }
    });
    store
        .expect_max_settlement_nonce_for_wallet()
        .returning(|_| Ok(None));

    let cancellation_token = CancellationToken::new();
    let service = mk_service_with_token(Arc::new(store), cancellation_token.clone()).await;
    load_and_spawn_pending_task(&service, job_id).await;

    let control = service
        .task_controls
        .lock()
        .await
        .get(&job_id)
        .cloned()
        .expect("control handle must be registered");
    control
        .try_send(TaskAdminCommand::ReloadAndRestart)
        .expect("reload command should fit in admin channel");

    // The failed reload deregisters the task; sample every handle
    // observed on the way down. The sampling is best-effort: on the
    // current_thread runtime the teardown window has no forced yield,
    // so the decisive assertion is the end state (entry absent).
    tokio::time::timeout(std::time::Duration::from_secs(10), async {
        loop {
            let control = service.task_controls.lock().await.get(&job_id).cloned();
            match control {
                Some(control) => assert!(
                    !control.is_closed(),
                    "mid-reload task must not be registered as closed"
                ),
                None => break,
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("failed reload should deregister the control handle");

    cancellation_token.cancel();
}
