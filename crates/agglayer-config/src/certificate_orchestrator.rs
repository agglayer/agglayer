use serde::Deserialize;

/// The CertificateOrchestrator configuration.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct CertificateOrchestrator {
    #[serde(default = "default_input_backpressure_buffer_size_default")]
    pub input_backpressure_buffer_size: usize,
}

impl Default for CertificateOrchestrator {
    fn default() -> Self {
        Self {
            input_backpressure_buffer_size: default_input_backpressure_buffer_size_default(),
        }
    }
}

fn default_input_backpressure_buffer_size_default() -> usize {
    1_000
}
