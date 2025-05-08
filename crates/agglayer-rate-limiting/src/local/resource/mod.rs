use super::{limiter, state, NetworkRateLimitingConfig};

mod certificate_settlement;
mod send_tx_settlement;

pub use certificate_settlement::{
    CertificateSettlement, RawLimitedInfo as CertificateSettlementRawLimitedInfo,
    SendCertificateRateLimited,
};
pub use send_tx_settlement::{
    RawLimitedInfo as SendTxSettlementRawLimitedInfo, SendTxRateLimited, SendTxSettlement,
};

pub type RawLimitedInfoFor<R> =
    limiter::RateLimited<<<R as Resource>::State as state::RawState>::LimitedInfo>;

/// Defines resources to which we apply rate limiting.
pub trait Resource {
    /// How the resource represents the time. Commonly wall clock time or epoch number.
    type Instant;

    /// The internal state tracker for this resource.
    type State: state::RawState<Instant = Self::Instant>;

    /// If rate limit is hit, what information is presented.
    type LimitedInfo: From<RawLimitedInfoFor<Self>>;
}

/// A resource rate limiter that can be initialized based on config.
pub trait ConfigurableResource: Resource + Sized {
    fn from_config(config: &NetworkRateLimitingConfig) -> limiter::RateLimiter<Self::State>;
}
