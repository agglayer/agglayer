use serde::{Deserialize, Serialize};

/// Wrapping applied to the pessimistic proof before it is settled on L1.
///
/// This also picks the `AgglayerGateway` route the proof is sent to, so the
/// two can never disagree.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ProofWrapping {
    #[default]
    Groth16,
    Plonk,
}
