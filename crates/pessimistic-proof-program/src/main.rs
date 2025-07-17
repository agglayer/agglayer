#![no_main]

use pessimistic_proof_core::{
    generate_pessimistic_proof, keccak::Keccak256Hasher, multi_batch_header::MultiBatchHeader,
    NetworkState, PessimisticProofOutput,
};

sp1_zkvm::entrypoint!(main);
pub fn main() {
    let initial_state_bytes = sp1_zkvm::io::read_vec();
    let initial_state = rkyv::from_bytes::<NetworkState, rkyv::rancor::Error>(&initial_state_bytes)
        .expect("Failed to deserialize witness data.");

    let batch_header_bytes = sp1_zkvm::io::read_vec();
    let batch_header = rkyv::from_bytes::<MultiBatchHeader<Keccak256Hasher>, rkyv::rancor::Error>(
        &batch_header_bytes,
    )
    .expect("Failed to deserialize batch header.");

    let (outputs, _targets) = generate_pessimistic_proof(initial_state, &batch_header).unwrap();

    let pp_inputs = PessimisticProofOutput::bincode_codec()
        .serialize(&outputs)
        .unwrap();

    sp1_zkvm::io::commit_slice(&pp_inputs);
}
