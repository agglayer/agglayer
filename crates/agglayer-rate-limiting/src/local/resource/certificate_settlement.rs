use agglayer_config::rate_limiting::{EpochRateLimit, NetworkRateLimitingConfig};

pub use super::state::PerEpochLimitedInfo as RawLimitedInfo;
use super::{limiter, state::PerEpochState, ConfigurableResource, Resource};

pub enum CertificateSettlement {}

impl Resource for CertificateSettlement {
    type Instant = agglayer_types::EpochNumber;

    type State = PerEpochState;

    type LimitedInfo = SendCertificateRateLimited;
}

#[derive(PartialEq, Eq, Clone, Debug, serde::Serialize, thiserror::Error)]
#[serde(rename_all = "kebab-case")]
pub enum SendCertificateRateLimited {
    #[error("The certificate settlement has been rate limited")]
    RateLimited(#[source] RawLimitedInfo),

    #[error("The certificate settlement disabled by rate limiter")]
    Diabled {},
}

impl From<limiter::RateLimited<RawLimitedInfo>> for SendCertificateRateLimited {
    fn from(info: limiter::RateLimited<RawLimitedInfo>) -> Self {
        match info {
            limiter::RateLimited::Disabled {} => Self::Diabled {},
            limiter::RateLimited::Inner(err) => Self::RateLimited(err),
        }
    }
}

impl ConfigurableResource for CertificateSettlement {
    fn from_config(config: &NetworkRateLimitingConfig) -> limiter::RateLimiter<Self::State> {
        match config.send_certificate {
            EpochRateLimit::Unlimited => limiter::RateLimiter::Unlimited,
            EpochRateLimit::Limited { max_per_epoch } => std::num::NonZeroU32::new(*max_per_epoch)
                .map_or(limiter::RateLimiter::Disabled, |max_per_epoch| {
                    limiter::RateLimiter::limited(PerEpochState::new(max_per_epoch))
                }),
        }
    }
}
