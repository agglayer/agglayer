//! Wall clock time rate limiter components.

use std::{num::NonZeroU32, time::Duration};

use serde_with::{serde_as, DurationSeconds};
use tokio::time::Instant;

#[cfg(test)]
mod tests;

/// An error indicating the request has been rate limited.
#[serde_with::serde_as]
#[derive(Clone, Eq, PartialEq, Debug, serde::Serialize, thiserror::Error)]
#[serde(rename_all = "kebab-case")]
#[error("Limit reached")]
pub struct RateLimited {
    /// Limit for number of requests in given time window.
    pub max_per_interval: u32,

    /// Rate limiting time window.
    #[serde_as(as = "DurationSeconds")]
    pub time_interval: Duration,

    /// Number of seconds left until rate limit is expected to pass.
    #[serde_as(as = "DurationSeconds")]
    pub until_next: Duration,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct Params {
    max_per_interval: NonZeroU32,
    time_interval: Duration,
}

impl Params {
    fn new(max_per_interval: NonZeroU32, time_interval: Duration) -> Self {
        Self {
            max_per_interval,
            time_interval,
        }
    }

    /// The time point before which the events are considered expired and do not
    /// contribute towards rate limiting.
    fn expiry_point(&self, now: Instant) -> Instant {
        now - self.time_interval
    }

    /// Maximum number of events per interval
    fn max_per_interval(&self) -> usize {
        self.max_per_interval.get() as usize
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RateLimiter {
    /// Past events recorded by the limiter and their times.
    ///
    /// Performance note: This is currently a `Vec` and wiping past events
    /// requires a linear scan. If the rate limit is small, say up to 16,
    /// this is the fastest way to do it. If there is a reason to expect the
    /// limits can be much higher, it should be changed to `BinaryHeap`.
    past: Vec<Instant>,

    /// Rate limiting parameters.
    params: Params,
}

impl RateLimiter {
    /// Create a new rate limiter with given parameters.
    pub fn new(max_per_interval: NonZeroU32, time_interval: Duration) -> Self {
        let params = Params::new(max_per_interval, time_interval);
        let past = Vec::with_capacity(params.max_per_interval());
        Self { past, params }
    }

    /// Check if given event would pass the rate limiter without recording it.
    ///
    /// Both [Self::check] and [Self::rate_limit] prune past expired events.
    /// If the time instants passed are not monotonically increasing, this
    /// may result in unexpected behavior.
    pub fn check(&mut self, time: Instant) -> Result<(), RateLimited> {
        assert!(self.past.len() <= self.params.max_per_interval.get() as usize);

        let num_events = self.updated_event_count(self.params.expiry_point(time));

        if num_events >= self.params.max_per_interval() {
            let earliest = *self.past.first().expect("rate limited with empty past");
            let until_next = earliest.duration_since(self.params.expiry_point(time));

            return Err(RateLimited {
                time_interval: self.params.time_interval,
                max_per_interval: self.params.max_per_interval.get(),
                until_next,
            });
        }
        Ok(())
    }

    /// Check adding an event at given time passes the rate limit test and
    /// record it in the rate limiter if it does.
    pub fn rate_limit(&mut self, time: Instant) -> Result<(), RateLimited> {
        self.check(time)?;
        self.add_event(time);
        Ok(())
    }

    /// The number of past events after wiping the expired ones.
    fn updated_event_count(&mut self, start: Instant) -> usize {
        self.updated_past(start).len()
    }

    /// The list of past events after wiping the expired ones.
    fn updated_past(&mut self, start: Instant) -> &[Instant] {
        self.wipe(start);
        &self.past
    }

    /// Wipe the expired past events before the specified time instant.
    fn wipe(&mut self, up_to: Instant) {
        self.past.retain(|t| *t > up_to);
    }

    /// Record a new event in the rate limiter.
    ///
    /// The max request count must have already been checked to avoid panic.
    fn add_event(&mut self, time: Instant) {
        assert!(self.past.len() < self.params.max_per_interval());
        self.past.push(time)
    }

    /// Check if the limiter is empty.
    fn is_clear(&mut self, time: Instant) -> bool {
        self.updated_past(time).is_empty()
    }
}

impl super::RateLimiter for RateLimiter {
    type Instant = Instant;

    type RateLimited = RateLimited;

    fn check(&mut self, time: Self::Instant) -> Result<(), Self::RateLimited> {
        self.check(time)
    }

    fn limit(&mut self, time: Self::Instant) -> Result<(), Self::RateLimited> {
        self.rate_limit(time)
    }

    fn is_empty(&mut self, time: Self::Instant) -> bool {
        self.is_clear(time)
    }
}
