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
    // while the receiver side is gone.
    let (task_control_handle, task_control) = TaskControlHandle::new(&service.cancellation_token);
    drop(task_control);
    service
        .task_controls
        .lock()
        .await
        .insert(job_id, task_control_handle);
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

    cancellation_token.cancel();
}
