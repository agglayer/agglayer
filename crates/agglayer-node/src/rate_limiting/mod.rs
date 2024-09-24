//! Transaction settlement rate limiter implementation.

mod local;
mod top_level;

#[cfg(test)]
pub mod tests;

// Internal exports.
use local::LocalRateLimiter;

// Public exports.
pub use agglayer_config::rate_limiting::{NetworkId, RateLimitingConfig};
pub use local::{component, Component, RateLimited, SlotGuard};
pub use top_level::RateLimiter;

pub type SendTxSlotGuard = SlotGuard<component::SendTx>;
