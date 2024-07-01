#![no_main]

use pessimistic_proof::{certificate::Certificate, generate_full_proof};

sp1_zkvm::entrypoint!(main);

pub fn main() {
    let certificates = sp1_zkvm::io::read::<Vec<Certificate>>();

    let new_roots = generate_full_proof(&certificates).unwrap();

    sp1_zkvm::io::commit(&new_roots);
}
