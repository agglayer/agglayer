//! Top-level rate limiter tracking all the networks.

use std::sync::Arc;

use agglayer_types::EpochNumber;
use parking_lot::{Mutex, MutexGuard};
use tokio::time::Instant;

use super::{component, Component, NetworkId, RateLimited, RateLimitingConfig, SlotGuard};

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

    /// Reserve rate limiting slot for `sendTx`.
    pub fn reserve_send_tx(
        &self,
        network_id: NetworkId,
        time: Instant,
    ) -> Result<SlotGuard<component::SendTx>, RateLimited> {
        self.reserve::<component::SendTx>(network_id, time)
    }

    /// Reserve rate limiting slot for `sendTx`.
    pub fn reserve_send_certificate(
        &self,
        network_id: NetworkId,
        epoch_no: EpochNumber,
    ) -> Result<SlotGuard<component::SendCertificate>, RateLimited> {
        self.reserve::<component::SendCertificate>(network_id, epoch_no)
    }

    /// Reserve rate limiting slot for given component.
    pub fn reserve<C: Component>(
        &self,
        network_id: NetworkId,
        time: C::Instant,
    ) -> Result<SlotGuard<C>, RateLimited> {
        let limiter = self.lock().limiter_for(network_id);
        limiter.reserve::<C>(time)
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
