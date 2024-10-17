use serde::{Deserialize, Serialize};

/// Proof is a wrapper around all the different types of proofs that can be
/// generated
#[derive(Clone, Serialize, Deserialize)]
pub enum Proof {
    #[allow(unused)]
    SP1(sp1_sdk::SP1ProofWithPublicValues),
}
