//! Transaction settlement rate limiter implementation.

mod local;
mod top_level;

#[cfg(test)]
pub mod tests;

pub use agglayer_config::rate_limiting::{
    NetworkId, NetworkRateLimitingConfig, RateLimitingConfig,
};
pub use local::{resource, ConfigurableResource, LocalRateLimiter, Resource, SlotGuard};
pub use top_level::RateLimiter;

pub type SendTxSlotGuard = SlotGuard<resource::SendTxSettlement>;
pub type SendTxRateLimiter = RateLimiter<resource::SendTxSettlement>;
