use serde::{Deserialize, Serialize};

use crate::{shutdown::ShutdownConfig, telemetry::TelemetryConfig, Log};

/// The Agglayer Prover configuration.
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct ProverConfig {
    /// The log configuration.
    #[serde(alias = "Log")]
    pub log: Log,

    /// Telemetry configuration.
    #[serde(alias = "Telemetry")]
    pub telemetry: TelemetryConfig,

    /// The list of configuration options used during shutdown.
    #[serde(default)]
    pub shutdown: ShutdownConfig,
}
