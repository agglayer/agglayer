//! The settlement service handles transaction settlement operations for the
//! AggLayer.
//!
//! This service consists of:
//! - [`SettlementService`]: The main service orchestrating settlement
//!   operations
//! - [`SettlementTask`]: Worker task for processing individual settlements

#![allow(dead_code)] // TODO remove after settlement service is integrated in the rest of the app

pub mod settlement_service;
pub mod settlement_task;

#[cfg(any(test, feature = "testutils"))]
pub use settlement_service::testutils;
#[cfg(any(test, feature = "testutils"))]
pub use settlement_service::MockSettlementServiceTrait;
pub use settlement_service::{RetrievedSettlementResult, SettlementService, SettlementServiceTrait};
pub use settlement_task::{
    ClientError, ClientErrorType, ContractCallOutcome, ContractCallResult, SettlementJob,
    SettlementJobResult,
};
