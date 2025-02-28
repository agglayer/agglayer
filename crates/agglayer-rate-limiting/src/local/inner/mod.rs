use agglayer_config::rate_limiting::{EpochRateLimit, NetworkRateLimitingConfig, TimeRateLimit};

use super::{
    limiter::{self, RateLimiter, SlotTracker},
    state::{
        self, PerEpochLimitedInfo, PerEpochState, RawState, WallClockLimitedInfo, WallClockState,
    },
};

pub mod component;

pub use component::Component;

/// Rate limiter state for single network / rollup without synchronization.
pub struct LocalRateLimiter {
    /// Rate limiter for `sendTx` settlement.
    send_tx: RateLimiter<WallClockState>,

    /// Rate limiter for `sendCertificate` settlement.
    send_certificate: RateLimiter<PerEpochState>,
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

        let send_certificate = match config.send_certificate {
            EpochRateLimit::Unlimited => RateLimiter::Unlimited,

            EpochRateLimit::Limited { max_per_epoch } => std::num::NonZeroU32::new(*max_per_epoch)
                .map_or(RateLimiter::Disabled, |max_per_epoch| {
                    RateLimiter::limited(PerEpochState::new(max_per_epoch))
                }),
        };

        LocalRateLimiter {
            send_tx,
            send_certificate,
        }
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
    #[error("The `sendTx` settlement has been limited")]
    SendTxRateLimited(#[source] WallClockLimitedInfo),

    #[error("The `sendTx` settlement disabled by rate limiter")]
    SendTxDiabled {},

    #[error("The `sendCertificate` settlement has been rate limited")]
    SendCertificateRateLimited(#[source] PerEpochLimitedInfo),

    #[error("The `sendCertificate` settlement disabled by rate limiter")]
    SendCertificateDisabled {},
}

impl RateLimited {
    fn send_tx(err: limiter::RateLimited<WallClockLimitedInfo>) -> Self {
        match err {
            limiter::RateLimited::Disabled {} => Self::SendTxDiabled {},
            limiter::RateLimited::Inner(err) => Self::SendTxRateLimited(err),
        }
    }

    fn send_certificate(err: limiter::RateLimited<PerEpochLimitedInfo>) -> Self {
        match err {
            limiter::RateLimited::Disabled {} => Self::SendCertificateDisabled {},
            limiter::RateLimited::Inner(err) => Self::SendCertificateRateLimited(err),
        }
    }
}
