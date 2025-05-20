#![no_main]

use bincode::Options;
use pessimistic_proof_core::{
    generate_pessimistic_proof, keccak::Keccak256Hasher, multi_batch_header::MultiBatchHeader,
    NetworkState, PessimisticProofOutput,
};

sp1_zkvm::entrypoint!(main);
pub fn main() {
    let initial_state = sp1_zkvm::io::read::<NetworkState>();
    let batch_header = sp1_zkvm::io::read::<MultiBatchHeader<Keccak256Hasher>>();

    let (outputs, _targets) = generate_pessimistic_proof(initial_state, &batch_header).unwrap();

    let pp_inputs = PessimisticProofOutput::bincode_options()
        .serialize(&outputs)
        .unwrap();

    sp1_zkvm::io::commit_slice(&pp_inputs);
}
