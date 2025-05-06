use agglayer_config::rate_limiting::{NetworkRateLimitingConfig, TimeRateLimit};

pub use super::state::WallClockLimitedInfo as RawLimitedInfo;
use super::{limiter, state::WallClockState, ConfigurableResource, Resource};

pub enum SendTxSettlement {}

impl Resource for SendTxSettlement {
    type Instant = tokio::time::Instant;

    type State = WallClockState;

    type LimitedInfo = SendTxRateLimited;
}

#[derive(PartialEq, Eq, Clone, Debug, serde::Serialize, thiserror::Error)]
#[serde(rename_all = "kebab-case")]
pub enum SendTxRateLimited {
    #[error("The `sendTx` settlement has been limited")]
    SendTxRateLimited(#[source] RawLimitedInfo),

    #[error("The `sendTx` settlement disabled by rate limiter")]
    SendTxDiabled {},
}

impl From<limiter::RateLimited<RawLimitedInfo>> for SendTxRateLimited {
    fn from(info: limiter::RateLimited<RawLimitedInfo>) -> Self {
        match info {
            limiter::RateLimited::Disabled {} => Self::SendTxDiabled {},
            limiter::RateLimited::Inner(err) => Self::SendTxRateLimited(err),
        }
    }
}

impl ConfigurableResource for SendTxSettlement {
    fn from_config(config: &NetworkRateLimitingConfig) -> limiter::RateLimiter<Self::State> {
        match config.send_tx {
            TimeRateLimit::Unlimited => limiter::RateLimiter::Unlimited,
            TimeRateLimit::Limited {
                max_per_interval,
                time_interval,
            } => std::num::NonZeroU32::new(*max_per_interval).map_or(
                limiter::RateLimiter::Disabled,
                |max_per_interval| {
                    limiter::RateLimiter::limited(WallClockState::new(
                        max_per_interval,
                        *time_interval,
                    ))
                },
            ),
        }
    }
}
