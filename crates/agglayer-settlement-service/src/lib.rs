//! The settlement service handles transaction settlement operations for the
//! AggLayer.
//!
//! This service consists of:
//! - [`SettlementService`]: The main service orchestrating settlement
//!   operations
//! - [`SettlementTask`]: Worker task for processing individual settlements

#![allow(dead_code)] // TODO remove after settlement service is integrated in the rest of the app

mod helpers;
pub mod settlement_service;
mod settlement_task;

pub use settlement_service::SettlementService;
