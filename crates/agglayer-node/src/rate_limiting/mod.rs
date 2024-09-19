//! Transaction settlement rate limiter implementation.

use std::{collections::BTreeMap, sync::Arc};

pub use agglayer_config::rate_limiting::{RateLimitingConfig, RollupId};
use parking_lot::{Mutex, MutexGuard};
use tokio::time::Instant;

pub mod local;

#[cfg(test)]
pub mod tests;

use local::LocalRateLimiter;
pub use local::RateLimited;

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
    pub fn check_send_tx_now(&self, nid: RollupId) -> Result<(), RateLimited> {
        self.check_send_tx(nid, Instant::now())
    }

    /// Check rate limiting for `sendTx` and record the request.
    pub fn limit_send_tx_now(&self, nid: RollupId) -> Result<(), RateLimited> {
        self.limit_send_tx(nid, Instant::now())
    }

    /// Check rate limiting for `sendTx` with given request timestamp.
    pub fn check_send_tx(&self, nid: RollupId, now: Instant) -> Result<(), RateLimited> {
        self.lock().check_send_tx(nid, now)
    }

    /// Check rate limiting for `sendTx` and record the request with given
    /// request timestamp.
    pub fn limit_send_tx(&self, nid: RollupId, now: Instant) -> Result<(), RateLimited> {
        self.lock().limit_send_tx(nid, now)
    }

    fn lock(&self) -> MutexGuard<RateLimiterImpl> {
        self.0.lock()
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
    per_network: BTreeMap<RollupId, LocalRateLimiter>,

    /// Rate limiting configuration
    config: RateLimitingConfig,
}

impl RateLimiterImpl {
    fn new(config: RateLimitingConfig) -> Self {
        Self {
            per_network: BTreeMap::new(),
            config,
        }
    }

    fn check_send_tx(&mut self, rollup_id: RollupId, time: Instant) -> Result<(), RateLimited> {
        self.limiter_for(rollup_id).check_send_tx(time)
    }

    fn limit_send_tx(&mut self, rollup_id: RollupId, time: Instant) -> Result<(), RateLimited> {
        self.limiter_for(rollup_id).limit_send_tx(time)
    }

    fn limiter_for(&mut self, rollup_id: RollupId) -> &mut LocalRateLimiter {
        let mk_limiter = || LocalRateLimiter::from_config(&self.config.config_for(rollup_id));
        self.per_network.entry(rollup_id).or_insert_with(mk_limiter)
    }
}
