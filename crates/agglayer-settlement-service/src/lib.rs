//! The settlement service handles transaction settlement operations for the
//! AggLayer.
//!
//! This service consists of:
//! - [`SettlementServiceTrait`]: the public API the certificate orchestrator
//!   depends on
//! - [`SettlementService`]: The main service orchestrating settlement
//!   operations
//! - `SettlementTask` (internal): worker task for processing individual
//!   settlements

#![allow(dead_code)] // TODO remove after settlement service is integrated in the rest of the app

pub mod settlement_service;
pub mod settlement_service_trait;
mod settlement_task;
mod utils;
mod wallet_nonce_locks;

pub use settlement_service::SettlementService;
#[cfg(feature = "testutils")]
pub use settlement_service_trait::MockSettlementServiceTrait;
pub use settlement_service_trait::SettlementServiceTrait;
