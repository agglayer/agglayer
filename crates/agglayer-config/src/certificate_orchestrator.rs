use prover::ProverConfig;
use serde::{Deserialize, Serialize};

pub mod prover;

/// The CertificateOrchestrator configuration.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct CertificateOrchestrator {
    #[serde(default = "default_input_backpressure_buffer_size_default")]
    pub input_backpressure_buffer_size: usize,

    #[serde(default = "default_prover_config_default")]
    pub prover: ProverConfig,
}

impl Default for CertificateOrchestrator {
    fn default() -> Self {
        Self {
            input_backpressure_buffer_size: default_input_backpressure_buffer_size_default(),
            prover: default_prover_config_default(),
        }
    }
}

fn default_input_backpressure_buffer_size_default() -> usize {
    1_000
}

/// The default prover configuration.
fn default_prover_config_default() -> ProverConfig {
    ProverConfig::SP1Local {}
}
