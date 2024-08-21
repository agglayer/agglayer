use serde::Serialize;
use sp1_sdk::{ProverClient, SP1ProofWithPublicValues, SP1Stdin, SP1VerifyingKey};

pub const PESSIMISTIC_PROOF_ELF: &[u8] =
    include_bytes!("../../pessimistic-proof-program/elf/riscv32im-succinct-zkvm-elf");

/// Generate the pessimistic proof for the given local state and batch header.
pub fn prove_pessimistic_proof(
    local_state: impl Serialize,
    batch_header: impl Serialize,
) -> SP1ProofWithPublicValues {
    let mut stdin = SP1Stdin::new();
    stdin.write(&local_state);
    stdin.write(&batch_header);

    // Generate the proof for the given program and input.
    let client = ProverClient::new();
    let (pk, _vk) = client.setup(PESSIMISTIC_PROOF_ELF);
    client.prove(&pk, stdin).compressed().run().unwrap()
}

/// Verification key for the pessimistic proof.
pub fn vk_pessimistic_proof() -> SP1VerifyingKey {
    // Generate the proof for the given program and input.
    let client = ProverClient::new();
    let (_pk, vk) = client.setup(PESSIMISTIC_PROOF_ELF);
    vk
}
