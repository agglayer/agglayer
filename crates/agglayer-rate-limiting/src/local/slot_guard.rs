use super::{limiter::SlotTracker, LocalRateLimiter, Resource};

/// Guard taking up one rate limiting slot.
///
/// The reserved slot may be either:
/// * recorded into the rate limiter by using [Self::record], or
/// * released by dropping the object (letting it go out of scope)
pub struct SlotGuard<R: Resource> {
    limiter: LocalRateLimiter<R>,
    slot: SlotTracker,
}

impl<R: Resource> SlotGuard<R> {
    /// New slot guard.
    pub(super) fn new(limiter: &LocalRateLimiter<R>, slot: SlotTracker) -> Self {
        let limiter = limiter.shallow_clone();
        Self { limiter, slot }
    }

    /// Record a rate limiting event.
    pub fn record(mut self, time: R::Instant) {
        self.limiter.0.lock().record(time, self.slot.take());
    }
}

impl<R: Resource> Drop for SlotGuard<R> {
    fn drop(&mut self) {
        self.limiter.0.lock().release(self.slot.take());
    }
}
