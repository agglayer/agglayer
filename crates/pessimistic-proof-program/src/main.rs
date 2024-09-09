#![no_main]

use bincode::Options;
use pessimistic_proof::local_exit_tree::hasher::Keccak256Hasher;
use pessimistic_proof::multi_batch_header::MultiBatchHeader;
use pessimistic_proof::{generate_pessimistic_proof, LocalNetworkState};

sp1_zkvm::entrypoint!(main);
pub fn main() {
    let initial_state = sp1_zkvm::io::read::<LocalNetworkState>();
    let batch_header = sp1_zkvm::io::read::<MultiBatchHeader<Keccak256Hasher>>();

    let outputs = generate_pessimistic_proof(initial_state, &batch_header).unwrap();

    let pp_inputs = bincode::DefaultOptions::new()
        .with_big_endian()
        .with_fixint_encoding()
        .allow_trailing_bytes()
        .serialize(&outputs)
        .unwrap();

    sp1_zkvm::io::commit_slice(&pp_inputs);
}
