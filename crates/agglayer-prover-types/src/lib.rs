use agglayer_types::ProofVerificationError;
use bincode::{
    config::{BigEndian, FixintEncoding, WithOtherEndian, WithOtherIntEncoding},
    DefaultOptions, Options as _,
};
use pessimistic_proof::ProofError;
use serde::{Deserialize, Serialize};

pub const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("generated/agglayer.prover.bin");

#[path = "generated/agglayer.prover.v1.rs"]
#[rustfmt::skip]
#[allow(warnings)]
pub mod v1;

pub fn default_bincode_options(
) -> WithOtherIntEncoding<WithOtherEndian<DefaultOptions, BigEndian>, FixintEncoding> {
    DefaultOptions::new()
        .with_big_endian()
        .with_fixint_encoding()
}

#[derive(Debug, Serialize, Deserialize, thiserror::Error)]
pub enum Error {
    #[error("Unable to execute prover")]
    UnableToExecuteProver,

    #[error("Prover failed: {0}")]
    ProverFailed(String),

    #[error("Prover verification failed: {0}")]
    ProofVerificationFailed(#[from] ProofVerificationError),

    #[error("Prover executor failed: {0}")]
    ExecutorFailed(#[from] ProofError),
}
