use agglayer_utils::log_assert;

use super::{state::State, RawState, SlotTracker};

/// Rate limiter with non-trivial state.
///
/// An object of [RateLimiterCore] type adds the ability to reserve rate
/// limiting slots to the limiter state. A reserved slot takes up one space
/// in the limiter capacity. If the maximum capacity is reached, subsequent
/// reservation requests will be declined. A reserved slot can then be either
/// released, freeing up one space for subsequent requests, or recorded to
/// the rate limiter state, taking up capacity until expiration.
#[derive(Debug)]
pub struct RateLimiterCore<S> {
    /// The internal state of the rate limiter.
    state: State<S>,

    /// Number of reserved slots for events being actively processed.
    reserved: usize,
}

impl<S: RawState> RateLimiterCore<S> {
    /// Create a new limiter state.
    pub fn new(raw_state: S) -> Self {
        let state = State::new(raw_state);
        let reserved = 0;
        Self { state, reserved }
    }

    /// Query the total occupancy, including the reserved slots.
    pub fn query(&mut self, time: S::Instant) -> usize {
        let num = self.state.query(time) + self.reserved;
        log_assert!(num <= self.state.max_events());
        num
    }

    /// Reserve a rate limiting slot.
    pub fn reserve(&mut self, time: S::Instant) -> Result<SlotTracker, S::LimitedInfo> {
        let occupancy = self.query(time);
        log_assert!(occupancy <= self.state.max_events());

        if occupancy < self.state.max_events() {
            self.reserved += 1;
            Ok(SlotTracker::new())
        } else {
            Err(self.state.limited_info(time))
        }
    }

    /// Release a rate limiting slot.
    pub fn release(&mut self, slot: SlotTracker) {
        let r = &mut self.reserved;
        let n = slot.release();
        log_assert!(n <= *r, "Releasing {n} slots but only {r} reserved");
        *r -= n;
    }

    /// Record a rate limiting event.
    pub fn record(&mut self, time: S::Instant, slot: SlotTracker) {
        log_assert!(self.state.raw().query() < self.state.max_events());
        self.state.record(time);
        self.release(slot);
    }

    /// Access the raw state
    #[cfg(test)]
    pub fn raw(&self) -> &S {
        self.state.raw()
    }
}

#[cfg(test)]
mod tests {
    use super::{RateLimiterCore, RawState};

    /// Simple testing state. Recorded events never expire.
    struct TestState {
        num: usize,
        max: usize,
    }

    impl TestState {
        fn new(max: usize) -> Self {
            let num = 0;
            Self { num, max }
        }
    }

    impl RawState for TestState {
        type Instant = ();
        type LimitedInfo = ();

        fn prune(&mut self, _time: ()) {}

        fn query(&self) -> usize {
            self.num
        }

        fn record(&mut self, _time: ()) {
            self.num += 1;
        }

        fn limited_info(&self, _time: ()) -> Self::LimitedInfo {}

        fn max_events(&self) -> usize {
            self.max
        }
    }

    impl RateLimiterCore<TestState> {
        fn check_counts(&mut self, reserved: usize, recorded: usize) {
            assert_eq!(self.reserved, reserved);
            assert_eq!(self.raw().num, recorded);
            assert_eq!(self.query(()), reserved + recorded);
        }
    }

    #[test]
    fn slot_reservation() {
        let mut limiter = RateLimiterCore::new(TestState::new(5));
        limiter.check_counts(0, 0);

        // Reserve and release a slot.
        let slot0 = limiter.reserve(()).unwrap();
        limiter.check_counts(1, 0);
        limiter.release(slot0);
        limiter.check_counts(0, 0);

        // Record two slots.
        let slot0 = limiter.reserve(()).unwrap();
        limiter.check_counts(1, 0);
        limiter.record((), slot0);
        limiter.check_counts(0, 1);
        let slot1 = limiter.reserve(()).unwrap();
        limiter.record((), slot1);
        limiter.check_counts(0, 2);

        // Reserve three slots.
        let slot2 = limiter.reserve(()).unwrap();
        limiter.check_counts(1, 2);
        let slot3 = limiter.reserve(()).unwrap();
        limiter.check_counts(2, 2);
        let slot4 = limiter.reserve(()).unwrap();
        limiter.check_counts(3, 2);

        // Try to reserve another one while we are at capacity.
        limiter.reserve(()).unwrap_err();
        limiter.check_counts(3, 2);

        // Record or release slots out of order.
        limiter.record((), slot3);
        limiter.check_counts(2, 3);
        limiter.record((), slot2);
        limiter.check_counts(1, 4);
        limiter.release(slot4);
        limiter.check_counts(0, 4);

        // Record into the last remaining rate limiting space.
        let slot4 = limiter.reserve(()).unwrap();
        limiter.record((), slot4);
        limiter.check_counts(0, 5);

        // Limiter completely full at this point.
        limiter.reserve(()).unwrap_err();
    }
}
