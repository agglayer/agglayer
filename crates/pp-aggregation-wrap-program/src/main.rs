#![no_main]
sp1_zkvm::entrypoint!(main);

use bincode::Options;
use pessimistic_proof::aggregation::wrap::wrap_proof;
use pessimistic_proof::PessimisticProofOutput;
use sha2::Digest;
use sha2::Sha256;

pub fn main() {
    // Read the verification key.
    let vkey = sp1_zkvm::io::read::<[u32; 8]>();

    // Read the public values.
    let public_values = sp1_zkvm::io::read::<Vec<u8>>();

    // Verify the proof.
    let public_values_digest = Sha256::digest(&public_values);
    sp1_zkvm::lib::verify::verify_sp1_proof(&vkey, &public_values_digest.into());

    let pp_output: PessimisticProofOutput = bincode::DefaultOptions::new()
        .deserialize(&public_values)
        .expect("Failed to deserialize");

    let tmp_arer = sp1_zkvm::io::read::<_>();
    let selected_mer = sp1_zkvm::io::read::<_>();
    let selected_rer = sp1_zkvm::io::read::<_>();
    let tmp_arer_proof = sp1_zkvm::io::read::<_>();
    let imported_lers_witness = sp1_zkvm::io::read::<_>();

    let wrapped_proof_output = wrap_proof(
        pp_output,
        tmp_arer,
        selected_mer,
        selected_rer,
        tmp_arer_proof,
        imported_lers_witness,
    );

    sp1_zkvm::io::commit(&wrapped_proof_output);
}
