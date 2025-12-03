pub const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("generated/agglayer.prover.bin");

#[path = "generated/agglayer.prover.v1.rs"]
#[rustfmt::skip]
#[allow(warnings)]
pub mod v1;

/// Proof is a wrapper around all the different types of proofs that can be
/// generated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Proof {
    SP1(SP1ProofWithPublicValues),
}
pub mod error;
pub use agglayer_interop::types::bincode;
pub use error::{Error, ErrorWrapper};
use serde::{Deserialize, Serialize};
use sp1_sdk::SP1ProofWithPublicValues;
