//! The settlement service handles transaction settlement operations for the
//! AggLayer.
//!
//! This service consists of:
//! - [`SettlementService`]: The main service orchestrating settlement
//!   operations
//! - [`SettlementTask`]: Worker task for processing individual settlements
//!
//! [`SettlementTask`]: settlement_task::SettlementTask

#![allow(dead_code)] // TODO remove after settlement service is integrated in the rest of the app

pub mod settlement_service;
pub mod settlement_service_trait;
mod settlement_task;
mod utils;

pub use settlement_service::SettlementService;
#[cfg(feature = "testutils")]
pub use settlement_service_trait::MockSettlementServiceTrait;
pub use settlement_service_trait::SettlementServiceTrait;
