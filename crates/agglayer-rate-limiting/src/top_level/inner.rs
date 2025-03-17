use std::collections::BTreeMap;

use crate::{LocalRateLimiter, NetworkId, RateLimitingConfig, Resource, ConfigurableResource};

/// A global rate-limiter implementation.
///
/// It contains individual rate limiters for individual networks.
pub struct RateLimiter<R: Resource> {
    /// A `sendTx` settlement limiter, one per network
    per_network: BTreeMap<NetworkId, LocalRateLimiter<R>>,

    /// Rate limiting configuration
    config: RateLimitingConfig,
}

impl<R: Resource> RateLimiter<R> {
    pub fn new(config: RateLimitingConfig) -> Self {
        Self {
            per_network: BTreeMap::new(),
            config,
        }
    }
}

impl<R: ConfigurableResource> RateLimiter<R> {
    pub fn limiter_for(&mut self, network_id: NetworkId) -> LocalRateLimiter<R> {
        let mk_limiter = || LocalRateLimiter::from_config(&self.config.config_for(network_id));
        self.per_network
            .entry(network_id)
            .or_insert_with(mk_limiter)
            .shallow_clone()
    }
}
