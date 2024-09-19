//! Wall clock time rate limiter components.

use std::{num::NonZeroU32, time::Duration};

use serde_with::{serde_as, DurationSeconds};
use tokio::time::Instant;

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

        let num_events = self.updated_event_count(time);

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

    /// Wipe the expired past events given the current time.
    fn wipe(&mut self, time: Instant) {
        let up_to = self.params.expiry_point(time);
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
    fn is_empty(&mut self, time: Instant) -> bool {
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
        self.is_empty(time)
    }
}

#[cfg(test)]
mod tests {
    use std::{num::NonZeroU32, time::Duration};

    use tokio::time::Instant;

    use super::{Params, RateLimited, RateLimiter};

    impl RateLimiter {
        /// Just like [Self::rate_limit] but with additional sanity checks.
        fn check_and_limit(&mut self, time: Instant) -> Result<(), RateLimited> {
            let orig_past_len = self.past.len();

            // Call check for the first time
            let check_result = self.check(time);

            // Record the history after the first check.
            let pruned_past = self.past.clone();
            assert!(pruned_past.len() <= orig_past_len);
            let is_empty = self.is_empty(time);
            assert_eq!(is_empty, pruned_past.is_empty());
            assert_eq!(self.past, pruned_past);

            // Make second check, assert history is idempotent.
            let check_result_2 = self.check(time);
            assert_eq!(check_result, check_result_2);
            assert_eq!(pruned_past, self.past);

            // Check and record rate limiting event.
            let limit_result = self.rate_limit(time);
            assert_eq!(check_result, limit_result);

            if check_result.is_ok() {
                assert_eq!(pruned_past.len() + 1, self.past.len());
            } else {
                assert_eq!(pruned_past, self.past);
            }

            check_result
        }
    }

    #[test]
    fn wipe_idempotent() {
        let time_interval = Duration::from_secs(100);
        let max_per_interval = 100;
        let now = Instant::now();

        let offsets = [0, 12, 21, 28, 45, 52, 75, 77, 85, 89];
        let past = offsets
            .into_iter()
            .map(|s| now + Duration::from_secs(s))
            .collect();

        let params = Params::new(NonZeroU32::new(max_per_interval).unwrap(), time_interval);

        let mut limiter = RateLimiter { past, params };

        for t in (0..120).map(|dt| now + Duration::from_secs(dt)) {
            limiter.wipe(t);
            let past = limiter.past.clone();
            limiter.wipe(t);
            assert_eq!(past, limiter.past);
        }
    }

    #[test]
    fn event_burst() {
        let time_interval = Duration::from_secs(100);
        let max_per_interval = 3;
        let now = Instant::now();
        let mut limiter =
            RateLimiter::new(NonZeroU32::new(max_per_interval).unwrap(), time_interval);

        assert_eq!(limiter.check_and_limit(now), Ok(()));
        assert_eq!(limiter.check_and_limit(now), Ok(()));
        assert_eq!(limiter.check_and_limit(now), Ok(()));

        assert_eq!(
            limiter.check_and_limit(now),
            Err(RateLimited {
                max_per_interval,
                time_interval,
                until_next: time_interval,
            })
        );

        assert_eq!(limiter.rate_limit(now + Duration::from_secs(100)), Ok(()));
    }

    #[test]
    fn limiting_error() {
        let time_interval = Duration::from_secs(100);
        let max_per_interval = 1;
        let now = Instant::now();
        let mut limiter =
            RateLimiter::new(NonZeroU32::new(max_per_interval).unwrap(), time_interval);

        assert_eq!(limiter.rate_limit(now), Ok(()));

        assert_eq!(
            limiter.rate_limit(now + Duration::from_secs(15)),
            Err(RateLimited {
                max_per_interval,
                time_interval,
                until_next: Duration::from_secs(85),
            })
        );
    }

    #[test]
    fn wall_clock_limiter() {
        let time_interval = Duration::from_secs(100);
        let limit = 3;
        let now = Instant::now();

        let offsets_and_expected_outcomes = [
            (0, true),
            (15, true),
            (55, true),
            (102, true),
            (149, true),
            (154, false),
            (155, true),
            (156, false),
            (280, true),
            (380, true),
            (381, true),
            (382, true),
            (383, false),
        ];

        let mut limiter = RateLimiter::new(NonZeroU32::new(limit).unwrap(), time_interval);

        for (offset, ok) in offsets_and_expected_outcomes {
            let res = limiter.check_and_limit(now + Duration::from_secs(offset));
            assert_eq!(res.is_ok(), ok, "offset {offset}");
        }
    }
}
