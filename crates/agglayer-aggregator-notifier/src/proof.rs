use serde::Serialize;

/// Proof is a wrapper around all the different types of proofs that can be
/// generated
#[derive(Clone, Serialize)]
pub enum Proof {
    SP1(sp1_sdk::SP1ProofWithPublicValues),
}
