//! Local rate limiter, i.e. one constraining a single network.
//!
//! A rate limiter consists of multiple components, one per each class of events
//! being limited. For example, [component::SendTx] takes care of limiting the
//! rate of `sendTx` settlement on L1. More components can be added as needed.
//!
//! The implementation is defined in multiple layers, each adding some features.
//! The layers correspond to modules as follows:
//!
//! * [state] defines the internal state of a limiter component.
//! * [limiter] implements rate limiting for single limiter component. It adds
//!   the ability to reserve a slot which can be committed later,chandling of
//!   trivial cases (e.g. no limit), and concurrency control.
//! * [inner] is effectively a collection of limiter components for one network.
//!   It defines what a [Component] is and bundles them into a struct.
//! * [self] (this module) defines the top-level [LocalRateLimiter]. It takes
//!   care of cross-thread synchronization and provides a safe interface to rate
//!   limiter using a [SlotGuard].

use std::sync::Arc;

use agglayer_config::rate_limiting::NetworkRateLimitingConfig;
use parking_lot::Mutex;

mod inner;
mod limiter;
mod slot_guard;
mod state;

pub use inner::{component, Component, RateLimited};
pub use slot_guard::SlotGuard;

/// Rate limiter state for single network / rollup.
pub struct LocalRateLimiter(Arc<Mutex<inner::LocalRateLimiter>>);

impl LocalRateLimiter {
    /// Create a rate limiter form configuration.
    pub fn from_config(config: &NetworkRateLimitingConfig) -> Self {
        let inner = inner::LocalRateLimiter::from_config(config);
        Self(Arc::new(Mutex::new(inner)))
    }

    /// Duplicate a handle to the rate limiter.
    pub fn shallow_clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }

    /// Reserve a rate limiting slot.
    pub fn reserve<C: Component>(&self, time: C::Instant) -> Result<SlotGuard<C>, RateLimited> {
        let mut this = self.0.lock();
        let slot = this.reserve::<C>(time)?;
        Ok(SlotGuard::new(self, slot))
    }
}
