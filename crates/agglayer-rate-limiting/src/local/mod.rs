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
//! * [limiter] implements rate limiting for single limiter resource for single
//!   network. It adds the ability to reserve a slot which can be committed later,
//!   and takes care of  the handling of trivial cases (e.g. no limit).
//! * [self] (this module) defines the top-level [LocalRateLimiter]. It takes
//!   care of cross-thread synchronization and provides a safe interface to rate
//!   limiter using a [SlotGuard].

use std::sync::Arc;

use agglayer_config::rate_limiting::NetworkRateLimitingConfig;
use parking_lot::{Mutex, MutexGuard};

mod limiter;
pub mod resource;
mod slot_guard;
mod state;

pub use resource::{Resource, ConfigurableResource};
pub use slot_guard::SlotGuard;

/// Rate limiter state for single network / rollup.
pub struct LocalRateLimiter<R: Resource>(Arc<Mutex<limiter::RateLimiter<R::State>>>);

impl<R: ConfigurableResource> LocalRateLimiter<R> {
    /// Create a rate limiter form configuration.
    pub fn from_config(config: &NetworkRateLimitingConfig) -> Self {
        todo!()
        /*
        let inner = inner::LocalRateLimiter::from_config(config);
        Self(Arc::new(Mutex::new(inner)))
        */
    }
}

impl<R: Resource> LocalRateLimiter<R> {

    /// Duplicate a handle to the rate limiter.
    pub fn shallow_clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }

    /// Reserve a slot in this rate limiter.
    pub fn reserve(&self, time: R::Instant) -> Result<SlotGuard<R>, R::LimitedInfo> {
        let slot = self.lock().reserve(time)?;
        Ok(SlotGuard::new(self, slot))
    }

    fn lock(&self) -> MutexGuard<limiter::RateLimiter<R::State>> {
        self.0.lock()
    }
}
