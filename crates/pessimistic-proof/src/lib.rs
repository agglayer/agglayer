pub mod local_exit_tree;

pub mod proof;
pub use proof::PessimisticProofOutput;
pub use proof::Proof;

pub mod local_balance_tree;

pub mod bridge_exit;

pub mod global_index;
pub mod imported_bridge_exit;
pub mod local_state;
pub mod nullifier_tree;
pub mod utils;

pub use local_state::LocalNetworkState;

pub mod keccak {
    pub use pessimistic_proof_core::keccak::*;
}

pub use pessimistic_proof_core::local_state::NetworkState;
pub use pessimistic_proof_core::multi_batch_header;
pub use pessimistic_proof_core::proof::ProofError;

pub mod core {
    pub use pessimistic_proof_core::aggchain_proof::AggchainData;
    pub use pessimistic_proof_core::generate_pessimistic_proof;
}

pub mod error;

/// ELF of the pessimistic proof program
pub const ELF: &[u8] =
    include_bytes!("../../pessimistic-proof-program/elf/riscv32im-succinct-zkvm-elf");
