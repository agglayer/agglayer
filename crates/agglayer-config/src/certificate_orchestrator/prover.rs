use serde::{Deserialize, Serialize};

/// The different prover configuration.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ProverConfig {
    SP1Network {},
    SP1Local {},
    SP1Mock {},
}
