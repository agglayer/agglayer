/// A low-level interface to the internal state of a rate limiter.
///
/// This is a low-level interface that a limiter implements. Not to be used
/// directly, the generic rate limiter provides higher-level functionality.
pub trait RawState {
    /// Type by which the flow of time is measured for this limiter.
    type Instant: Copy + Ord;

    /// Information exposed to the caller in case it's been rate limited.
    type LimitedInfo;

    /// Update the state by pruning expired events given current time.
    fn prune(&mut self, time: Self::Instant);

    /// Query the current limiter occupancy count.
    fn query(&self) -> usize;

    /// Record a rate-limited event into the rate limiter state.
    fn record(&mut self, time: Self::Instant);

    /// Get the state information in case of rate being limited.
    fn limited_info(&self, time: Self::Instant) -> Self::LimitedInfo;

    /// Get the maximum number of events in the limiter.
    fn max_events(&self) -> usize;
}
