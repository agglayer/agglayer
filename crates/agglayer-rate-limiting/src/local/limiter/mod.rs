use agglayer_utils::log_assert;

use super::state::{self, RawState};

mod core;
mod slot_tracker;

pub use self::{core::RateLimiterCore, slot_tracker::SlotTracker};

/// Single network single component rate limiter.
///
/// This rate limiter handles edge cases, namely if no rate limit is imposed or
/// if rate limit is zero. Otherwise, it delegates to the inner limiter.
pub enum RateLimiter<S> {
    /// The request is disallowed completely.
    Disabled,

    /// Delegate to the inner rate limiter.
    Limited(RateLimiterCore<S>),

    /// No rate limit imposed, requests can be as frequent as desired.
    Unlimited,
}

impl<S: RawState> RateLimiter<S> {
    /// Create a new non-trivial rate limiter.
    pub fn limited(raw_state: S) -> Self {
        Self::Limited(RateLimiterCore::new(raw_state))
    }

    /// Reserve a rate limiting slot.
    pub fn reserve(
        &mut self,
        time: S::Instant,
    ) -> Result<SlotTracker, RateLimited<S::LimitedInfo>> {
        match self {
            Self::Disabled => Err(RateLimited::Disabled {}),
            Self::Limited(inner) => inner.reserve(time).map_err(RateLimited::Inner),
            Self::Unlimited => Ok(SlotTracker::new()),
        }
    }

    /// Release a rate limiting slot.
    pub fn release(&mut self, slot: SlotTracker) {
        match self {
            Self::Limited(inner) => inner.release(slot),
            Self::Unlimited => drop(slot.release()),
            Self::Disabled => {
                log_assert!(slot.release() == 0, "Event in disabled rate limiter");
            }
        }
    }

    /// Record a rate limiting event.
    pub fn record(&mut self, time: S::Instant, slot: SlotTracker) {
        match self {
            Self::Limited(inner) => inner.record(time, slot),
            Self::Unlimited => drop(slot.release()),
            Self::Disabled => {
                log_assert!(slot.release() == 0, "Event in disabled rate limiter");
            }
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum RateLimited<E> {
    Disabled {},
    Inner(E),
}
