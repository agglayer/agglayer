use serde::{Deserialize, Serialize};

/// The different prover configuration.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum ProverConfig {
    #[serde(rename = "sp1-local")]
    SP1Local {},
    #[serde(rename = "sp1-mock")]
    SP1Mock {},
    #[serde(rename = "sp1-network")]
    SP1Network {},
}

impl Default for ProverConfig {
    fn default() -> Self {
        Self::SP1Local {}
    }
}
