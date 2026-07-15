use std::{sync::Condvar, time::Duration};

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

/// Regression test for issue 1597: two settlement jobs building
/// concurrently on the same wallet must not be assigned the same
/// nonce.
///
/// The mock store gates the first `insert_settlement_attempt`
/// (bounded) until both jobs have read the stored nonce, so without
/// the per-wallet lock both jobs deterministically observe the same
/// value; with the lock, the second read cannot happen and the wait
/// times out. The wait sits on the save rather than the read
/// because mockall serializes concurrent calls to the same mocked
/// method. The store is stateful: saving an attempt raises the
/// value later reads observe, exactly like the real store.
// Multi-threaded runtime required: the mock store's condvar gate
// blocks worker threads, and would deadlock a current-thread runtime.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_jobs_on_same_wallet_get_distinct_nonces() {
    // `--no-mining` keeps submitted transactions pending so the run
    // loop cannot progress to settlement-result writes (which the
    // mock store does not expect) before the test aborts the jobs.
    let anvil = Anvil::new().arg("--no-mining").spawn();
    let provider = Arc::new(build_provider(&anvil));

    let recorded_nonces: Arc<Mutex<Vec<u64>>> = Arc::new(Mutex::new(Vec::new()));
    let highest_stored_nonce: Arc<Mutex<Option<u64>>> = Arc::new(Mutex::new(None));
    let read_rendezvous: Arc<(Mutex<u32>, Condvar)> = Arc::new((Mutex::new(0), Condvar::new()));

    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    store
        .expect_insert_settlement_job()
        .times(2)
        .returning(|_, _| Ok(()));
    store.expect_max_settlement_nonce_for_wallet().returning({
        let highest_stored_nonce = highest_stored_nonce.clone();
        let read_rendezvous = read_rendezvous.clone();
        move |_| {
            // Count the read and wake a saver parked in the
            // `insert_settlement_attempt` gate below. The wait
            // cannot live here: mockall runs this closure under a
            // per-method mutex, so two readers are never inside it
            // concurrently.
            let (read_count, condvar) = &*read_rendezvous;
            *read_count.lock().unwrap() += 1;
            condvar.notify_all();
            Ok(highest_stored_nonce.lock().unwrap().map(Nonce))
        }
    });
    store
        .expect_insert_settlement_attempt()
        .times(2)
        .returning({
            let recorded_nonces = recorded_nonces.clone();
            let highest_stored_nonce = highest_stored_nonce.clone();
            let read_rendezvous = read_rendezvous.clone();
            move |_, _, attempt| {
                // Rendezvous gate: hold the first save (bounded)
                // until both jobs have read the stored nonce, so
                // unsynchronized tasks deterministically overlap in
                // the read-to-save window. With the per-wallet lock
                // in place the second read cannot happen while the
                // first job holds the lock, so the wait times out
                // and the read-save pairs serialize.
                let (read_count, condvar) = &*read_rendezvous;
                let reads = read_count.lock().unwrap();
                let (reads, _timed_out) = condvar
                    .wait_timeout_while(reads, Duration::from_secs(2), |reads| *reads < 2)
                    .unwrap();
                drop(reads);
                recorded_nonces.lock().unwrap().push(attempt.nonce.0);
                let mut highest = highest_stored_nonce.lock().unwrap();
                *highest = Some(highest.map_or(attempt.nonce.0, |h| h.max(attempt.nonce.0)));
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
    .expect("settlement service should start");

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
