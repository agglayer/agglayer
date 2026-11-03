use std::{convert::Infallible, future::Future, ops::ControlFlow, time::Duration};

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
use tracing::{debug, error, warn, Level};

#[derive(Debug)]
pub(crate) enum RetryCallbackError<E> {
    Error(E),
    Cancelled,
}

/// Number of consecutive failed retries after which every further retry
/// logged at debug level is logged at warning level instead, so a retry loop
/// stuck on a "keep polling" signal still surfaces in the default logs. With
/// the default non-inclusion policy (60s doubling up to 10min intervals) this
/// only triggers roughly 10 hours into a stuck wait, far beyond any healthy
/// inclusion or finality delay.
const FORCE_WARNING_AFTER_RETRIES: u64 = 64;

/// Number of consecutive warning-level retries after which they are logged at
/// error level instead, so a single blip stays quiet but an L1 that keeps
/// failing stands out.
const ESCALATE_TO_ERROR_AFTER_RETRIES: u64 = 5;

/// Calls `callback` until it succeeds.
///
/// Transient errors are retried using the provided policy. Permanent errors are
/// returned immediately.
///
/// `log_level` picks how each retried error is logged. It never decides
/// whether to retry:
/// - `ERROR`: a definite answer (see [`is_definite_answer`]), which very likely
///   means a misconfigured endpoint, logged at error level from the first
///   retry.
/// - `WARN`: an anomaly that may clear on its own, logged at warning level,
///   then at error level once [`ESCALATE_TO_ERROR_AFTER_RETRIES`] retries in a
///   row have failed with one.
/// - Any other level (`DEBUG`): an expected "keep polling" signal, logged at
///   debug level, then at warning level after [`FORCE_WARNING_AFTER_RETRIES`]
///   retries.
///
/// Cancellation is observed both before and during each callback invocation, so
/// an already-cancelled token never starts a new callback and a pending
/// callback (for example a stalled request) is abandoned promptly.
pub(crate) async fn retry_callback_until_success<T, E, F, Fut, I, L>(
    policy: &TxRetryPolicy,
    cancellation_token: &CancellationToken,
    callback: F,
    mut is_transient: I,
    log_level: L,
) -> Result<T, RetryCallbackError<E>>
where
    E: std::fmt::Debug,
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    I: FnMut(&E) -> bool,
    L: FnMut(&E) -> Level,
{
    retry_loop(
        policy,
        cancellation_token,
        callback,
        |error| {
            if is_transient(&error) {
                ControlFlow::Continue(error)
            } else {
                ControlFlow::Break(error)
            }
        },
        log_level,
    )
    .await
}

/// Calls `callback` until it succeeds, retrying every error.
///
/// Only cancellation ends the loop early, and the error type says so: no
/// error can be returned. Use it for idempotent calls such as L1 reads, where
/// no error, however unexpected, is a reason to give up.
///
/// Logging and cancellation behave as in [`retry_callback_until_success`].
pub(crate) async fn retry_every_error_until_success<T, E, F, Fut, L>(
    policy: &TxRetryPolicy,
    cancellation_token: &CancellationToken,
    callback: F,
    log_level: L,
) -> Result<T, RetryCallbackError<Infallible>>
where
    E: std::fmt::Debug,
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    L: FnMut(&E) -> Level,
{
    retry_loop(
        policy,
        cancellation_token,
        callback,
        ControlFlow::Continue,
        log_level,
    )
    .await
}

/// Shared retry loop. `retryable` continues with the error to retry it, or
/// breaks with the error to return.
async fn retry_loop<T, E, P, F, Fut, R, L>(
    policy: &TxRetryPolicy,
    cancellation_token: &CancellationToken,
    mut callback: F,
    mut retryable: R,
    mut log_level: L,
) -> Result<T, RetryCallbackError<P>>
where
    E: std::fmt::Debug,
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    R: FnMut(E) -> ControlFlow<P, E>,
    L: FnMut(&E) -> Level,
{
    let mut next_interval = policy.initial_interval;
    let mut retry_attempt = 0_u64;
    let mut anomaly_streak = 0_u64;

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
                let error = match retryable(error) {
                    ControlFlow::Continue(error) => error,
                    ControlFlow::Break(error) => return Err(RetryCallbackError::Error(error)),
                };

                retry_attempt = retry_attempt.saturating_add(1);
                let sleep_duration = next_interval.saturating_add(random_jitter(policy.jitter));
                let level = log_level(&error);
                if level == Level::ERROR {
                    anomaly_streak = anomaly_streak.saturating_add(1);
                    error!(
                        ?error,
                        retry_attempt,
                        ?sleep_duration,
                        "L1 endpoint gave a definite error answer, likely misconfigured; retrying"
                    );
                } else if level == Level::WARN {
                    anomaly_streak = anomaly_streak.saturating_add(1);
                    if anomaly_streak >= ESCALATE_TO_ERROR_AFTER_RETRIES {
                        error!(
                            ?error,
                            retry_attempt,
                            ?sleep_duration,
                            "Error while executing retryable callback"
                        );
                    } else {
                        warn!(
                            ?error,
                            retry_attempt,
                            ?sleep_duration,
                            "Transient error while executing retryable callback"
                        );
                    }
                } else {
                    anomaly_streak = 0;
                    if retry_attempt >= FORCE_WARNING_AFTER_RETRIES {
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

/// Whether `error` is a definite answer from the L1 endpoint, the same for
/// every request: an HTTP 4xx other than 408 and 429, or a JSON-RPC
/// invalid-request, method-not-found or invalid-params error. These codes are
/// standardized, unlike vendor server errors. It only picks the log level,
/// never whether to retry.
pub(crate) fn is_definite_answer(error: &TransportError) -> bool {
    if let Some(status) = error
        .as_transport_err()
        .and_then(|error| error.as_http_error())
        .map(|error| error.status)
    {
        return (400..500).contains(&status) && !matches!(status, 408 | 429);
    }

    // -32602 invalid params, -32601 method not found, -32600 invalid request.
    error
        .as_error_resp()
        .is_some_and(|error| matches!(error.code, -32602..=-32600))
}

/// Log level for a retried L1 RPC error: error for a definite answer,
/// warning otherwise.
pub(crate) fn transport_log_level(error: &TransportError) -> Level {
    if is_definite_answer(error) {
        Level::ERROR
    } else {
        Level::WARN
    }
}

/// Calls an alloy callback until it succeeds, retrying on transient transport
/// and JSON-RPC errors.
///
/// Only for calls where an error can be a definite answer: the settlement
/// broadcast and gas estimation. L1 reads use
/// [`retry_every_error_until_success`] instead, since a read is always safe
/// to repeat and an unexpected error must never end it.
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
        transport_log_level,
    )
    .await
}

/// Allowlist of errors known to be transient. It is incomplete by nature, so
/// it only drives [`retry_alloy_callback_until_success`].
fn is_transient_alloy_error(error: &TransportError) -> bool {
    RateLimitRetryPolicy::default().should_retry(error) || is_request_timeout_error(error)
}

fn is_request_timeout_error(error: &TransportError) -> bool {
    // HTTP 408 and 504 have standardized timeout semantics, unlike JSON-RPC
    // server error codes, so they are sufficient without inspecting the body.
    if error
        .as_transport_err()
        .and_then(|error| error.as_http_error())
        .is_some_and(|error| matches!(error.status, 408 | 504))
    {
        return true;
    }

    // -32009 is in JSON-RPC's implementation-defined server-error range and
    // has non-timeout meanings in some clients. Require timeout semantics too
    // when a provider returns it as a regular JSON-RPC error response.
    error.as_error_resp().is_some_and(|error| {
        error.code == -32009 && message_describes_timeout(error.message.as_ref())
    })
}

fn message_describes_timeout(message: &str) -> bool {
    let mut previous_word_describes_time = false;

    for word in message
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
    {
        if word.eq_ignore_ascii_case("timeout") || word.eq_ignore_ascii_case("timedout") {
            return true;
        }

        if previous_word_describes_time && word.eq_ignore_ascii_case("out") {
            return true;
        }

        previous_word_describes_time =
            word.eq_ignore_ascii_case("time") || word.eq_ignore_ascii_case("timed");
    }

    false
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
        // dialect maps to `None`; any other error response is returned, so
        // callers retry it with a warning rather than read it as "not
        // included yet".
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
mod tests;
