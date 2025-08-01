//! Clock metrics for observability
//!
//! This module provides comprehensive metrics for monitoring the Agglayer block
//! clock, including connection status, block processing, epoch tracking, and
//! health monitoring.

use lazy_static::lazy_static;
use opentelemetry::{global, metrics::*};

const AGGLAYER_CLOCK_OTEL_SCOPE_NAME: &str = "agglayer_node_clock";

lazy_static! {
    /// Current block height gauge
    pub static ref CURRENT_BLOCK_HEIGHT: Gauge<u64> = global::meter(AGGLAYER_CLOCK_OTEL_SCOPE_NAME)
        .u64_gauge("current_block_height")
        .with_description("Current block height processed by the clock")
        .build();

    /// Current epoch number gauge
    pub static ref CURRENT_EPOCH: Gauge<u64> = global::meter(AGGLAYER_CLOCK_OTEL_SCOPE_NAME)
        .u64_gauge("current_epoch")
        .with_description("Current epoch number")
        .build();

    /// Gauge for connection status (1 = connected, 0 = disconnected)
    pub static ref CONNECTION_STATUS: Gauge<u64> = global::meter(AGGLAYER_CLOCK_OTEL_SCOPE_NAME)
        .u64_gauge("connection_status")
        .with_description("WebSocket connection status (1=connected, 0=disconnected)")
        .build();

    /// Counter for reconnection attempts
    pub static ref RECONNECTION_ATTEMPTS: Counter<u64> = global::meter(AGGLAYER_CLOCK_OTEL_SCOPE_NAME)
        .u64_counter("reconnection_attempts_total")
        .with_description("Total number of reconnection attempts")
        .build();

    /// Gauge for health status (1=healthy, 0.5=starting, 0.25=degraded, 0=unhealthy)
    pub static ref HEALTH_STATUS: Gauge<f64> = global::meter(AGGLAYER_CLOCK_OTEL_SCOPE_NAME)
        .f64_gauge("health_status")
        .with_description("Current health status of the block clock")
        .build();

    /// Counter for subscription lag events
    pub static ref SUBSCRIPTION_LAG: Counter<u64> = global::meter(AGGLAYER_CLOCK_OTEL_SCOPE_NAME)
        .u64_counter("blocks_subscription_lag_total")
        .with_description("Total number of subscription lag events")
        .build();

    /// Counter for connection errors
    pub static ref CONNECTION_ERRORS: Counter<u64> = global::meter(AGGLAYER_CLOCK_OTEL_SCOPE_NAME)
        .u64_counter("connection_errors_total")
        .with_description("Total number of connection errors")
        .build();
}

/// Helper function to record clock startup
#[inline]
pub fn record_clock_startup() {
    HEALTH_STATUS.record(0.5, &[]); // Starting status
    CONNECTION_STATUS.record(0, &[]); // Initially disconnected
}

/// Helper function to record successful connection
#[inline]
pub fn record_connection_established() {
    CONNECTION_STATUS.record(1, &[]);
    HEALTH_STATUS.record(1.0, &[]); // Healthy status
}

/// Helper function to record connection lost
#[inline]
pub fn record_connection_failed() {
    CONNECTION_STATUS.record(0, &[]);
    CONNECTION_ERRORS.add(1, &[]);
}

/// Helper function to record reconnection attempt
#[inline]
pub fn record_reconnection_attempt() {
    RECONNECTION_ATTEMPTS.add(1, &[]);
}

/// Helper function to record current epoch
#[inline]
pub fn record_current_epoch(epoch_number: u64) {
    CURRENT_EPOCH.record(epoch_number, &[]);
}

/// Helper function to record subscription lag
#[inline]
pub fn record_subscription_lag(lagged_count: u64) {
    SUBSCRIPTION_LAG.add(lagged_count, &[]);
}

/// Helper function to record clock shutdown
#[inline]
pub fn record_clock_shutdown() {
    CONNECTION_STATUS.record(0, &[]);
    HEALTH_STATUS.record(0.0, &[]); // Unhealthy status
}

/// Helper function to record the current block height
#[inline]
pub fn record_current_block_height(height: u64) {
    CURRENT_BLOCK_HEIGHT.record(height, &[]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_initialization() {
        // Test that metrics can be accessed without panicking
        CURRENT_BLOCK_HEIGHT.record(100, &[]);
        CURRENT_EPOCH.record(10, &[]);
        CONNECTION_STATUS.record(1, &[]);
        HEALTH_STATUS.record(1.0, &[]);
    }

    #[test]
    fn test_helper_functions() {
        // Test helper functions
        record_clock_startup();
        record_connection_established();
        record_connection_failed();
        record_reconnection_attempt();

        record_current_block_height(1000);
        record_current_epoch(50);
        record_subscription_lag(5);
    }
}
