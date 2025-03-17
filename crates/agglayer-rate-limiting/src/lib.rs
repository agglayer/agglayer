//! Transaction settlement rate limiter implementation.

mod local;
mod top_level;

#[cfg(test)]
pub mod tests;

pub use agglayer_config::rate_limiting::{NetworkId, RateLimitingConfig};
pub use local::{resource, Resource, SlotGuard, LocalRateLimiter, ConfigurableResource};
pub use top_level::RateLimiter;

pub type SendTxSlotGuard = SlotGuard<resource::SendTxSettlement>;
pub type SendTxRateLimiter = RateLimiter<resource::SendTxSettlement>;
