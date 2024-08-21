use serde::{Deserialize, Serialize};

/// Public inputs of the aggregation proof of several pessimistic proofs.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AggregatedPPPublicInputs {
    /// Verification key for the pessimistic proof
    pub vkey: [u32; 8],
    // TODO: add more stuff
}
