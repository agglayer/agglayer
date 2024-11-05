#![no_main]
sp1_zkvm::entrypoint!(main);

use bincode::Options;
use pessimistic_proof::aggregation::combine::combine_proofs;
use pessimistic_proof::aggregation::wrap::AggregationProofOutput;
use sha2::Digest;
use sha2::Sha256;

pub fn main() {
    // Read the verification keys.
    let vkey = sp1_zkvm::io::read::<[u32; 8]>();

    // Read the public values.
    let public_values0 = sp1_zkvm::io::read::<Vec<u8>>();
    let public_values1 = sp1_zkvm::io::read::<Vec<u8>>();

    // Verify the first proof.
    let public_values_digest0 = Sha256::digest(&public_values0);
    sp1_zkvm::lib::verify::verify_sp1_proof(&vkey, &public_values_digest0.into());

    // Verify the second proof.
    let public_values_digest1 = Sha256::digest(&public_values1);
    sp1_zkvm::lib::verify::verify_sp1_proof(&vkey, &public_values_digest1.into());

    let wrapped_proof0: AggregationProofOutput =
        bincode::deserialize::<AggregationProofOutput>(&public_values0)
            .expect("Failed to deserialize");
    let wrapped_proof1: AggregationProofOutput =
        bincode::deserialize::<AggregationProofOutput>(&public_values1)
            .expect("Failed to deserialize");

    let combined_proof_output = combine_proofs(wrapped_proof0, wrapped_proof1);

    sp1_zkvm::io::commit(&combined_proof_output);
}
