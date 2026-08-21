use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Condvar, Mutex,
    },
    time::Duration,
};

use agglayer_config::Config;
use agglayer_storage::tests::mocks::{
    MockDebugStore, MockEpochsStore, MockPendingStore, MockStateStore,
};
use agglayer_types::{CertificateId, Digest};
use alloy::providers::{mock::Asserter, ProviderBuilder};

#[tokio::test(flavor = "current_thread")]
async fn certificate_header_query_keeps_the_runtime_responsive() {
    let certificate_id = CertificateId::new(Digest([1; 32]));
    let (storage_started, storage_started_rx) = tokio::sync::oneshot::channel();
    let release = Arc::new((Mutex::new(false), Condvar::new()));
    let release_for_storage = release.clone();
    let storage_timed_out = Arc::new(AtomicBool::new(false));
    let storage_timed_out_from_query = storage_timed_out.clone();

    let mut state_store = MockStateStore::new();
    state_store
        .expect_get_certificate_header()
        .return_once(move |_| {
            storage_started
                .send(())
                .expect("test task should wait for the storage call");

            let (released, wake) = &*release_for_storage;
            // The timeout only prevents a regressed synchronous call from hanging the
            // suite; the test task releases this immediately when the runtime
            // remains responsive.
            let (_released, timeout) = wake
                .wait_timeout_while(
                    released
                        .lock()
                        .expect("release lock should not be poisoned"),
                    Duration::from_secs(10),
                    |released| !*released,
                )
                .expect("release lock should not be poisoned");
            storage_timed_out_from_query.store(timeout.timed_out(), Ordering::SeqCst);
            Ok(None)
        });

    let asserter = Asserter::new();
    let service = Arc::new(crate::AgglayerService::new(
        tokio::sync::mpsc::channel(1).0,
        Arc::new(MockPendingStore::new()),
        Arc::new(state_store),
        Arc::new(MockDebugStore::new()),
        Arc::new(MockEpochsStore::new()),
        Arc::new(Config::default()),
        Arc::new(ProviderBuilder::new().connect_mocked_client(asserter)),
    ));

    let query = tokio::spawn(async move { service.fetch_certificate_header(certificate_id).await });

    storage_started_rx
        .await
        .expect("blocking storage task should start");
    let (released, wake) = &*release;
    *released
        .lock()
        .expect("release lock should not be poisoned") = true;
    wake.notify_one();

    assert!(matches!(
        query.await.expect("query task should not panic"),
        Err(crate::CertificateRetrievalError::NotFound { .. })
    ));
    assert!(
        !storage_timed_out.load(Ordering::SeqCst),
        "the current-thread runtime did not run while storage was blocked"
    );
}
