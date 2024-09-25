use std::collections::BTreeMap;

use super::{super::LocalRateLimiter, NetworkId, RateLimitingConfig};

/// A global rate-limiter implementation.
///
/// It contains individual rate limiters for individual networks and endpoints.
pub struct RateLimiter {
    /// A `sendTx` settlement limiter, one per network
    per_network: BTreeMap<NetworkId, LocalRateLimiter>,

    /// Rate limiting configuration
    config: RateLimitingConfig,
}

impl RateLimiter {
    pub fn new(config: RateLimitingConfig) -> Self {
        Self {
            per_network: BTreeMap::new(),
            config,
        }
    }

    pub fn limiter_for(&mut self, network_id: NetworkId) -> LocalRateLimiter {
        let mk_limiter = || LocalRateLimiter::from_config(&self.config.config_for(network_id));
        self.per_network
            .entry(network_id)
            .or_insert_with(mk_limiter)
            .shallow_clone()
    }
}
