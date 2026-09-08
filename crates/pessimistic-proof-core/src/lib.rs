pub use agglayer_primitives::keccak;

pub mod proof;
pub use proof::{generate_pessimistic_proof, PessimisticProofOutput, ProofError};

pub mod local_balance_tree;

pub mod aggchain_data;
pub mod local_state;
pub mod multi_batch_header;
pub mod nullifier_tree;

pub use local_state::NetworkState;

include!(concat!(env!("OUT_DIR"), "/version.rs"));

/// Selector high byte is the proof wrapping, low three bytes are the program
/// major, so the two wrappings can never claim the same route.
const fn selector(wrapping: u8) -> [u8; 4] {
    let [_, a, b, c] = PESSIMISTIC_PROOF_PROGRAM_VERSION.to_be_bytes();
    [wrapping, a, b, c]
}

pub const PP_SELECTOR_PLONK: [u8; 4] = selector(0x00);
pub const PP_SELECTOR_GROTH16: [u8; 4] = selector(0x01);
