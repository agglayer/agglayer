#![no_main]

use pessimistic_proof::{batch_header::BatchHeader, generate_leaf_proof, LocalNetworkState};

sp1_zkvm::entrypoint!(main);

pub fn main() {
    let initial_state = sp1_zkvm::io::read::<LocalNetworkState>();
    let batch_header = sp1_zkvm::io::read::<BatchHeader>();

    let new_roots = generate_leaf_proof(initial_state, &batch_header).unwrap();

    sp1_zkvm::io::commit(&new_roots);
}
