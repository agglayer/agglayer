#![no_main]
sp1_zkvm::entrypoint!(main);

use bincode::Options;
use pessimistic_proof::aggregation::wrap_proof;
use pessimistic_proof::PessimisticProofOutput;
use sha2::Digest;
use sha2::Sha256;

pub fn main() {
    // Read the verification keys.
    let vkey = sp1_zkvm::io::read::<[u32; 8]>();

    // Read the public values.
    let public_values = sp1_zkvm::io::read::<Vec<u8>>();

    // Verify the proof.
    let public_values_digest = Sha256::digest(&public_values);
    sp1_zkvm::lib::verify::verify_sp1_proof(&vkey, &public_values_digest.into());

    let proof_output: PessimisticProofOutput = bincode::DefaultOptions::new()
        .deserialize(&public_values)
        .expect("Failed to deserialize");

    let tmp_rer = sp1_zkvm::io::read::<_>();
    let new_mer = sp1_zkvm::io::read::<_>();
    let new_rer = sp1_zkvm::io::read::<_>();
    let tmp_rer_proof = sp1_zkvm::io::read::<_>();
    let imported_lers_witness = sp1_zkvm::io::read::<_>();

    let stage1_agg_proof_output = wrap_proof(
        proof_output,
        tmp_rer,
        new_mer,
        new_rer,
        tmp_rer_proof,
        imported_lers_witness,
    );

    sp1_zkvm::io::commit(&stage1_agg_proof_output);
}
