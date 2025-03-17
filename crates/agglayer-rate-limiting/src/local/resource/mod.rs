use super::{limiter, state};

mod send_tx_settlement;

pub use send_tx_settlement::{SendTxRateLimited, SendTxSettlement};

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
pub trait ConfigurableResource: Resource {}
