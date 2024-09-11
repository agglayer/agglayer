//! Transaction settlement rate limiter implementation.

use std::{
    collections::BTreeMap,
    num::NonZeroU32,
    sync::{Arc, Mutex, MutexGuard},
};

use agglayer_config::rate_limiting::{RateLimitingConfig, RollupId, TimeRateLimit};
use tokio::time::Instant;

mod wall_clock;

#[derive(PartialEq, Eq, Clone, Debug, serde::Serialize, thiserror::Error)]
pub enum Error {
    #[error("The `sendTx` settlement has been limited: {0}")]
    SendTxRateLimited(wall_clock::RateLimited),

    #[error("The `sendTx` settlement disabled by rate limiter")]
    SendTxDiabled,
}

/// A global rate-limiter.
///
/// This is a shared handle to the rate limiter that can be used to access it
/// concurrently.
///
/// Implementation note: The whole rate limiter is currently guarded by one
/// mutex which is shared among the networks. It may be sensible to also have
/// a mutex per network to increase the rate limiter throughput.
#[derive(Clone)]
pub struct RateLimiter(Arc<Mutex<RateLimiterImpl>>);

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(config: RateLimitingConfig) -> Self {
        Self(Arc::new(Mutex::new(RateLimiterImpl::new(config))))
    }

    /// Check rate limiting for `sendTx`.
    pub fn check_send_tx_now(&self, nid: RollupId) -> Result<(), Error> {
        self.check_send_tx(nid, Instant::now())
    }

    /// Check rate limiting for `sendTx` and record the request.
    pub fn limit_send_tx_now(&self, nid: RollupId) -> Result<(), Error> {
        self.limit_send_tx(nid, Instant::now())
    }

    /// Check rate limiting for `sendTx` with given request timestamp.
    pub fn check_send_tx(&self, nid: RollupId, now: Instant) -> Result<(), Error> {
        self.lock().check_send_tx(nid, now)
    }

    /// Check rate limiting for `sendTx` and record the request with given
    /// request timestamp.
    pub fn limit_send_tx(&self, nid: RollupId, now: Instant) -> Result<(), Error> {
        self.lock().limit_send_tx(nid, now)
    }

    fn lock(&self) -> MutexGuard<RateLimiterImpl> {
        self.0.lock().expect("mutex poisoned")
    }
}

impl std::fmt::Debug for RateLimiter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad("RateLimiter(_)")
    }
}

/// A global rate-limiter implementation.
///
/// It contains individual rate limiters for individual networks and endpoints.
struct RateLimiterImpl {
    /// A `sendTx` settlement limiter, one per network
    send_tx: BTreeMap<RollupId, wall_clock::RateLimiter>,

    /// Rate limiting configuration
    config: RateLimitingConfig,
}

impl RateLimiterImpl {
    fn new(config: RateLimitingConfig) -> Self {
        Self {
            send_tx: BTreeMap::new(),
            config,
        }
    }

    fn check_send_tx(&mut self, nid: RollupId, now: Instant) -> Result<(), Error> {
        self.with_send_tx_limiter(nid, |limiter| {
            limiter.check(now).map_err(Error::SendTxRateLimited)
        })
    }

    fn limit_send_tx(&mut self, nid: RollupId, now: Instant) -> Result<(), Error> {
        self.with_send_tx_limiter(nid, |limiter| {
            limiter.rate_limit(now).map_err(Error::SendTxRateLimited)
        })
    }

    /// Shorthand for working with tx rate limiter.
    ///
    /// * If settlement is not limited for given rollup, return `Ok(())`.
    /// * If settlement is disabled for the rollup, error out.
    /// * Otherwise, apply the function provided to rollup's rate limiter.
    fn with_send_tx_limiter(
        &mut self,
        nid: RollupId,
        func: impl FnOnce(&mut wall_clock::RateLimiter) -> Result<(), Error>,
    ) -> Result<(), Error> {
        match self.config.send_tx_limit(nid) {
            TimeRateLimit::Unlimited => Ok(()),

            TimeRateLimit::Limited {
                max_per_interval,
                time_interval,
            } => {
                let max_per_interval =
                    NonZeroU32::new(*max_per_interval).ok_or(Error::SendTxDiabled)?;
                let mk_limiter = || wall_clock::RateLimiter::new(max_per_interval, *time_interval);
                let limiter = self.send_tx.entry(nid).or_insert_with(mk_limiter);
                func(limiter)
            }
        }
    }
}
