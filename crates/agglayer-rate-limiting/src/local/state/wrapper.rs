//! A safer interface to rate limiter state.

use agglayer_utils::log_assert;

use super::RawState;

/// A wrapper over the raw state that ensures it's up to date when queried.
///
/// It updates the internal "raw" rate limiter state before each query and
/// checks various invariants.
#[derive(Eq, PartialEq, Debug)]
pub struct State<S> {
    /// The raw underlying state.
    raw: S,
}

impl<S> State<S> {
    /// A new state wrapper
    pub fn new(raw: S) -> Self {
        Self { raw }
    }
}

impl<S: RawState> State<S> {
    /// Query the current limiter occupancy count.
    pub fn query(&mut self, time: S::Instant) -> usize {
        self.prune(time);
        self.raw.query()
    }

    /// Record a rate-limited event into the rate limiter state.
    pub fn record(&mut self, time: S::Instant) {
        log_assert!(
            self.raw.query() < self.raw.max_events(),
            "Recording into a full limiter"
        );
        self.raw.record(time)
    }

    /// Get the state information in case of rate being limited.
    pub fn limited_info(&self, time: S::Instant) -> S::LimitedInfo {
        self.raw.limited_info(time)
    }

    /// Limit on the number of event occurrences.
    pub fn max_events(&self) -> usize {
        self.raw.max_events()
    }

    /// Get non-modifying access to the underlying raw state.
    pub fn raw(&self) -> &S {
        &self.raw
    }

    /// Remove no longer relevant events.
    fn prune(&mut self, time: S::Instant) {
        let n_before = self.raw.query();
        log_assert!(n_before <= self.raw.max_events(), "Limiter overflow");

        self.raw.prune(time);

        let n_after = self.raw.query();
        log_assert!(
            n_after <= n_before,
            "Pruning increased occupancy from {n_before} to {n_after}"
        );
    }
}
