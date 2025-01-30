//! A collection of shared testing utilities.

pub mod event_data;
pub mod forest;
pub mod runner;
pub mod sample_data;

/// The ELF we want to execute inside the zkVM.
pub const PESSIMISTIC_PROOF_ELF: &[u8] =
    include_bytes!("../../pessimistic-proof-program/elf/riscv32im-succinct-zkvm-elf");

/// The Aggchain proof ECDSA ELF example.
pub const AGGCHAIN_PROOF_ECDSA_ELF: &[u8] =
    include_bytes!("../aggchain-proof-ecdsa-example/program/elf/riscv32im-succinct-zkvm-elf");
