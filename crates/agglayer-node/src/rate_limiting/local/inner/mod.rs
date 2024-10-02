use agglayer_config::rate_limiting::{NetworkRateLimitingConfig, TimeRateLimit};

use super::{
    limiter::{self, RateLimiter, SlotTracker},
    state::{self, RawState, WallClockLimitedInfo, WallClockState},
};

pub mod component;

pub use component::Component;

/// Rate limiter state for single network / rollup without synchronization.
pub struct LocalRateLimiter {
    /// Rate limiter for `sendTx` settlement.
    send_tx: RateLimiter<WallClockState>,
}

impl LocalRateLimiter {
    pub fn from_config(config: &NetworkRateLimitingConfig) -> Self {
        let send_tx = match config.send_tx {
            TimeRateLimit::Unlimited => RateLimiter::Unlimited,

            TimeRateLimit::Limited {
                max_per_interval,
                time_interval,
            } => std::num::NonZeroU32::new(*max_per_interval).map_or(
                RateLimiter::Disabled,
                |max_per_interval| {
                    RateLimiter::limited(WallClockState::new(max_per_interval, *time_interval))
                },
            ),
        };

        LocalRateLimiter { send_tx }
    }

    pub fn reserve<C: Component>(&mut self, time: C::Instant) -> Result<SlotTracker, RateLimited> {
        C::precondition(self)?;
        C::component(self).reserve(time).map_err(C::error)
    }

    pub fn release<C: Component>(&mut self, slot: SlotTracker) {
        C::component(self).release(slot)
    }

    pub fn record<C: Component>(&mut self, time: C::Instant, slot: SlotTracker) {
        C::component(self).record(time, slot)
    }
}

#[derive(PartialEq, Eq, Clone, Debug, serde::Serialize, thiserror::Error)]
#[serde(rename_all = "kebab-case")]
pub enum RateLimited {
    #[error("The `sendTx` settlement has been limited: {0}")]
    SendTxRateLimited(WallClockLimitedInfo),

    #[error("The `sendTx` settlement disabled by rate limiter")]
    SendTxDiabled {},
}

impl RateLimited {
    fn send_tx(err: limiter::RateLimited<WallClockLimitedInfo>) -> Self {
        match err {
            limiter::RateLimited::Disabled {} => Self::SendTxDiabled {},
            limiter::RateLimited::Inner(err) => Self::SendTxRateLimited(err),
        }
    }
}
