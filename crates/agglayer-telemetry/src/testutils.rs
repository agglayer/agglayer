//! Test-only helpers for asserting on real exported metrics.

use prometheus::Registry;

use crate::{encode_registry, init_meter_provider};

/// Test harness binding the process-global meter provider to a local
/// prometheus registry.
///
/// The default global meter is a no-op that silently discards instruments
/// and observable-gauge callbacks, so a test asserting on exported metrics
/// must install a real provider first. [`MetricsHarness::install`] reuses
/// the same installation code path as the production metrics server
/// ([`ServerBuilder`](crate::ServerBuilder)).
///
/// # Isolation
///
/// Installing the harness replaces the meter provider for the whole
/// process. Tests using it rely on nextest's process-per-test isolation
/// and must each install their own harness.
#[derive(Debug)]
pub struct MetricsHarness {
    registry: Registry,
}

impl MetricsHarness {
    /// Installs a fresh `SdkMeterProvider` backed by a private prometheus
    /// registry as the process-global meter provider.
    ///
    /// # Panics
    ///
    /// Panics when the prometheus exporter cannot be built, which indicates
    /// a bug in the metrics pipeline rather than a recoverable condition.
    #[must_use]
    pub fn install() -> Self {
        let registry = Registry::new();
        init_meter_provider(&registry);
        Self { registry }
    }

    /// Encodes the current registry content as prometheus text.
    ///
    /// # Panics
    ///
    /// Panics when the registry content cannot be encoded, which indicates
    /// a bug in the metrics pipeline rather than a recoverable condition.
    #[must_use]
    pub fn gather(&self) -> String {
        encode_registry(&self.registry).expect("failed to encode the metrics registry")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_binds_global_meter_and_gather_exports_instruments() {
        // This test deliberately owns the process-global meter provider:
        // nextest runs one process per test, so nothing else can race it.
        let harness = MetricsHarness::install();

        let counter = opentelemetry::global::meter("harness-test")
            .u64_counter("harness_test_counter")
            .build();
        counter.add(3, &[]);

        // The prometheus exporter appends `_total` to counter names.
        let metrics = harness.gather();
        assert!(
            metrics
                .lines()
                .any(|line| line.starts_with("harness_test_counter") && line.ends_with(" 3")),
            "expected a harness_test_counter sample, got:\n{metrics}"
        );
    }
}
