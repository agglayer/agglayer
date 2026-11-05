use std::{
    borrow::Cow,
    error::Error,
    fmt::{Display, Formatter},
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

use agglayer_config::Multiplier;
use alloy::{
    consensus::Transaction as _,
    network::TransactionBuilder as _,
    node_bindings::Anvil,
    primitives::U256,
    providers::{Provider, ProviderBuilder},
    rpc::types::TransactionRequest,
    transports::{RpcError, TransportError, TransportErrorKind},
};
use tokio::time::{advance, Instant};
use tokio_util::sync::CancellationToken;

use super::*;

// Existing single-endpoint variable used across the repository.
const L1_RPC_ENV: &str = "L1_RPC_ENDPOINT";
const MAX_SCAN_BLOCKS: u64 = 2_048;

#[derive(Debug)]
struct TransientTestError;

impl Display for TransientTestError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "transient test error")
    }
}

impl Error for TransientTestError {}

#[derive(Debug)]
struct PermanentTestError;

impl Display for PermanentTestError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "permanent test error")
    }
}

impl Error for PermanentTestError {}

fn retry_policy(
    initial_interval: Duration,
    interval_multiplier_factor: u64,
    max_interval: Duration,
    jitter: Duration,
) -> TxRetryPolicy {
    TxRetryPolicy {
        initial_interval,
        interval_multiplier_factor: Multiplier::from_u64_per_1000(interval_multiplier_factor),
        max_interval,
        jitter,
    }
}

#[tokio::test]
async fn retry_callback_until_success_returns_permanent_error_immediately() {
    let attempts = Arc::new(AtomicUsize::new(0));
    let cancellation_token = CancellationToken::new();
    let policy = retry_policy(
        Duration::from_millis(10),
        2000,
        Duration::from_millis(40),
        Duration::ZERO,
    );

    let error = retry_callback_until_success(
        &policy,
        &cancellation_token,
        || {
            let attempts = attempts.clone();
            async move {
                attempts.fetch_add(1, Ordering::SeqCst);
                Err::<(), _>(PermanentTestError)
            }
        },
        |_| false,
        |_| true,
    )
    .await
    .unwrap_err();

    assert_eq!(attempts.load(Ordering::SeqCst), 1);
    assert!(matches!(error, RetryCallbackError::Error(_)));
}

#[tokio::test]
async fn retry_callback_until_success_retries_transient_error_until_ok() {
    let attempts = Arc::new(AtomicUsize::new(0));
    let cancellation_token = CancellationToken::new();
    let policy = retry_policy(Duration::ZERO, 1000, Duration::ZERO, Duration::ZERO);

    let value = retry_callback_until_success(
        &policy,
        &cancellation_token,
        || {
            let attempts = attempts.clone();
            async move {
                let attempt = attempts.fetch_add(1, Ordering::SeqCst);
                if attempt < 2 {
                    Err::<u64, _>(TransientTestError)
                } else {
                    Ok(42)
                }
            }
        },
        |_| true,
        |_| true,
    )
    .await
    .unwrap();

    assert_eq!(value, 42);
    assert_eq!(attempts.load(Ordering::SeqCst), 3);
}

/// Counts emitted tracing events by level, to assert which level the retry
/// helper picks for a transient error.
struct LevelCountingSubscriber {
    warn_events: Arc<AtomicUsize>,
    debug_events: Arc<AtomicUsize>,
}

impl tracing::Subscriber for LevelCountingSubscriber {
    fn enabled(&self, _metadata: &tracing::Metadata<'_>) -> bool {
        true
    }

    fn new_span(&self, _span: &tracing::span::Attributes<'_>) -> tracing::span::Id {
        tracing::span::Id::from_u64(1)
    }

    fn record(&self, _span: &tracing::span::Id, _values: &tracing::span::Record<'_>) {}

    fn record_follows_from(&self, _span: &tracing::span::Id, _follows: &tracing::span::Id) {}

    fn event(&self, event: &tracing::Event<'_>) {
        match *event.metadata().level() {
            tracing::Level::WARN => self.warn_events.fetch_add(1, Ordering::SeqCst),
            tracing::Level::DEBUG => self.debug_events.fetch_add(1, Ordering::SeqCst),
            _ => 0,
        };
    }

    fn enter(&self, _span: &tracing::span::Id) {}

    fn exit(&self, _span: &tracing::span::Id) {}
}

/// Runs `retry_callback_until_success` through `failures` transient
/// failures with the given `needs_warning_log`, returning `(warn_events,
/// debug_events)`.
async fn count_retry_log_levels(
    failures: usize,
    needs_warning_log: fn(&TransientTestError) -> bool,
) -> (usize, usize) {
    let warn_events = Arc::new(AtomicUsize::new(0));
    let debug_events = Arc::new(AtomicUsize::new(0));
    // Thread-local default: `#[tokio::test]` runs on a current-thread
    // runtime, so every retry log lands on this subscriber and parallel
    // tests cannot pollute the counters.
    let _guard = tracing::subscriber::set_default(LevelCountingSubscriber {
        warn_events: warn_events.clone(),
        debug_events: debug_events.clone(),
    });

    let attempts = Arc::new(AtomicUsize::new(0));
    let cancellation_token = CancellationToken::new();
    let policy = retry_policy(Duration::ZERO, 1000, Duration::ZERO, Duration::ZERO);

    retry_callback_until_success(
        &policy,
        &cancellation_token,
        || {
            let attempts = attempts.clone();
            async move {
                if attempts.fetch_add(1, Ordering::SeqCst) < failures {
                    Err::<(), _>(TransientTestError)
                } else {
                    Ok(())
                }
            }
        },
        |_| true,
        needs_warning_log,
    )
    .await
    .unwrap();

    (
        warn_events.load(Ordering::SeqCst),
        debug_events.load(Ordering::SeqCst),
    )
}

#[tokio::test]
async fn retry_callback_until_success_logs_retries_at_warning_by_default() {
    assert_eq!(count_retry_log_levels(2, |_| true).await, (2, 0));
}

#[tokio::test]
async fn retry_callback_until_success_logs_quiet_retries_at_debug() {
    assert_eq!(count_retry_log_levels(2, |_| false).await, (0, 2));
}

#[tokio::test]
async fn retry_callback_until_success_escalates_quiet_retries_to_warning_after_threshold() {
    // 65 consecutive failures: retries 1..=63 stay at debug, 64 and 65 are
    // escalated to warnings.
    assert_eq!(count_retry_log_levels(65, |_| false).await, (2, 63));
}

#[tokio::test(start_paused = true)]
async fn retry_callback_until_success_stops_when_cancelled() {
    let attempts = Arc::new(AtomicUsize::new(0));
    let cancellation_token = CancellationToken::new();
    let policy = retry_policy(
        Duration::from_secs(30),
        1000,
        Duration::from_secs(30),
        Duration::ZERO,
    );

    let handle = tokio::spawn({
        let attempts = attempts.clone();
        let cancellation_token = cancellation_token.clone();
        async move {
            retry_callback_until_success(
                &policy,
                &cancellation_token,
                || {
                    let attempts = attempts.clone();
                    async move {
                        attempts.fetch_add(1, Ordering::SeqCst);
                        Err::<(), _>(TransientTestError)
                    }
                },
                |_| true,
                |_| true,
            )
            .await
        }
    });

    tokio::task::yield_now().await;
    assert_eq!(attempts.load(Ordering::SeqCst), 1);

    cancellation_token.cancel();

    let error = handle.await.unwrap().unwrap_err();
    assert_eq!(attempts.load(Ordering::SeqCst), 1);
    assert!(matches!(error, RetryCallbackError::Cancelled));
}

#[tokio::test]
async fn retry_callback_until_success_returns_cancelled_before_calling_callback() {
    let cancellation_token = CancellationToken::new();
    cancellation_token.cancel();
    let policy = retry_policy(
        Duration::from_millis(10),
        1000,
        Duration::from_millis(10),
        Duration::ZERO,
    );
    let called = Arc::new(AtomicBool::new(false));

    let result = retry_callback_until_success(
        &policy,
        &cancellation_token,
        {
            let called = called.clone();
            move || {
                let called = called.clone();
                async move {
                    called.store(true, Ordering::SeqCst);
                    Ok::<(), TransientTestError>(())
                }
            }
        },
        |_| true,
        |_| true,
    )
    .await;

    assert!(matches!(result, Err(RetryCallbackError::Cancelled)));
    assert!(
        !called.load(Ordering::SeqCst),
        "callback must not run once the token is already cancelled"
    );
}

#[tokio::test]
async fn retry_callback_until_success_stops_when_cancelled_during_callback() {
    let cancellation_token = CancellationToken::new();
    let policy = retry_policy(
        Duration::from_secs(30),
        1000,
        Duration::from_secs(30),
        Duration::ZERO,
    );

    let handle = tokio::spawn({
        let cancellation_token = cancellation_token.clone();
        async move {
            retry_callback_until_success(
                &policy,
                &cancellation_token,
                std::future::pending::<Result<(), TransientTestError>>,
                |_| true,
                |_| true,
            )
            .await
        }
    });

    tokio::task::yield_now().await;
    cancellation_token.cancel();

    let result = tokio::time::timeout(Duration::from_secs(5), handle)
        .await
        .expect("retry must observe cancellation during a pending callback")
        .expect("retry task should not panic");
    assert!(matches!(result, Err(RetryCallbackError::Cancelled)));
}

#[tokio::test(start_paused = true)]
async fn retry_callback_until_success_applies_backoff_until_max_interval() {
    let attempts = Arc::new(AtomicUsize::new(0));
    let call_times = Arc::new(Mutex::new(Vec::<Instant>::new()));
    let cancellation_token = CancellationToken::new();
    let policy = retry_policy(
        Duration::from_millis(10),
        2000,
        Duration::from_millis(25),
        Duration::ZERO,
    );

    let handle = tokio::spawn({
        let attempts = attempts.clone();
        let call_times = call_times.clone();
        let cancellation_token = cancellation_token.clone();
        async move {
            retry_callback_until_success(
                &policy,
                &cancellation_token,
                || {
                    let attempts = attempts.clone();
                    let call_times = call_times.clone();
                    async move {
                        call_times.lock().unwrap().push(Instant::now());
                        let attempt = attempts.fetch_add(1, Ordering::SeqCst);
                        if attempt < 3 {
                            Err::<(), _>(TransientTestError)
                        } else {
                            Ok(())
                        }
                    }
                },
                |_| true,
                |_| true,
            )
            .await
        }
    });

    tokio::task::yield_now().await;
    assert_eq!(attempts.load(Ordering::SeqCst), 1);

    advance(Duration::from_millis(10)).await;
    tokio::task::yield_now().await;
    assert_eq!(attempts.load(Ordering::SeqCst), 2);

    advance(Duration::from_millis(20)).await;
    tokio::task::yield_now().await;
    assert_eq!(attempts.load(Ordering::SeqCst), 3);

    advance(Duration::from_millis(25)).await;
    handle.await.unwrap().unwrap();

    let call_times = call_times.lock().unwrap();
    let intervals = call_times
        .windows(2)
        .map(|window| window[1] - window[0])
        .collect::<Vec<_>>();
    assert_eq!(
        intervals,
        vec![
            Duration::from_millis(10),
            Duration::from_millis(20),
            Duration::from_millis(25),
        ]
    );
}

#[tokio::test(start_paused = true)]
async fn retry_alloy_callback_until_success_retries_rate_limited_errors() {
    let attempts = Arc::new(AtomicUsize::new(0));
    let cancellation_token = CancellationToken::new();
    let policy = retry_policy(
        Duration::from_millis(10),
        1000,
        Duration::from_millis(10),
        Duration::ZERO,
    );

    let handle = tokio::spawn({
        let attempts = attempts.clone();
        let cancellation_token = cancellation_token.clone();
        async move {
            retry_alloy_callback_until_success(&policy, &cancellation_token, || {
                let attempts = attempts.clone();
                async move {
                    let attempt = attempts.fetch_add(1, Ordering::SeqCst);
                    if attempt < 2 {
                        let error: TransportError =
                            RpcError::ErrorResp(alloy::rpc::json_rpc::ErrorPayload {
                                code: 429,
                                message: Cow::Borrowed("too many requests"),
                                data: None,
                            });
                        Err::<u64, _>(error)
                    } else {
                        Ok(7)
                    }
                }
            })
            .await
        }
    });

    tokio::task::yield_now().await;

    advance(Duration::from_millis(10)).await;
    tokio::task::yield_now().await;
    advance(Duration::from_millis(10)).await;

    assert_eq!(handle.await.unwrap().unwrap(), 7);
    assert_eq!(attempts.load(Ordering::SeqCst), 3);
}

#[test]
fn transient_alloy_error_recognizes_request_timeouts_narrowly() {
    const TIMEOUT_RESPONSE: &str =
        r#"{"id":11481,"jsonrpc":"2.0","error":{"code":-32009,"message":"Request timed out"}}"#;

    assert!(is_transient_alloy_error(&TransportErrorKind::http_error(
        408,
        TIMEOUT_RESPONSE.to_owned()
    )));
    assert!(is_transient_alloy_error(&TransportErrorKind::http_error(
        408,
        String::new()
    )));
    assert!(is_transient_alloy_error(&TransportErrorKind::http_error(
        504,
        String::new()
    )));
    assert!(!is_transient_alloy_error(&TransportErrorKind::http_error(
        400,
        TIMEOUT_RESPONSE.to_owned()
    )));
    for message in ["Request timed out", "REQUEST TIMEOUT", "request time-out"] {
        let error = RpcError::ErrorResp(alloy::rpc::json_rpc::ErrorPayload {
            code: -32009,
            message: Cow::Borrowed(message),
            data: None,
        });
        assert!(is_transient_alloy_error(&error));
    }

    for (code, message) in [
        (-32009, "invalid signature"),
        (-32000, "Request timed out"),
        (-32602, "Request timed out"),
        (-32009, "Time limit elapsed while sending output"),
    ] {
        let error = RpcError::ErrorResp(alloy::rpc::json_rpc::ErrorPayload {
            code,
            message: Cow::Borrowed(message),
            data: None,
        });
        assert!(!is_transient_alloy_error(&error));
    }
}

#[tokio::test(start_paused = true)]
async fn retry_alloy_callback_until_success_retries_http_timeout_responses() {
    const TIMEOUT_RESPONSE: &str =
        r#"{"id":11481,"jsonrpc":"2.0","error":{"code":-32009,"message":"Request timed out"}}"#;

    let attempts = Arc::new(AtomicUsize::new(0));
    let cancellation_token = CancellationToken::new();
    let policy = retry_policy(
        Duration::from_millis(10),
        1000,
        Duration::from_millis(10),
        Duration::ZERO,
    );

    let handle = tokio::spawn({
        let attempts = attempts.clone();
        let cancellation_token = cancellation_token.clone();
        async move {
            retry_alloy_callback_until_success(&policy, &cancellation_token, || {
                let attempts = attempts.clone();
                async move {
                    let attempt = attempts.fetch_add(1, Ordering::SeqCst);
                    if attempt < 2 {
                        Err::<u64, _>(TransportErrorKind::http_error(
                            if attempt == 0 { 408 } else { 504 },
                            TIMEOUT_RESPONSE.to_owned(),
                        ))
                    } else {
                        Ok(11)
                    }
                }
            })
            .await
        }
    });

    tokio::task::yield_now().await;
    assert_eq!(attempts.load(Ordering::SeqCst), 1);

    advance(Duration::from_millis(10)).await;
    tokio::task::yield_now().await;
    assert_eq!(attempts.load(Ordering::SeqCst), 2);

    advance(Duration::from_millis(10)).await;

    assert_eq!(handle.await.unwrap().unwrap(), 11);
    assert_eq!(attempts.load(Ordering::SeqCst), 3);
}

#[tokio::test(start_paused = true)]
async fn retry_alloy_callback_until_success_retries_retryable_deser_errors() {
    let attempts = Arc::new(AtomicUsize::new(0));
    let cancellation_token = CancellationToken::new();
    let policy = retry_policy(
        Duration::from_millis(10),
        1000,
        Duration::from_millis(10),
        Duration::ZERO,
    );

    let handle = tokio::spawn({
        let attempts = attempts.clone();
        let cancellation_token = cancellation_token.clone();
        async move {
            retry_alloy_callback_until_success(&policy, &cancellation_token, || {
                let attempts = attempts.clone();
                async move {
                    let attempt = attempts.fetch_add(1, Ordering::SeqCst);
                    if attempt < 2 {
                        Err::<u64, _>(RpcError::DeserError {
                            err: serde_json::from_str::<u64>("not json").unwrap_err(),
                            text: r#"{"error":{"code":429,"message":"too many requests"}}"#
                                .to_string(),
                        })
                    } else {
                        Ok(9)
                    }
                }
            })
            .await
        }
    });

    tokio::task::yield_now().await;

    advance(Duration::from_millis(10)).await;
    tokio::task::yield_now().await;
    advance(Duration::from_millis(10)).await;

    assert_eq!(handle.await.unwrap().unwrap(), 9);
    assert_eq!(attempts.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn retry_alloy_callback_until_success_returns_permanent_error_immediately() {
    let attempts = Arc::new(AtomicUsize::new(0));
    let cancellation_token = CancellationToken::new();
    let policy = retry_policy(
        Duration::from_millis(10),
        1000,
        Duration::from_millis(10),
        Duration::ZERO,
    );

    let error = retry_alloy_callback_until_success(&policy, &cancellation_token, || {
        let attempts = attempts.clone();
        async move {
            attempts.fetch_add(1, Ordering::SeqCst);
            let error: TransportError = RpcError::ErrorResp(alloy::rpc::json_rpc::ErrorPayload {
                code: -32601,
                message: Cow::Borrowed("Method not found"),
                data: None,
            });
            Err::<(), _>(error)
        }
    })
    .await
    .unwrap_err();

    assert_eq!(attempts.load(Ordering::SeqCst), 1);
    assert!(matches!(error, RetryCallbackError::Error(_)));
}

fn external_rpc_url_from_env() -> Option<String> {
    match std::env::var(L1_RPC_ENV) {
        Ok(url) if url.trim().is_empty() => {
            println!("{L1_RPC_ENV} is set but empty; failing test");
            panic!("{L1_RPC_ENV} is defined but empty");
        }
        Ok(url) => {
            println!("{L1_RPC_ENV} is set; running external RPC compatibility check");
            Some(url)
        }
        Err(_) => {
            println!("{L1_RPC_ENV} is not set; skipping external RPC compatibility check");
            None
        }
    }
}

async fn find_recent_mined_transaction(
    provider: &impl Provider,
) -> TransportResult<Option<(Address, u64, SettlementTxHash)>> {
    let latest_block = provider.get_block_number().await?;
    let blocks_to_scan = latest_block.saturating_add(1).min(MAX_SCAN_BLOCKS);

    println!("Scanning up to {blocks_to_scan} block(s) for a mined transaction sample");

    for offset in 0..blocks_to_scan {
        let block_number = latest_block - offset;
        let Some(block) = provider
            .get_block_by_number(block_number.into())
            .full()
            .await?
        else {
            continue;
        };

        let Some(tx) = block.transactions.first_transaction() else {
            continue;
        };

        println!(
            "Found sample transaction in block {block_number} at nonce {}",
            tx.nonce()
        );

        return Ok(Some((
            tx.from(),
            tx.nonce(),
            SettlementTxHash::from(tx.tx_hash()),
        )));
    }

    println!("No mined transaction sample found in scan range");

    Ok(None)
}

#[tokio::test]
async fn tx_hash_on_l1_for_nonce_returns_mined_tx() {
    let anvil = Anvil::new().spawn();
    let sender = anvil.addresses()[0];
    let provider = build_provider(&anvil);

    let tx = TransactionRequest::default()
        .to(anvil.addresses()[1])
        .value(U256::from(1));
    let receipt = provider
        .send_transaction(tx)
        .await
        .unwrap()
        .get_receipt()
        .await
        .unwrap();

    // Anvil's `eth_getTransactionBySenderAndNonce` index can briefly lag
    // behind receipt availability under load (e.g. coverage instrumentation
    // on CI), transiently returning `None` or a still-pending transaction
    // for a freshly mined nonce. Poll with a bounded deadline so the test
    // asserts eventual consistency instead of a single racy read; a genuine
    // regression still fails once the deadline elapses.
    let expected = SettlementTxHash::from(receipt.transaction_hash);
    let deadline = Instant::now() + Duration::from_secs(5);
    let result = loop {
        let result = tx_hash_on_l1_for_nonce(&provider, sender, Nonce(0))
            .await
            .unwrap();
        if result.is_some() || Instant::now() >= deadline {
            break result;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    };
    assert_eq!(result, Some(expected));
}

#[tokio::test]
async fn tx_hash_on_l1_for_nonce_ignores_mempool_only_tx() {
    let anvil = Anvil::new().arg("--no-mining").spawn();
    let sender = anvil.addresses()[0];
    let provider = build_provider(&anvil);

    let tx = TransactionRequest::default()
        .to(anvil.addresses()[1])
        .value(U256::from(1));
    let _ = provider.send_transaction(tx).await.unwrap();

    let result = tx_hash_on_l1_for_nonce(&provider, sender, Nonce(0))
        .await
        .unwrap();
    assert_eq!(result, None);
}

#[tokio::test]
async fn tx_hash_on_l1_for_nonce_returns_none_for_non_submitted_nonce() {
    let anvil = Anvil::new().spawn();
    let sender = anvil.addresses()[0];
    let provider = build_provider(&anvil);

    let result = tx_hash_on_l1_for_nonce(&provider, sender, Nonce(0))
        .await
        .unwrap();
    assert_eq!(result, None);
}

#[tokio::test]
async fn tx_hash_on_l1_for_nonce_maps_tenderly_not_found_to_none() {
    // Tenderly Gateway answers `-32001 "not found"` instead of the `null`
    // other nodes return when no transaction matches the sender and nonce.
    let asserter = alloy::providers::mock::Asserter::new();
    let provider = ProviderBuilder::new().connect_mocked_client(asserter.clone());
    asserter.push_failure(alloy::rpc::json_rpc::ErrorPayload {
        code: -32001,
        message: Cow::Borrowed("not found"),
        data: None,
    });

    let result = tx_hash_on_l1_for_nonce(&provider, Address::ZERO, Nonce(0))
        .await
        .unwrap();
    assert_eq!(result, None);
}

#[tokio::test]
async fn tx_hash_on_l1_for_nonce_propagates_other_error_responses() {
    let asserter = alloy::providers::mock::Asserter::new();
    let provider = ProviderBuilder::new().connect_mocked_client(asserter.clone());
    asserter.push_failure(alloy::rpc::json_rpc::ErrorPayload {
        code: -32001,
        message: Cow::Borrowed("header not found"),
        data: None,
    });

    let error = tx_hash_on_l1_for_nonce(&provider, Address::ZERO, Nonce(0))
        .await
        .unwrap_err();
    assert!(matches!(&error, RpcError::ErrorResp(error) if error.code == -32001));
}

#[tokio::test]
async fn contract_call_result_from_receipt_maps_revert() {
    let anvil = Anvil::new().spawn();
    let provider = build_provider(&anvil);

    // Deployment whose initcode immediately reverts (PUSH1 0 PUSH1 0 REVERT),
    // with an explicit gas limit so the failing tx skips estimation and gets
    // mined.
    let tx = TransactionRequest::default()
        .into_create()
        .input(Bytes::from_static(&[0x60, 0x00, 0x60, 0x00, 0xfd]).into())
        .gas_limit(100_000);
    let receipt = provider
        .send_transaction(tx)
        .await
        .unwrap()
        .get_receipt()
        .await
        .unwrap();
    assert!(!receipt.status());

    let result = contract_call_result_from_receipt(&receipt)
        .expect("mined reverted tx should have a result");
    assert_eq!(result.outcome, ContractCallOutcome::Revert);
}

// Manual run for any custom L1 RPC endpoint:
// L1_RPC_ENDPOINT="https://<your-rpc-url>" cargo test -p agglayer-settlement-service tx_hash_on_l1_for_nonce_supports_external_l1_rpc_when_configured -- --nocapture
#[tokio::test]
async fn tx_hash_on_l1_for_nonce_supports_external_l1_rpc_when_configured() {
    println!("Starting external L1 RPC sender+nonce lookup test");

    let Some(rpc_url) = external_rpc_url_from_env() else {
        return;
    };

    let parsed_rpc_url = match rpc_url.parse() {
        Ok(url) => url,
        Err(_) => panic!("{L1_RPC_ENV} is invalid"),
    };

    println!("Parsed RPC URL; creating HTTP provider");

    let provider = ProviderBuilder::new().connect_http(parsed_rpc_url);

    println!("Fetching a mined transaction sample from recent blocks");

    let sample = match find_recent_mined_transaction(&provider).await {
        Ok(sample) => sample,
        Err(_) => panic!("Failed to query recent blocks through {L1_RPC_ENV}"),
    };

    let Some((sender, nonce, expected_hash)) = sample else {
        panic!(
            "No mined transactions found in the last {} blocks through {}; submit at least one \
             transaction and retry",
            MAX_SCAN_BLOCKS, L1_RPC_ENV,
        );
    };

    println!("Querying tx hash via wallet + nonce RPC");

    let result = match tx_hash_on_l1_for_nonce(&provider, sender, Nonce(nonce)).await {
        Ok(result) => result,
        Err(_) => panic!("{L1_RPC_ENV} rejected eth_getTransactionBySenderAndNonce"),
    };

    println!("Comparing RPC result with sampled transaction hash");

    assert_eq!(
        result,
        Some(expected_hash),
        "Unexpected tx hash when querying by wallet + nonce through {}",
        L1_RPC_ENV,
    );

    println!("Querying an absent (wallet, nonce) pair via wallet + nonce RPC");

    // A nonce far beyond anything the sampled wallet has used, so no
    // transaction can match. Providers signal absence either with `null`
    // or with an error dialect like Tenderly's `-32001 "not found"`; both
    // must map to `None`.
    let absent_nonce = Nonce(nonce.saturating_add(1_000_000));
    let result = match tx_hash_on_l1_for_nonce(&provider, sender, absent_nonce).await {
        Ok(result) => result,
        Err(error) => panic!(
            "{L1_RPC_ENV} rejected eth_getTransactionBySenderAndNonce for absent nonce \
             {absent_nonce}: {error:?}"
        ),
    };
    assert_eq!(
        result, None,
        "Expected no tx hash for absent nonce {} through {}",
        absent_nonce, L1_RPC_ENV,
    );

    println!("External L1 RPC sender+nonce lookup validated");
}
