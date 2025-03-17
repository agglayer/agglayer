//! Top-level rate limiter tracking all the networks.

use std::sync::Arc;

use parking_lot::{Mutex, MutexGuard};

use crate::{ConfigurableResource, NetworkId, RateLimitingConfig, Resource};

/// A global rate-limiter.
///
/// This is a shared handle to the rate limiter that can be used to access it
/// concurrently.
pub struct RateLimiter<R: Resource>(Arc<Mutex<inner::RateLimiter<R>>>);

impl<R: Resource> Clone for RateLimiter<R> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

mod inner;

impl<R: ConfigurableResource> RateLimiter<R> {
    /// Create a new rate limiter
    pub fn new(config: RateLimitingConfig) -> Self {
        Self(Arc::new(Mutex::new(inner::RateLimiter::new(config))))
    }

    /// Get rate limiter for given network
    pub fn limiter_for(&self, network_id: NetworkId) -> crate::local::LocalRateLimiter<R> {
        self.lock().limiter_for(network_id)
    }

    fn lock(&self) -> MutexGuard<inner::RateLimiter<R>> {
        self.0.lock()
    }
}

impl<R: Resource> std::fmt::Debug for RateLimiter<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad("RateLimiter(_)")
    }
}
