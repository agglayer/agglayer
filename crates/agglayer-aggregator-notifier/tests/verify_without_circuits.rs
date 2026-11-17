//! The node image ships without the SP1 circuit artifacts (~2GB).
//!
//! The node only verifies plonk proofs, with the verifying key embedded in
//! `sp1-verifier`. Up to SP1 v5, verifying downloaded the artifacts into
//! `~/.sp1/circuits` when they were missing. This test fails if an SP1 upgrade
//! brings that back.
//!
//! The fixtures are a plonk proof and the verifying key it was made for, as
//! returned by `Runner::generate_plonk_proof` of the test suite:
//! `proof.save(..)` and the JSON of the key. Only an SP1 upgrade that changes
//! the plonk circuit invalidates them.

use pessimistic_proof::ELF;
use sp1_sdk::{
    blocking::{EnvProver, Prover, ProverClient},
    Elf, SP1ProofWithPublicValues, SP1VerifyingKey,
};

const PROOF_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/plonk_proof.bin"
);
const VKEY: &[u8] = include_bytes!("fixtures/plonk_vkey.json");

#[test]
fn plonk_proof_is_verified_without_circuit_artifacts() {
    // The artifacts directories are placed under a regular file, so that any
    // attempt to install them fails right away instead of downloading them.
    let blocker = std::env::temp_dir().join(format!("agglayer-no-circuits-{}", std::process::id()));
    std::fs::write(&blocker, b"").unwrap();
    std::env::set_var("SP1_PLONK_CIRCUIT_PATH", blocker.join("plonk"));
    std::env::set_var("SP1_GROTH16_CIRCUIT_PATH", blocker.join("groth16"));

    let proof = SP1ProofWithPublicValues::load(PROOF_PATH).unwrap();
    let verifying_key: SP1VerifyingKey = serde_json::from_slice(VKEY).unwrap();

    // Same client, setup and verification as the certifier.
    let verifier = EnvProver::Light(ProverClient::builder().light().build());
    verifier.setup(Elf::Static(ELF)).unwrap();
    verifier.verify(&proof, &verifying_key, None).unwrap();

    assert!(
        blocker.is_file(),
        "the SP1 circuit artifacts directory was created"
    );
    std::fs::remove_file(&blocker).unwrap();
}
