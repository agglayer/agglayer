use super::{limiter, state, LocalRateLimiter, RateLimited, RateLimiter, RawState};

/// A component of a rate limiter.
///
/// Each rate limiter may have multiple components. For example, if we are
/// limiting transaction settlement rate and peanut rate, there will be two type
/// tags corresponding to the two components, i.e. [`component::SendTx`] and
/// (hypothetical) `component::PeanutRate`, each implementing [Component].
pub trait Component {
    /// Time measure for this limiter component.
    type Instant: Copy + Ord;

    /// Information in case of rate limit being reached.
    type LimitedInfo;

    /// Internal state type for this component.
    type State: RawState<Instant = Self::Instant, LimitedInfo = Self::LimitedInfo>;

    /// Check various preconditions before the rate limiter proper is applied.
    fn precondition(_limiter: &mut LocalRateLimiter) -> Result<(), RateLimited> {
        Ok(())
    }

    /// Get the rate limiter for this component.
    fn component(limiter: &mut LocalRateLimiter) -> &mut RateLimiter<Self::State>;

    /// Convert the component-specific error into the rate limited error.
    fn error(info: limiter::RateLimited<Self::LimitedInfo>) -> RateLimited;
}

/// The rate limiter component for `sendTx` settlement.
pub enum SendTx {}

impl Component for SendTx {
    type Instant = tokio::time::Instant;
    type LimitedInfo = state::WallClockLimitedInfo;
    type State = state::WallClockState;

    fn component(limiter: &mut LocalRateLimiter) -> &mut RateLimiter<Self::State> {
        &mut limiter.send_tx
    }

    fn error(info: limiter::RateLimited<Self::LimitedInfo>) -> RateLimited {
        RateLimited::send_tx(info)
    }
}
