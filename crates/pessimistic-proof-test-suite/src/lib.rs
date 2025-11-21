//! A collection of shared testing utilities.

use eyre::Context as _;
use sp1_sdk::HashableKey as _;

pub mod event_data;
pub mod forest;
pub mod runner;
pub mod sample_data;
pub mod test_vector;

/// The ELF we want to execute inside the zkVM.
pub const PESSIMISTIC_PROOF_ELF: &[u8] = pessimistic_proof::ELF;

/// The Aggchain proof ECDSA ELF example.
pub const AGGCHAIN_PROOF_ECDSA_ELF: &[u8] =
    include_bytes!("../aggchain-proof-ecdsa-example/program/elf/riscv32im-succinct-zkvm-elf");

pub async fn compute_program_vkey(program: &'static [u8]) -> eyre::Result<String> {
    let vkey = prover_executor::Executor::compute_program_vkey(program)
        .await
        .context("Failed to compute program vkey")?;
    Ok(vkey.bytes32())
}
