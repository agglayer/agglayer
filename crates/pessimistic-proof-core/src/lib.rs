pub use agglayer_primitives::keccak;

pub mod proof;
pub use proof::{generate_pessimistic_proof, PessimisticProofOutput, ProofError};

pub mod local_balance_tree;

pub mod aggchain_proof;
pub mod local_state;
pub mod multi_batch_header;
pub mod nullifier_tree;

pub use local_state::NetworkState;

include!(concat!(env!("OUT_DIR"), "/version.rs"));
pub const PESSIMISTIC_PROOF_PROGRAM_SELECTOR: [u8; 4] =
    PESSIMISTIC_PROOF_PROGRAM_VERSION.to_be_bytes();
