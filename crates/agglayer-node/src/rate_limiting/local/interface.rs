/// Rate limiter interface.
pub trait RateLimiter {
    /// Type in which the flow of time is measured for this limiter.
    type Instant;

    /// Information exposed to the caller in case it's been rate limited.
    type RateLimited;

    /// Check if rate limit has been reached.
    fn check(&mut self, time: Self::Instant) -> Result<(), Self::RateLimited>;

    /// Apply rate limit.
    fn limit(&mut self, time: Self::Instant) -> Result<(), Self::RateLimited>;

    /// Check if the rate limiter contains any events in this period.
    fn is_empty(&mut self, time: Self::Instant) -> bool;
}
