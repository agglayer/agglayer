#![no_main]
sp1_zkvm::entrypoint!(main);

// use pessimistic_proof::aggregation::AggregatedPPPublicInputs;
use sha2::Digest;
use sha2::Sha256;

// TODO: Do something with the proofs and pis.
pub fn main() {
    // Read the verification keys.
    let vkey = sp1_zkvm::io::read::<[u32; 8]>();

    // Read the public values.
    let public_values = sp1_zkvm::io::read::<Vec<Vec<u8>>>();

    // Verify the proofs.
    for i in 0..public_values.len() {
        let public_values = &public_values[i];
        let public_values_digest = Sha256::digest(public_values);
        sp1_zkvm::lib::verify::verify_sp1_proof(&vkey, &public_values_digest.into());
    }

    // let pis = AggregatedPPPublicInputs { vkey };
    // sp1_zkvm::io::commit(&pis);
}
