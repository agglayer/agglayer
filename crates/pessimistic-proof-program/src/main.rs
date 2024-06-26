#![no_main]

use pessimistic_proof::{batch::Batch, generate_full_proof};

sp1_zkvm::entrypoint!(main);

pub fn main() {
    let batches = sp1_zkvm::io::read::<Vec<Batch>>();

    let new_roots = generate_full_proof(&batches).unwrap();

    sp1_zkvm::io::commit(&new_roots);
}
