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
    include_bytes!("../aggchain-proof-ecdsa-example/program/elf/riscv64im-succinct-zkvm-elf");

pub async fn compute_program_vkey(program: &'static [u8]) -> eyre::Result<String> {
    let vkey = prover_executor::Executor::compute_program_vkey(program)
        .await
        .context("Failed to compute program vkey")?;
    Ok(vkey.bytes32())
}

/// A dummy SP1 [`Proof`](agglayer_types::Proof) whose public values decode to a
/// zeroed [`PessimisticProofOutput`](pessimistic_proof::PessimisticProofOutput),
/// for tests that need a deserializable settlement proof without running the
/// prover.
pub fn dummy_settlement_proof() -> agglayer_types::Proof {
    use pessimistic_proof::PessimisticProofOutput;

    let output = PessimisticProofOutput {
        prev_local_exit_root: agglayer_tries::roots::LocalExitRoot::new(
            agglayer_types::Digest::ZERO,
        ),
        prev_pessimistic_root: agglayer_types::Digest::ZERO,
        l1_info_root: agglayer_types::Digest::ZERO,
        origin_network: agglayer_types::NetworkId::new(0),
        aggchain_hash: agglayer_types::Digest::ZERO,
        new_local_exit_root: agglayer_tries::roots::LocalExitRoot::new(
            agglayer_types::Digest::ZERO,
        ),
        new_pessimistic_root: agglayer_types::Digest::ZERO,
    };
    let public_values = PessimisticProofOutput::bincode_codec()
        .serialize(&output)
        .expect("serialize dummy pessimistic proof output");

    // A mock Plonk proof: an empty `encoded_proof` makes `.bytes()` return an
    // empty vector (no prover needed), and the public values still decode to
    // `output`.
    agglayer_types::Proof::SP1(sp1_sdk::SP1ProofWithPublicValues {
        proof: sp1_sdk::SP1Proof::Plonk(sp1_prover::PlonkBn254Proof {
            public_inputs: std::array::from_fn(|_| String::new()),
            encoded_proof: String::new(),
            raw_proof: String::new(),
            plonk_vkey_hash: [0u8; 32],
        }),
        public_values: sp1_sdk::SP1PublicValues::from(public_values.as_slice()),
        sp1_version: String::new(),
        tee_proof: None,
    })
}
