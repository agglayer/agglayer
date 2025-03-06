//! Top-level rate limiter tracking all the networks.

use std::sync::Arc;

use parking_lot::{Mutex, MutexGuard};

use super::{NetworkId, RateLimitingConfig};

/// A global rate-limiter.
///
/// This is a shared handle to the rate limiter that can be used to access it
/// concurrently.
#[derive(Clone)]
pub struct RateLimiter(Arc<Mutex<inner::RateLimiter>>);

mod inner;

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(config: RateLimitingConfig) -> Self {
        Self(Arc::new(Mutex::new(inner::RateLimiter::new(config))))
    }

    /// Get rate limiter for given network
    pub fn limiter_for(&self, network_id: NetworkId) -> crate::local::LocalRateLimiter {
        self.lock().limiter_for(network_id)
    }

    fn lock(&self) -> MutexGuard<inner::RateLimiter> {
        self.0.lock()
    }
}

impl std::fmt::Debug for RateLimiter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad("RateLimiter(_)")
    }
}
