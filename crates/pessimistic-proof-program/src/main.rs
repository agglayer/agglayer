#![no_main]

use pessimistic_proof::local_exit_tree::hasher::Keccak256Hasher;
use pessimistic_proof::multi_batch_header::MultiBatchHeader;
use pessimistic_proof::{generate_pessimistic_proof, LocalNetworkState};

sp1_zkvm::entrypoint!(main);

pub fn main() {
    let initial_state = sp1_zkvm::io::read::<LocalNetworkState>();
    let batch_header = sp1_zkvm::io::read::<MultiBatchHeader<Keccak256Hasher>>();

    let outputs = generate_pessimistic_proof(initial_state, &batch_header).unwrap();

    sp1_zkvm::io::commit(&outputs);
}
