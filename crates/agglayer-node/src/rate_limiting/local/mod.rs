//! Local rate limiter, i.e. one constraining a single network.

use agglayer_config::rate_limiting::{NetworkRateLimitingConfig, TimeRateLimit};
use tokio::time::Instant;

mod interface;
pub mod trivial;
pub mod wall_clock;

pub use interface::RateLimiter;

/// Rate limiter state for single network / rollup.
pub struct LocalRateLimiter {
    /// Rate limiter for `sendTx` settlement.
    send_tx: trivial::RateLimiter<wall_clock::RateLimiter>,
}

impl LocalRateLimiter {
    pub fn from_config(config: &NetworkRateLimitingConfig) -> Self {
        let send_tx = match config.send_tx {
            TimeRateLimit::Unlimited => trivial::RateLimiter::Unlimited,
            TimeRateLimit::Limited {
                max_per_interval,
                time_interval,
            } => std::num::NonZeroU32::new(*max_per_interval).map_or(
                trivial::RateLimiter::Disabled,
                |max_per_interval| {
                    let inner = wall_clock::RateLimiter::new(max_per_interval, *time_interval);
                    trivial::RateLimiter::Limited(inner)
                },
            ),
        };

        LocalRateLimiter { send_tx }
    }

    pub fn check_send_tx(&mut self, time: Instant) -> Result<(), RateLimited> {
        self.send_tx.check(time).map_err(RateLimited::send_tx)
    }

    pub fn limit_send_tx(&mut self, time: Instant) -> Result<(), RateLimited> {
        self.send_tx.limit(time).map_err(RateLimited::send_tx)
    }
}

#[derive(PartialEq, Eq, Clone, Debug, serde::Serialize, thiserror::Error)]
#[serde(rename_all = "kebab-case")]
pub enum RateLimited {
    #[error("The `sendTx` settlement has been limited: {0}")]
    SendTxRateLimited(wall_clock::RateLimited),

    #[error("The `sendTx` settlement disabled by rate limiter")]
    SendTxDiabled {},
}

impl RateLimited {
    fn send_tx(err: trivial::RateLimited<wall_clock::RateLimited>) -> Self {
        match err {
            trivial::RateLimited::Disabled {} => Self::SendTxDiabled {},
            trivial::RateLimited::Inner(err) => Self::SendTxRateLimited(err),
        }
    }
}
