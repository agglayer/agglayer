use std::{
    sync::atomic::{AtomicU32, Ordering},
    thread,
    time::Duration,
};

use agglayer_config::rate_limiting::{NetworkId, TimeRateLimit};
use tokio::time::Instant;

use super::{RateLimited, RateLimiter, RateLimitingConfig};

impl RateLimiter {
    fn limit_send_tx(&self, network_id: NetworkId, time: Instant) -> Result<(), RateLimited> {
        self.reserve_send_tx(network_id, time)
            .map(|guard| guard.record(time))
    }
}

const ONE_PER_100S: TimeRateLimit = TimeRateLimit::Limited {
    max_per_interval: 1,
    time_interval: Duration::from_secs(100),
};

const THREE_PER_100S: TimeRateLimit = TimeRateLimit::Limited {
    max_per_interval: 3,
    time_interval: Duration::from_secs(100),
};

const DISABLED: TimeRateLimit = TimeRateLimit::Limited {
    max_per_interval: 0,
    time_interval: Duration::from_secs(100),
};

#[test]
fn concurrent_access() {
    let limiter = RateLimiter::new(RateLimitingConfig::new(THREE_PER_100S));
    let now = Instant::now();

    let success_count = AtomicU32::new(0);
    let failure_count = AtomicU32::new(0);

    thread::scope(|scope| {
        (0..5).for_each(|_| {
            let limiter = limiter.clone();
            let success_count = &success_count;
            let failure_count = &failure_count;

            scope.spawn(move || {
                let result = limiter.limit_send_tx(42, now);
                if result.is_ok() {
                    success_count.fetch_add(1, Ordering::AcqRel);
                } else {
                    failure_count.fetch_add(1, Ordering::AcqRel);
                }
            });
        });
    });

    // Rate limit 3 with 5 threads => there should be 3 successes and 2 fails.
    assert_eq!(success_count.into_inner(), 3);
    assert_eq!(failure_count.into_inner(), 2);
}

#[test]
fn per_network() {
    let limiter = RateLimiter::new(RateLimitingConfig::new(ONE_PER_100S));
    let now = Instant::now();

    let network_ids = [2, 42, 1337, 95];

    for network_id in network_ids {
        assert_eq!(limiter.limit_send_tx(network_id, now), Ok(()));
    }

    for (network_id, offset) in network_ids.into_iter().zip([5, 9, 64, 99]) {
        let now = now + Duration::from_secs(offset);
        assert!(limiter.limit_send_tx(network_id, now).is_err());
    }

    for (network_id, offset) in network_ids.into_iter().zip([102, 105, 157, 455]) {
        let now = now + Duration::from_secs(offset);
        assert!(limiter.limit_send_tx(network_id, now).is_ok());
    }
}

#[test]
fn network_exempt() {
    let config =
        RateLimitingConfig::new(ONE_PER_100S).with_send_tx_override(42, TimeRateLimit::Unlimited);
    let limiter = RateLimiter::new(config);
    let now = Instant::now();
    let at = |secs: u64| now + Duration::from_secs(secs);

    // Network 42 has no limits, other are limited to 1tx per 100s
    assert_eq!(limiter.limit_send_tx(42, at(0)), Ok(()));
    assert_eq!(limiter.limit_send_tx(42, at(1)), Ok(()));
    assert_eq!(limiter.limit_send_tx(7, at(1)), Ok(()));
    assert_eq!(limiter.limit_send_tx(42, at(5)), Ok(()));
    assert_eq!(limiter.limit_send_tx(5, at(9)), Ok(()));
    assert_eq!(limiter.limit_send_tx(42, at(18)), Ok(()));
    assert_eq!(limiter.limit_send_tx(42, at(20)), Ok(()));
    assert!(limiter.limit_send_tx(5, at(37)).is_err());
    assert_eq!(limiter.limit_send_tx(42, at(38)), Ok(()));
    assert_eq!(limiter.limit_send_tx(42, at(59)), Ok(()));
    assert!(limiter.limit_send_tx(5, at(71)).is_err());
    assert!(limiter.limit_send_tx(7, at(95)).is_err());
    assert!(limiter.limit_send_tx(7, at(100)).is_err());
    assert_eq!(limiter.limit_send_tx(42, at(105)), Ok(()));
    assert_eq!(limiter.limit_send_tx(7, at(109)), Ok(()));
}

#[test]
fn network_disabled() {
    let config = RateLimitingConfig::new(ONE_PER_100S).with_send_tx_override(55, DISABLED);
    let limiter = RateLimiter::new(config);
    let now = Instant::now();

    assert_eq!(limiter.limit_send_tx(12, now), Ok(()));
    assert_eq!(
        limiter.limit_send_tx(55, now),
        Err(RateLimited::SendTxDiabled {})
    );
    assert_eq!(limiter.limit_send_tx(19, now), Ok(()));
}
