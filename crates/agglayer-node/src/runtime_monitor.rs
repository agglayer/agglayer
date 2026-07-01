//! Tokio runtime responsiveness probe.
//!
//! A blocked worker thread -- for example a synchronous RocksDB call running on
//! an async thread (see `agglayer_storage`) -- prevents otherwise-ready tasks
//! from being polled. Even a trivial handler such as `/health` then appears to
//! "hang". This probe measures how late a fixed-interval timer fires; the
//! overshoot approximates how long the scheduler was unable to make progress.
//!
//! It is intentionally cheap and requires no `tokio_unstable` build flags. For
//! per-task attribution (which future blocked the worker) reach for
//! `tokio-metrics`/`tokio-console`, which do require `tokio_unstable`.

use std::time::Duration;

use tokio::time::Instant;
use tokio_util::sync::CancellationToken;
use tracing::warn;

/// Computes how much later than `interval` a tick actually fired.
///
/// Returns [`Duration::ZERO`] when the tick was on time or early, so a healthy
/// scheduler reports approximately zero lag.
fn overshoot(observed: Duration, interval: Duration) -> Duration {
    observed.saturating_sub(interval)
}

/// Runs the scheduler-lag probe until `cancellation` is triggered.
///
/// Spawn this on the runtime you want to observe; it reads metrics from
/// [`tokio::runtime::Handle::current`]. `interval` and `warn_threshold` come
/// from configuration (see `agglayer_config::TelemetryConfig`).
pub(crate) async fn run(
    interval: Duration,
    warn_threshold: Duration,
    cancellation: CancellationToken,
) {
    let handle = tokio::runtime::Handle::current();
    let mut last = Instant::now();

    loop {
        tokio::select! {
            biased;
            _ = cancellation.cancelled() => break,
            _ = tokio::time::sleep(interval) => {}
        }

        let now = Instant::now();
        let lag = overshoot(now.duration_since(last), interval);
        last = now;

        let lag_ms = lag.as_secs_f64() * 1_000.0;
        agglayer_telemetry::runtime::SCHEDULER_LAG_MS.record(lag_ms, &[]);
        agglayer_telemetry::runtime::GLOBAL_QUEUE_DEPTH
            .record(handle.metrics().global_queue_depth() as u64, &[]);

        if lag >= warn_threshold {
            warn!(
                lag_ms = lag_ms,
                "Tokio scheduler lag spike; worker threads may be blocked (e.g. blocking I/O on \
                 an async thread)"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overshoot_is_zero_when_on_time_or_early() {
        let interval = Duration::from_millis(500);
        assert_eq!(overshoot(interval, interval), Duration::ZERO);
        assert_eq!(
            overshoot(Duration::from_millis(450), interval),
            Duration::ZERO
        );
    }

    #[test]
    fn overshoot_reports_lateness() {
        let interval = Duration::from_millis(500);
        assert_eq!(
            overshoot(Duration::from_millis(620), interval),
            Duration::from_millis(120)
        );
    }
}
