//! The node image ships without the SP1 circuit artifacts (~2GB).
//!
//! The node only verifies plonk proofs, with the verifying key embedded in
//! `sp1-verifier`. Up to SP1 v5, verifying downloaded the artifacts into
//! `~/.sp1/circuits` when they were missing. This test fails if an SP1 upgrade
//! brings that back.
//!
//! The fixtures are a plonk proof and the verifying key it was made for. The
//! SDK rejects a proof whose `sp1_version` differs from `SP1_CIRCUIT_VERSION`,
//! so they have to be regenerated at every SP1 upgrade that bumps it: take the
//! proof and key returned by `Runner::generate_plonk_proof` of the test suite,
//! then `proof.save(..)` and write the key as JSON.

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
    std::env::set_var("SP1_PLONK_CIRCUIT_PATH", format!("{PROOF_PATH}/plonk"));
    std::env::set_var("SP1_GROTH16_CIRCUIT_PATH", format!("{PROOF_PATH}/groth16"));

    let proof = SP1ProofWithPublicValues::load(PROOF_PATH).unwrap();
    let verifying_key: SP1VerifyingKey = serde_json::from_slice(VKEY).unwrap();

    // Same client as the certifier. `setup` is only called to check that it
    // does not need the circuits either: the proof is verified against the
    // fixture key, so that a change of the program does not break the test.
    let verifier = EnvProver::Light(ProverClient::builder().light().build());
    verifier.setup(Elf::Static(ELF)).unwrap();
    verifier.verify(&proof, &verifying_key, None).unwrap();
}
