pub mod proof;
pub use proof::{PessimisticProofOutput, Proof};

pub mod local_balance_tree;
pub mod local_exit_tree;

pub mod local_state;
pub mod nullifier_tree;

pub use local_state::LocalNetworkState;

pub mod keccak {
    pub use pessimistic_proof_core::keccak::*;
}

pub use pessimistic_proof_core::{
    local_state::NetworkState, multi_batch_header, proof::ProofError,
};
pub use unified_bridge;

pub mod core {
    pub use pessimistic_proof_core::{
        aggchain_proof::{AggchainData, Vkey},
        generate_pessimistic_proof,
        local_state::commitment,
        PESSIMISTIC_PROOF_PROGRAM_SELECTOR, PESSIMISTIC_PROOF_PROGRAM_VERSION,
    };
}

pub mod error;

/// ELF of the pessimistic proof program
pub const ELF: &[u8] = agglayer_elf_build::elf_bytes!();
