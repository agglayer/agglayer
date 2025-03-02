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
    #[serde_as(as = "Option<DurationSeconds>")]
    pub until_next: Option<Duration>,
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

/// Information wall clock rate limiter has to track.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct WallClockState {
    /// Past events recorded by the limiter and their times.
    ///
    /// Performance note: This is currently a `Vec` and wiping past events
    /// requires a linear scan. If the rate limit is small, say up to 16,
    /// this is the fastest way to do it. If there is a reason to expect the
    /// limits can be much higher, it should be changed to `BinaryHeap`.
    past: Vec<Instant>,

    /// Limiter parameters.
    params: Params,
}

impl WallClockState {
    /// Create a new rate limiter with given parameters.
    pub fn new(max_per_interval: NonZeroU32, time_interval: Duration) -> Self {
        let params = Params::new(max_per_interval, time_interval);
        let past = Vec::with_capacity(params.max_per_interval());
        Self { past, params }
    }
}

impl super::RawState for WallClockState {
    type Instant = Instant;

    type LimitedInfo = RateLimited;

    fn prune(&mut self, time: Instant) {
        let up_to = self.params.expiry_point(time);
        self.past.retain(|t| *t > up_to);
    }

    fn query(&self) -> usize {
        self.past.len()
    }

    fn record(&mut self, time: Instant) {
        assert!(self.past.len() < self.params.max_per_interval());
        self.past.push(time)
    }

    fn limited_info(&self, time: Instant) -> Self::LimitedInfo {
        let earliest = self.past.first();
        let until_next = earliest.map(|t| t.duration_since(self.params.expiry_point(time)));

        RateLimited {
            time_interval: self.params.time_interval,
            max_per_interval: self.params.max_per_interval.get(),
            until_next,
        }
    }

    fn max_events(&self) -> usize {
        self.params.max_per_interval()
    }
}

#[cfg(test)]
mod tests {
    use std::{num::NonZeroU32, time::Duration};

    use tokio::time::Instant;

    use crate::local::{
        limiter::RateLimiterCore,
        state::{
            wall_clock::{Params, RateLimited, WallClockState},
            RawState,
        },
    };

    type TestLimiter = RateLimiterCore<WallClockState>;

    impl TestLimiter {
        /// Create from params, just for testing.
        fn from_params(max_per_interval: NonZeroU32, time_interval: Duration) -> Self {
            TestLimiter::new(WallClockState::new(max_per_interval, time_interval))
        }

        /// Apply rate limiting with a bunch of additional checks.
        fn check_and_limit(&mut self, time: Instant) -> Result<(), RateLimited> {
            let orig_past_len = self.raw().past.len();

            // Call check for the first time
            let occupancy = self.query(time);

            // Record the history after the first check.
            let pruned_past = self.raw().past.clone();
            assert!(occupancy >= pruned_past.len());
            assert!(pruned_past.len() <= orig_past_len);

            // Perform check, assert history is already up to date.
            let check_result = self.reserve(time);
            let occupancy_after = occupancy + check_result.is_ok() as usize;
            assert_eq!(self.query(time), occupancy_after);
            assert_eq!(pruned_past, self.raw().past);

            // Save the result but release the slot.
            let check_result = check_result.map(|slot| self.release(slot));
            assert_eq!(self.query(time), occupancy);
            assert_eq!(pruned_past, self.raw().past);

            // Perform additional check, assert history does not change.
            let check_result_2 = self.reserve(time);
            assert_eq!(self.query(time), occupancy_after);
            assert_eq!(pruned_past, self.raw().past);

            // This time record the event.
            let check_result_2 = check_result_2.map(|slot| self.record(time, slot));
            assert_eq!(check_result, check_result_2);
            assert_eq!(self.query(time), occupancy_after);

            let expected_past_len = pruned_past.len() + check_result.is_ok() as usize;
            assert_eq!(self.raw().past.len(), expected_past_len);

            check_result
        }
    }

    #[test]
    fn prune_idempotent() {
        let time_interval = Duration::from_secs(100);
        let max_per_interval = 100;
        let now = Instant::now();

        let offsets = [0, 12, 21, 28, 45, 52, 75, 77, 85, 89];
        let past = offsets
            .into_iter()
            .map(|s| now + Duration::from_secs(s))
            .collect();

        let params = Params::new(NonZeroU32::new(max_per_interval).unwrap(), time_interval);

        let mut state = WallClockState { past, params };

        for t in (0..120).map(|dt| now + Duration::from_secs(dt)) {
            state.prune(t);
            let past = state.past.clone();
            state.prune(t);
            assert_eq!(past, state.past);
        }
    }

    #[test]
    fn event_burst() {
        let time_interval = Duration::from_secs(100);
        let max_per_interval = 3;
        let now = Instant::now();
        let mut limiter =
            TestLimiter::from_params(NonZeroU32::new(max_per_interval).unwrap(), time_interval);

        assert_eq!(limiter.check_and_limit(now), Ok(()));
        assert_eq!(limiter.check_and_limit(now), Ok(()));
        assert_eq!(limiter.check_and_limit(now), Ok(()));

        assert_eq!(
            limiter.check_and_limit(now),
            Err(RateLimited {
                max_per_interval,
                time_interval,
                until_next: Some(time_interval),
            })
        );

        assert_eq!(
            limiter.check_and_limit(now + Duration::from_secs(100)),
            Ok(())
        );
    }

    #[test]
    fn limiting_error() {
        let time_interval = Duration::from_secs(100);
        let max_per_interval = 1;
        let now = Instant::now();
        let mut limiter =
            TestLimiter::from_params(NonZeroU32::new(max_per_interval).unwrap(), time_interval);

        assert_eq!(limiter.check_and_limit(now), Ok(()));

        assert_eq!(
            limiter.check_and_limit(now + Duration::from_secs(15)),
            Err(RateLimited {
                max_per_interval,
                time_interval,
                until_next: Some(Duration::from_secs(85)),
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

        let mut limiter = TestLimiter::from_params(NonZeroU32::new(limit).unwrap(), time_interval);

        for (offset, ok) in offsets_and_expected_outcomes {
            let res = limiter.check_and_limit(now + Duration::from_secs(offset));
            assert_eq!(res.is_ok(), ok, "offset {offset}");
        }
    }
}
