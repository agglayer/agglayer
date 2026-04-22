use agglayer_sp1::ProofMode;
use sp1_sdk::{Prover, ProverClient, SP1ProofMode};

/// A minimal valid RISC-V ELF. The mock prover's `setup` pipeline parses the
/// ELF header, so `b"\x7fELF"` alone is not enough — we reuse the same empty
/// ELF artifact the rest of the workspace ships for mock-proof tests.
const EMPTY_ELF: &[u8] = include_bytes!("empty.elf");

fn mock_proof(mode: SP1ProofMode) -> sp1_sdk::SP1ProofWithPublicValues {
    let client = ProverClient::builder().mock().build();
    sp1_sdk::SP1ProofWithPublicValues::create_mock_proof(
        &client.setup(EMPTY_ELF).0,
        sp1_sdk::SP1PublicValues::new(),
        mode,
        sp1_sdk::SP1_CIRCUIT_VERSION,
    )
}

#[test]
fn core_mode() {
    let proof = mock_proof(SP1ProofMode::Core);
    assert_eq!(ProofMode::from(&proof.proof), ProofMode::Core);
}

#[test]
fn compressed_mode() {
    let proof = mock_proof(SP1ProofMode::Compressed);
    assert_eq!(ProofMode::from(&proof.proof), ProofMode::Compressed);
}

#[test]
fn plonk_mode() {
    let proof = mock_proof(SP1ProofMode::Plonk);
    assert_eq!(ProofMode::from(&proof.proof), ProofMode::Plonk);
}

#[test]
fn groth16_mode() {
    let proof = mock_proof(SP1ProofMode::Groth16);
    assert_eq!(ProofMode::from(&proof.proof), ProofMode::Groth16);
}
