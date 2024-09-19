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
    let mut limiter = RateLimiter::new(NonZeroU32::new(max_per_interval).unwrap(), time_interval);

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
    let mut limiter = RateLimiter::new(NonZeroU32::new(max_per_interval).unwrap(), time_interval);

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
