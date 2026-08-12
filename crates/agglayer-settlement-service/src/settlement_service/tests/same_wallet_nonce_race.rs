use std::time::Duration;

use alloy::node_bindings::Anvil;

use super::*;
use crate::utils::build_provider;

async fn wait_for_recorded_nonces(recorded: &Arc<Mutex<Vec<u64>>>, count: usize) {
    tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            if recorded.lock().unwrap().len() >= count {
                return;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("timed out waiting for settlement attempts to be recorded");
}

/// Regression for issues #1575 / #1597: two settlement jobs building
/// concurrently on the same wallet must receive distinct, gap-free nonces
/// from the shared allocator.
///
/// `--no-mining` keeps submitted transactions pending so the run loop cannot
/// progress to settlement-result writes (which the mock store does not expect)
/// before the test aborts the jobs.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_jobs_on_same_wallet_get_distinct_nonces() {
    let anvil = Anvil::new().arg("--no-mining").spawn();
    let provider = Arc::new(build_provider(&anvil));

    let recorded_nonces: Arc<Mutex<Vec<u64>>> = Arc::new(Mutex::new(Vec::new()));

    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    store
        .expect_insert_settlement_job()
        .times(2)
        .returning(|_, _| Ok(()));
    // Seeding may call this zero or more times depending on who wins the
    // first-seed race; allow any count and always report an empty queue.
    store
        .expect_max_settlement_nonce_for_wallet()
        .returning(|_| Ok(None));
    store
        .expect_insert_settlement_attempt()
        .times(2)
        .returning({
            let recorded_nonces = recorded_nonces.clone();
            move |_, _, attempt| {
                recorded_nonces.lock().unwrap().push(attempt.nonce.0);
                Ok(())
            }
        });

    let store = Arc::new(store);
    let service = SettlementService::start(
        SettlementServiceConfig::default(),
        Arc::new(SettlementTransactionConfig::default()),
        provider,
        store,
        CancellationToken::new(),
    )
    .await
    .expect("settlement service should start")
    .0;

    let (watcher_a, watcher_b) = tokio::join!(
        service.request_new_settlement(None, mk_job(1)),
        service.request_new_settlement(None, mk_job(2)),
    );
    let watcher_a = watcher_a.expect("job a should be created");
    let watcher_b = watcher_b.expect("job b should be created");

    wait_for_recorded_nonces(&recorded_nonces, 2).await;

    service
        .admin_abort_task(watcher_a.job_id())
        .await
        .expect("abort job a");
    service
        .admin_abort_task(watcher_b.job_id())
        .await
        .expect("abort job b");

    let mut nonces = recorded_nonces.lock().unwrap().clone();
    nonces.sort_unstable();
    assert_eq!(
        nonces,
        vec![0, 1],
        "concurrent same-wallet jobs must receive distinct consecutive nonces"
    );
}
