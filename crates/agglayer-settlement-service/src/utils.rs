use std::{future::Future, time::Duration};

use agglayer_config::settlement_service::TxRetryPolicy;
use agglayer_types::{ContractCallOutcome, ContractCallResult, Nonce, SettlementTxHash};
use alloy::{
    network::{ReceiptResponse, TransactionResponse as _},
    primitives::{Address, Bytes},
    providers::Provider,
    transports::{
        layers::{RateLimitRetryPolicy, RetryPolicy},
        TransportError, TransportResult,
    },
};
use rand::Rng as _;
use tokio_util::sync::CancellationToken;
use tracing::{debug, warn};

#[derive(Debug)]
pub(crate) enum RetryCallbackError<E> {
    Error(E),
    Cancelled,
}

/// Number of consecutive failed retries after which every further retry is
/// logged at warning level even when `needs_warning_log` says debug, so a
/// retry loop stuck on an expected "keep polling" signal still surfaces in the
/// default logs. With the default non-inclusion policy (60s doubling up to
/// 10min intervals) this only triggers roughly 10 hours into a stuck wait, far
/// beyond any healthy inclusion or finality delay.
const FORCE_WARNING_AFTER_RETRIES: u64 = 64;

/// Calls `callback` until it succeeds.
///
/// Transient errors are retried using the provided policy. Permanent errors are
/// returned immediately.
///
/// Each retried error is logged at warning level, unless `needs_warning_log`
/// returns `false` for it — an expected "keep polling" signal rather than an
/// anomaly — in which case it is only logged at debug level. Once
/// [`FORCE_WARNING_AFTER_RETRIES`] consecutive retries have failed, every
/// further retry is logged at warning level regardless.
///
/// Cancellation is observed both before and during each callback invocation, so
/// an already-cancelled token never starts a new callback and a pending
/// callback (for example a stalled request) is abandoned promptly.
pub(crate) async fn retry_callback_until_success<T, E, F, Fut, I, W>(
    policy: &TxRetryPolicy,
    cancellation_token: &CancellationToken,
    mut callback: F,
    mut is_transient: I,
    mut needs_warning_log: W,
) -> Result<T, RetryCallbackError<E>>
where
    E: std::fmt::Debug,
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    I: FnMut(&E) -> bool,
    W: FnMut(&E) -> bool,
{
    let mut next_interval = policy.initial_interval;
    let mut retry_attempt = 0_u64;

    loop {
        // Race the callback against cancellation so a shutdown is observed even
        // while the callback future is still pending; `biased` also returns
        // before the callback is polled when the token is already cancelled.
        let outcome = tokio::select! {
            biased;
            _ = cancellation_token.cancelled() => return Err(RetryCallbackError::Cancelled),
            outcome = callback() => outcome,
        };
        match outcome {
            Ok(value) => return Ok(value),
            Err(error) => {
                if !is_transient(&error) {
                    return Err(RetryCallbackError::Error(error));
                }

                retry_attempt = retry_attempt.saturating_add(1);
                let sleep_duration = next_interval.saturating_add(random_jitter(policy.jitter));
                if needs_warning_log(&error) || retry_attempt >= FORCE_WARNING_AFTER_RETRIES {
                    warn!(
                        ?error,
                        retry_attempt,
                        ?sleep_duration,
                        "Transient error while executing retryable callback"
                    );
                } else {
                    debug!(
                        ?error,
                        retry_attempt,
                        ?sleep_duration,
                        "Transient error while executing retryable callback"
                    );
                }

                tokio::select! {
                    biased;
                    _ = cancellation_token.cancelled() => {
                        return Err(RetryCallbackError::Cancelled);
                    }
                    _ = tokio::time::sleep(sleep_duration) => {}
                }

                next_interval = policy
                    .interval_multiplier_factor
                    .saturating_mul_duration(next_interval)
                    .min(policy.max_interval);
            }
        }
    }
}

/// Calls an alloy callback until it succeeds, retrying only on the retryable
/// transport and JSON-RPC errors recognized by alloy itself.
pub(crate) async fn retry_alloy_callback_until_success<T, F, Fut>(
    policy: &TxRetryPolicy,
    cancellation_token: &CancellationToken,
    callback: F,
) -> Result<T, RetryCallbackError<TransportError>>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = TransportResult<T>>,
{
    retry_callback_until_success(
        policy,
        cancellation_token,
        callback,
        is_transient_alloy_error,
        |_| true,
    )
    .await
}

pub(crate) fn is_transient_alloy_error(error: &TransportError) -> bool {
    RateLimitRetryPolicy::default().should_retry(error)
}

fn random_jitter(max_jitter: Duration) -> Duration {
    if max_jitter.is_zero() {
        return Duration::ZERO;
    }

    let max_jitter_millis = max_jitter.as_millis().try_into().unwrap_or(u64::MAX);
    Duration::from_millis(rand::rng().random_range(0..=max_jitter_millis))
}

/// Returns the [`SettlementTxHash`] for a mined transaction matching the
/// given wallet and nonce, or `None` if no such mined transaction exists.
///
/// Mempool-only transactions are ignored.
pub(crate) async fn tx_hash_on_l1_for_nonce(
    provider: &impl Provider,
    wallet: Address,
    nonce: Nonce,
) -> TransportResult<Option<SettlementTxHash>> {
    let tx = match provider
        .get_transaction_by_sender_nonce(wallet, nonce.0)
        .await
    {
        Ok(Some(tx)) => tx,
        Ok(None) => return Ok(None),
        // Tenderly Gateway (the L1 RPC of current deployments) answers
        // `-32001 "not found"` instead of the `null` other nodes return when
        // no transaction matches the sender and nonce. Only this exact
        // dialect maps to `None`; any other error response keeps failing
        // loudly rather than being silently read as "not included yet".
        Err(TransportError::ErrorResp(error))
            if error.code == -32001 && error.message == "not found" =>
        {
            return Ok(None);
        }
        Err(error) => return Err(error),
    };
    Ok(tx
        .block_number()
        .is_some()
        .then(|| SettlementTxHash::from(tx.tx_hash())))
}

/// Builds the [`ContractCallResult`] for a mined transaction receipt, or
/// `None` if the receipt has no block info yet.
///
/// The metadata (return data or revert reason) is not available in receipts,
/// so it is left empty.
pub(crate) fn contract_call_result_from_receipt(
    receipt: &impl ReceiptResponse,
) -> Option<ContractCallResult> {
    let block_hash = receipt.block_hash()?;
    let block_number = receipt.block_number()?;

    let succeeded = receipt.status();
    // Test-only failpoint: force the settlement tx to look reverted so the run
    // loop finalizes the job as a revert. Compiled out of production builds.
    #[cfg(feature = "testutils")]
    let succeeded = succeeded && !fail::eval("settlement::force_revert", |_| true).unwrap_or(false);

    Some(ContractCallResult {
        outcome: if succeeded {
            ContractCallOutcome::Success
        } else {
            ContractCallOutcome::Revert
        },
        metadata: Bytes::new(),
        block_hash,
        block_number,
        tx_hash: SettlementTxHash::from(receipt.transaction_hash()),
    })
}

/// Builds an Anvil-backed L1 provider signing with its first funded account.
#[cfg(test)]
pub(crate) fn build_provider(
    anvil: &alloy::node_bindings::AnvilInstance,
) -> impl Provider + alloy::providers::WalletProvider + 'static {
    use alloy::{
        network::EthereumWallet, providers::ProviderBuilder, signers::local::PrivateKeySigner,
    };

    let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
    ProviderBuilder::new()
        .wallet(EthereumWallet::from(signer))
        .connect_http(anvil.endpoint_url())
}

#[cfg(test)]
mod tests {
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
        transports::{RpcError, TransportError},
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
                let error: TransportError =
                    RpcError::ErrorResp(alloy::rpc::json_rpc::ErrorPayload {
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
                "No mined transactions found in the last {} blocks through {}; submit at least \
                 one transaction and retry",
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
}
