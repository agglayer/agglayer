use std::{future::Future, time::Duration};

use agglayer_config::settlement_service::TxRetryPolicy;
use agglayer_types::SettlementTxHash;
use alloy::{
    network::TransactionResponse as _,
    primitives::Address,
    providers::Provider,
    transports::{
        layers::{RateLimitRetryPolicy, RetryPolicy},
        TransportError, TransportResult,
    },
};
use rand::Rng as _;
use tokio_util::sync::CancellationToken;
use tracing::warn;

use crate::settlement_task::Nonce;

#[derive(Debug)]
pub(crate) enum RetryCallbackError<E> {
    Error(E),
    Cancelled,
}

/// Calls `callback` until it succeeds.
///
/// Transient errors are retried using the provided policy. Permanent errors are
/// returned immediately.
pub(crate) async fn retry_callback_until_success<T, E, F, Fut, I>(
    policy: &TxRetryPolicy,
    cancellation_token: &CancellationToken,
    mut callback: F,
    mut is_transient: I,
) -> Result<T, RetryCallbackError<E>>
where
    E: std::fmt::Debug,
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    I: FnMut(&E) -> bool,
{
    let mut next_interval = policy.initial_interval;
    let mut retry_attempt = 0_u64;

    loop {
        match callback().await {
            Ok(value) => return Ok(value),
            Err(error) => {
                if !is_transient(&error) {
                    return Err(RetryCallbackError::Error(error));
                }

                retry_attempt = retry_attempt.saturating_add(1);
                let sleep_duration = next_interval.saturating_add(random_jitter(policy.jitter));
                warn!(
                    ?error,
                    retry_attempt,
                    ?sleep_duration,
                    "Transient error while executing retryable callback"
                );

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
    )
    .await
}

fn is_transient_alloy_error(error: &TransportError) -> bool {
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
    let result = provider
        .get_transaction_by_sender_nonce(wallet, nonce.0)
        .await?;
    let Some(tx) = result else {
        return Ok(None);
    };
    Ok(tx
        .block_number()
        .is_some()
        .then(|| SettlementTxHash::from(tx.tx_hash())))
}

#[cfg(test)]
mod tests {
    use std::{
        borrow::Cow,
        error::Error,
        fmt::{Display, Formatter},
        sync::{
            atomic::{AtomicUsize, Ordering},
            Arc, Mutex,
        },
        time::Duration,
    };

    use agglayer_config::Multiplier;
    use alloy::{
        network::EthereumWallet,
        node_bindings::{Anvil, AnvilInstance},
        primitives::U256,
        providers::ProviderBuilder,
        rpc::types::TransactionRequest,
        signers::local::PrivateKeySigner,
        transports::{RpcError, TransportError},
    };
    use tokio::time::{advance, Instant};
    use tokio_util::sync::CancellationToken;

    use super::*;

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

    fn build_provider(anvil: &AnvilInstance) -> impl Provider {
        let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
        ProviderBuilder::new()
            .wallet(EthereumWallet::from(signer))
            .connect_http(anvil.endpoint_url())
    }

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
        )
        .await
        .unwrap();

        assert_eq!(value, 42);
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
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

        let result = tx_hash_on_l1_for_nonce(&provider, sender, Nonce(0))
            .await
            .unwrap();
        assert_eq!(
            result,
            Some(SettlementTxHash::from(receipt.transaction_hash))
        );
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
}
