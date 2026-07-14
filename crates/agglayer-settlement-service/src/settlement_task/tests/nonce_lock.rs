use super::*;

/// A task aborted while queued on the per-wallet nonce lock must exit
/// promptly instead of staying parked until the lock holder releases
/// (e.g. a same-wallet job stuck in transient L1 retries).
/// Paused time makes the timeout fire immediately if the task stays
/// parked, so a regression fails fast instead of after 5 wall-clock
/// seconds.
#[tokio::test(start_paused = true)]
async fn abort_while_queued_on_wallet_nonce_lock_cancels_run() {
    let wallet_nonce_locks = Arc::new(WalletNonceLocks::default());
    let provider = mk_provider();
    let wallet = provider.default_signer_address();
    // The test holds the wallet lock so the task queues behind it.
    let _held_guard = wallet_nonce_locks.lock(wallet).await;

    let (control_handle, control) = TaskControlHandle::new(&CancellationToken::new());
    let mut task = SettlementTask {
        id: mk_job_id(1),
        job: mk_job(),
        tx_config: Arc::new(SettlementTransactionConfig::default()),
        provider: Arc::new(provider),
        // No expectations: an aborted queued task must not touch the store.
        store: Arc::new(MockStateStore::new()),
        wallet_nonce_locks: wallet_nonce_locks.clone(),
        control,
        attempts: BTreeMap::new(),
    };
    let run = tokio::spawn(async move { task.run().await });

    // Let the task reach the lock queue before aborting it.
    for _ in 0..10 {
        tokio::task::yield_now().await;
    }
    control_handle.cancel();

    let result = tokio::time::timeout(Duration::from_secs(5), run)
        .await
        .expect("aborted task must exit while the wallet lock is still held")
        .expect("settlement task must not panic");
    assert!(matches!(result, SettlementTaskRunResult::Cancelled));
}
