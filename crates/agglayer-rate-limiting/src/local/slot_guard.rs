use std::marker::PhantomData;

use super::{limiter::SlotTracker, Component, LocalRateLimiter};

/// Guard taking up one rate limiting slot.
///
/// The reserved slot may be either:
/// * recorded into the rate limiter by using [Self::record], or
/// * released by dropping the object (letting it go out of scope)
pub struct SlotGuard<C: Component> {
    _component: PhantomData<fn() -> C>,
    limiter: LocalRateLimiter,
    slot: SlotTracker,
}

impl<C: Component> SlotGuard<C> {
    /// New slot guard.
    pub(super) fn new(limiter: &LocalRateLimiter, slot: SlotTracker) -> Self {
        Self {
            _component: PhantomData,
            limiter: limiter.shallow_clone(),
            slot,
        }
    }

    /// Record a rate limiting event.
    pub fn record(mut self, time: C::Instant) {
        self.limiter.0.lock().record::<C>(time, self.slot.take());
    }
}

impl<C: Component> Drop for SlotGuard<C> {
    fn drop(&mut self) {
        self.limiter.0.lock().release::<C>(self.slot.take());
    }
}
