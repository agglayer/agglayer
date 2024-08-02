//! A collection of shared testing utilities.

// This is to suppress dead code warnings since tests use subset of functionality available here.
// If this module is extracted into a crate, this should be removed to get back the warnings.
#![allow(unused)]

pub mod event_data;
pub mod forest;
pub mod runner;
pub mod sample_data;

/// The ELF we want to execute inside the zkVM.
pub const PESSIMISTIC_PROOF_ELF: &[u8] =
    include_bytes!("../../../pessimistic-proof-program/elf/riscv32im-succinct-zkvm-elf");
